use super::super::ProjectService;
use crate::{
    data_context::DATA_CONTEXT,
    domain::file_tree::{FileTreeOperation, FileTreeOutcome, ProjectClipboard},
    domain::project::DirectorySnapshot,
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Result, bail};
use std::path::{Component, Path};

impl ProjectService {
    // Explicit row edits publish immediately; native watcher batches stay periodic.
    pub(in crate::application::project) async fn refresh_local_directories(
        &self,
        parents: &[&Path],
        moved: Option<(&Path, &Path)>,
    ) {
        let mut directories: std::collections::HashSet<_> =
            parents.iter().map(|path| path.to_path_buf()).collect();
        if let Some((source, destination)) = moved {
            for path in DATA_CONTEXT.project.snapshots().1.keys() {
                if let Ok(suffix) = path.strip_prefix(source) {
                    directories.insert(path.clone());
                    directories.insert(destination.join(suffix));
                }
            }
        }
        let mut updates = Vec::new();
        for directory in directories {
            let snapshot = match INFRASTRUCTURE
                .filesystem
                .list_directory(DATA_CONTEXT.project_root().to_owned(), directory.clone())
                .await
            {
                Ok(entries) => DirectorySnapshot {
                    entries: entries.into(),
                    error: None,
                },
                Err(error) => {
                    tracing::warn!(path = %directory.display(), %error, "local file operation directory refresh failed");
                    DirectorySnapshot {
                        entries: Vec::new().into(),
                        error: Some(format!("{error:#}")),
                    }
                }
            };
            updates.push((directory, snapshot));
        }
        let previous_statistics = DATA_CONTEXT.project.statistics();
        let statistics = if !previous_statistics.scanned_at.is_empty() {
            Some(
                match INFRASTRUCTURE
                    .filesystem
                    .statistics(DATA_CONTEXT.project_root().to_owned())
                    .await
                {
                    Ok(snapshot) => snapshot,
                    Err(error) => {
                        tracing::warn!(%error, "local file operation statistics refresh failed");
                        let mut snapshot = (*previous_statistics).clone();
                        snapshot.error = Some(format!("{error:#}"));
                        snapshot
                    }
                },
            )
        } else {
            None
        };
        for (directory, snapshot) in updates {
            DATA_CONTEXT
                .project
                .set_directory_silent(directory, snapshot);
        }
        if let Some(statistics) = statistics {
            DATA_CONTEXT.project.set_statistics_silent(statistics);
        }
        DATA_CONTEXT.notify_gui_refresh(&[
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
        ]);
    }

    pub(crate) fn validate_file_name(&self, name: &str) -> Result<String> {
        Ok(validate_name(name)?.to_owned())
    }
    pub(in crate::application::project) async fn file_tree_operation_core(
        &self,
        operation: FileTreeOperation,
    ) -> Result<FileTreeOutcome> {
        let _guard = self.file_operations.lock().await;
        let _refresh = crate::application::APPLICATION
            .file_watch
            .lock_refresh()
            .await;
        self.file_tree_operation_unlocked(operation).await
    }

