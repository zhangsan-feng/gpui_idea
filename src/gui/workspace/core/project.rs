use super::super::{ProjectFile, TreeRow, Workspace};
use crate::application::APPLICATION;
use crate::data_context::DATA_CONTEXT;
use crate::{
    application::lifecycle::run_application,
    domain::project::{DirectorySnapshot, ProjectEntry},
};
use gpui_kit::{Context, Window};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

impl Workspace {
    pub(in crate::gui::workspace) fn move_project_entry(
        &mut self,
        source: PathBuf,
        target_directory: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing || self.close_prompt_open || self.moving_path.is_some() {
            return;
        }
        let Some(name) = source.file_name() else {
            return;
        };
        let destination = target_directory.join(name);
        if source == destination {
            self.notice = Some("文件或目录已经位于目标目录".into());
            cx.notify();
            return;
        }
        if target_directory.starts_with(&source) {
            self.notice = Some("不能将目录移动到自身或子目录".into());
            cx.notify();
            return;
        }
        let root = DATA_CONTEXT.project_root();
        let affected = |path: &Path| path.starts_with(&source);
        let busy = self
            .loading_files
            .iter()
            .any(|file| affected(file.relative_path()))
            || self
                .saving_files
                .iter()
                .any(|file| affected(file.relative_path()))
            || self
                .loading_directories
                .iter()
                .any(|directory| affected(directory))
            || self
                .loading_directories
                .contains(source.parent().unwrap_or(Path::new("")))
            || self.loading_directories.contains(&target_directory);
        let dirty = DATA_CONTEXT
            .documents
            .dirty_documents()
            .iter()
            .any(|document| document.path.strip_prefix(root).is_ok_and(affected));
        if busy || dirty {
            self.notice = Some("相关文件正在加载、保存或有未保存的编辑，请稍后再移动".into());
            cx.notify();
            return;
        }
        tracing::info!(source = %source.display(), target = %target_directory.display(), "project drag move accepted");
        self.moving_path = Some(source.clone());
        cx.notify();

        let project = &APPLICATION.project;
        cx.spawn_in(window, async move |this, cx| {
            let move_source = source.clone();
            let move_target = target_directory.clone();
            let result = run_application(async move {
                project.move_entry(move_source, move_target).await
            })
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.moving_path = None;
                match result {
                    Ok(destination) => {
                        this.apply_project_move(&source, &destination, window, cx);
                        this.notice = Some(format!("已移动到 {}", destination.display()));
                    }
                    Err(error) => {
                        tracing::error!(source = %source.display(), target = %target_directory.display(), error = %error, "project drag move failed");
                        this.notice = Some(format!("移动失败：{error:#}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_project_move(
        &mut self,
        source: &Path,
        destination: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let remap = |path: &Path| {
            path.strip_prefix(source)
                .map(|suffix| destination.join(suffix))
                .unwrap_or_else(|_| path.to_owned())
        };
        let root = DATA_CONTEXT.project_root().to_owned();
        let reopened: Vec<_> = self
            .open_files
            .iter()
            .filter(|file| file.relative_path().starts_with(source))
            .map(|file| ProjectFile(remap(file.relative_path())))
            .collect();
        let old_selected = self.selected_file.clone();
        for file in self
            .editor_order
            .iter()
            .filter(|file| file.relative_path().starts_with(source))
        {
            let old_path = file.path(&root);
            if let Some(editor) = self.editors.get(file) {
                let state = editor.read(cx);
                let cursor = state.cursor_position();
                let scroll = state.scroll_offset();
                let new_path = root.join(remap(file.relative_path()));
                self.pending_file_views.insert(
                    new_path.clone(),
                    crate::domain::history::FileViewState {
                        path: new_path,
                        cursor_line: cursor.line,
                        cursor_character: cursor.character,
                        scroll_x: scroll.x.into(),
                        scroll_y: scroll.y.into(),
                    },
                );
            }
            APPLICATION.documents.release_clean(&old_path);
        }
        self.editors
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.editor_subscriptions
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.autosave_timers
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.load_errors
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.editor_order
            .retain(|file| !file.relative_path().starts_with(source));
        for file in &mut self.open_files {
            file.0 = remap(&file.0);
        }
        let new_selected = ProjectFile(remap(old_selected.relative_path()));
        self.selected_file = new_selected.clone();
        self.expanded_directories = self
            .expanded_directories
            .iter()
            .map(|path| remap(path))
            .collect();
        self.expanded_directories
            .insert(destination.parent().unwrap_or(Path::new("")).to_owned());
        self.expanded_directories.insert(PathBuf::new());
        for file in reopened {
            self.select_file(file, window, cx);
        }
        if self.open_files.contains(&new_selected) {
            self.select_file(new_selected, window, cx);
        }
        let mut refresh = self.expanded_directories.clone();
        refresh.insert(source.parent().unwrap_or(Path::new("")).to_owned());
        refresh.insert(destination.parent().unwrap_or(Path::new("")).to_owned());
        for directory in refresh {
            self.load_directory(directory, cx);
        }
        self.schedule_session_save(cx);
        tracing::info!(source = %source.display(), destination = %destination.display(), reopened_tabs = self.open_files.len(), "project tree updated after move");
    }

    pub(in crate::gui::workspace) fn toggle_directory(
        &mut self,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        if !self.expanded_directories.remove(&path) {
            self.expanded_directories.insert(path.clone());
            let (_, snapshots) = DATA_CONTEXT.project.snapshots();
            if !snapshots
                .get(&path)
                .is_some_and(|snapshot| snapshot.error.is_none())
            {
                self.load_directory(path.clone(), cx);
            }
        }
        tracing::debug!(directory = %path.display(), expanded = self.expanded_directories.contains(&path), "project directory toggled");
        if self.expanded_directories.contains(&path) {
            self.remember_opened(
                crate::domain::history::OpenKind::Directory,
                DATA_CONTEXT.project_root().join(&path),
                cx,
            );
        }
        self.schedule_session_save(cx);
        self.rebuild_tree();
        cx.notify();
    }

    pub(in crate::gui::workspace) fn refresh_project(&mut self, cx: &mut Context<Self>) {
        let paths: Vec<_> = self.expanded_directories.iter().cloned().collect();
        self.load_directory(PathBuf::new(), cx);
        for path in paths {
            self.load_directory(path, cx);
        }
    }

    pub(in crate::gui::workspace) fn load_directory(
        &mut self,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        if !self.loading_directories.insert(path.clone()) {
            return;
        }
        tracing::info!(directory = %path.display(), "project directory load requested");

        let project = &APPLICATION.project;
        let relative = path.clone();
        self.rebuild_tree();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { project.load_directory(relative).await }).await;
            let _ = this.update(cx, |this, cx| {
                this.loading_directories.remove(&path);
                if let Err(error) = result {
                    this.notice = Some(format!("目录读取失败：{error:#}"));
                }
                // Only directory changes rebuild the projection; typing never scans the tree.
                this.rebuild_tree();
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::gui::workspace) fn rebuild_tree(&mut self) {
        let (revision, snapshots) = DATA_CONTEXT.project.snapshots();
        let mut rows = Vec::new();
        if self.expanded_directories.contains(Path::new("")) {
            append_directory(
                Path::new(""),
                1,
                &snapshots,
                &self.expanded_directories,
                &self.loading_directories,
                &mut rows,
            );
        }
        self.tree_rows = rows;
        self.project_revision = revision;
    }
}

fn append_directory(
    path: &Path,
    depth: usize,
    snapshots: &HashMap<PathBuf, Arc<DirectorySnapshot>>,
    expanded: &HashSet<PathBuf>,
    loading: &HashSet<PathBuf>,
    rows: &mut Vec<TreeRow>,
) {
    let message = |text: String, retry: bool| TreeRow::Message {
        directory: path.to_owned(),
        depth,
        text,
        retry,
    };
    let Some(snapshot) = snapshots.get(path) else {
        rows.push(message("正在读取目录…".into(), false));
        return;
    };
    if loading.contains(path) {
        rows.push(message("正在读取目录…".into(), false));
    }
    if let Some(error) = &snapshot.error {
        rows.push(message(format!("读取失败，点击重试：{error}"), true));
        return;
    }
    if snapshot.entries.is_empty() && !loading.contains(path) {
        rows.push(message("空目录".into(), false));
    }
    for entry in snapshot.entries.iter() {
        rows.push(TreeRow::Entry(ProjectEntry::clone(entry), depth));
        if entry.is_directory && !entry.is_link && expanded.contains(&entry.relative_path) {
            append_directory(
                &entry.relative_path,
                depth + 1,
                snapshots,
                expanded,
                loading,
                rows,
            );
        }
    }
}