    pub(in crate::application::project) async fn file_tree_operation_unlocked(
        &self,
        operation: FileTreeOperation,
    ) -> Result<FileTreeOutcome> {
        let root = DATA_CONTEXT.project_root().to_owned();
        let files = &INFRASTRUCTURE.filesystem;
        match operation {
            FileTreeOperation::CreateDirectory { directory, name } => {
                let name = validate_name(&name)?;
                let path = directory.join(name);
                let change = files
                    .begin_gui_file_change(root.clone(), vec![path.clone()])
                    .await?;
                change.finish(files.create_directory(root, path.clone()).await)?;
                self.refresh_local_directories(&[&directory, &path], None)
                    .await;
                tracing::info!(path = %path.display(), "project directory created");
                Ok(FileTreeOutcome::DirectoryCreated(path))
            }
            FileTreeOperation::Create { directory, name } => {
                let name = validate_name(&name)?;
                let path = directory.join(name);
                let change = files
                    .begin_gui_file_change(root.clone(), vec![path.clone()])
                    .await?;
                change.finish(files.create_file(root, path.clone(), Vec::new()).await)?;
                self.refresh_local_directories(&[&directory], None).await;
                tracing::info!(path = %path.display(), "project file created");
                Ok(FileTreeOutcome::Created(path))
            }
            FileTreeOperation::Rename { source, name } => {
                ensure_idle(&source)?;
                let name = validate_name(&name)?;
                let destination = source.parent().unwrap_or(Path::new("")).join(name);
                if source == destination {
                    bail!("名称没有变化");
                }
                let change = files
                    .begin_gui_file_change(root.clone(), vec![source.clone(), destination.clone()])
                    .await?;
                change.finish(
                    files
                        .rename(root, source.clone(), destination.clone())
                        .await,
                )?;
                self.invalidate_moved_run_target(&source, &destination);
                self.refresh_local_directories(
                    &[source.parent().unwrap_or(Path::new(""))],
                    Some((&source, &destination)),
                )
                .await;
                tracing::info!(source = %source.display(), destination = %destination.display(), "project entry renamed");
                Ok(FileTreeOutcome::Renamed {
                    source,
                    destination,
                })
            }
            FileTreeOperation::Delete { source } => {
                ensure_idle(&source)?;
                let change = files
                    .begin_gui_file_change(root.clone(), vec![source.clone()])
                    .await?;
                change.finish(files.remove_entry(root.clone(), source.clone()).await)?;
                for document in DATA_CONTEXT.documents.snapshots() {
                    if document.path.starts_with(root.join(&source)) {
                        DATA_CONTEXT.documents.release_clean_silent(&document.path);
                    }
                }
                let mut directories = vec![source.parent().unwrap_or(Path::new("")).to_owned()];
                directories.extend(
                    DATA_CONTEXT
                        .project
                        .snapshots()
                        .1
                        .keys()
                        .filter(|path| path.starts_with(&source))
                        .cloned(),
                );
                let directories: Vec<_> = directories.iter().map(|path| path.as_path()).collect();
                self.refresh_local_directories(&directories, None).await;
                tracing::info!(path = %source.display(), "project entry deleted");
                Ok(FileTreeOutcome::Deleted(source))
            }
            FileTreeOperation::Reveal { source } => {
                files.reveal_entry(root, source).await?;
                Ok(FileTreeOutcome::Revealed)
            }
            FileTreeOperation::Copy { source } => {
                ensure_idle(&source)?;
                files.metadata(root.clone(), source.clone()).await?;
                Ok(FileTreeOutcome::Copied(ProjectClipboard {
                    kind: "gpui-idea-project-entry".into(),
                    project_root: root,
                    source,
                }))
            }
            FileTreeOperation::Paste {
                clipboard,
                directory,
            } => {
                if clipboard.kind != "gpui-idea-project-entry" || clipboard.project_root != root {
                    bail!("请先复制当前项目中的文件或目录");
                }
                ensure_idle(&clipboard.source)?;
                let destination = files
                    .copy_destination(root.clone(), clipboard.source.clone(), directory.clone())
                    .await?;
                let change = files
                    .begin_gui_file_change(root.clone(), vec![destination.clone()])
                    .await?;
                change.finish(
                    files
                        .copy_entry(root, clipboard.source, destination.clone())
                        .await,
                )?;
                self.refresh_local_directories(&[&directory], None).await;
                tracing::info!(path = %destination.display(), "project entry pasted");
                Ok(FileTreeOutcome::Pasted(destination))
            }
        }
    }
}

pub(super) fn ensure_idle(relative: &Path) -> Result<()> {
    if relative.as_os_str().is_empty() {
        bail!("不能修改或复制项目根目录");
    }
    let absolute = DATA_CONTEXT.project_root().join(relative);
    if DATA_CONTEXT
        .documents
        .snapshots()
        .iter()
        .any(|doc| doc.path.starts_with(&absolute) && (doc.dirty() || doc.saving))
    {
        bail!("相关文件有未保存的编辑或正在保存，请先保存后再操作");
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<&str> {
    let name = name.trim();
    let components: Vec<_> = Path::new(name).components().collect();
    if name.is_empty()
        || name.len() > 255
        || components.len() != 1
        || !matches!(components[0], Component::Normal(_))
        || name.ends_with('.')
        || name
            .chars()
            .any(|ch| ch.is_control() || "<>:\"/\\|?*".contains(ch))
    {
        bail!("请输入有效的文件名，不能包含路径或特殊字符");
    }
    let base = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (base.starts_with("COM") || base.starts_with("LPT"))
            && base.len() == 4
            && matches!(base.as_bytes()[3], b'1'..=b'9')
    {
        bail!("不能使用 Windows 保留名称");
    }
    Ok(name)
}
