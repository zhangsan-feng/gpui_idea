use crate::application::APPLICATION;
use crate::data_context::DATA_CONTEXT;
use crate::gui::workspace::FileTreeMenuAction;
use crate::gui::workspace::TreeRow;
use crate::gui::workspace::file_tree::FileTree;
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

impl FileTree {
    pub(in crate::gui::workspace) fn toggle_directory(
        &mut self,
        path: PathBuf,
        cx: &mut Context<FileTree>,
    ) {
        if !self.state.expanded_directories.remove(&path) {
            self.state.expanded_directories.insert(path.clone());
            let (_, snapshots) = DATA_CONTEXT.project.snapshots();
            if !snapshots
                .get(&path)
                .is_some_and(|snapshot| snapshot.error.is_none())
            {
                self.load_directory(path.clone(), cx);
            }
        }
        tracing::debug!(directory = %path.display(), expanded = self.state.expanded_directories.contains(&path), "project directory toggled");
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
        self.rebuild_tree();
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
    }

    pub(in crate::gui::workspace) fn refresh_project(&mut self, cx: &mut Context<FileTree>) {
        let paths: Vec<_> = self.state.expanded_directories.iter().cloned().collect();
        self.load_directory(PathBuf::new(), cx);
        for path in paths {
            self.load_directory(path, cx);
        }
    }

    pub(in crate::gui::workspace) fn load_directory(
        &mut self,
        path: PathBuf,
        cx: &mut Context<FileTree>,
    ) {
        if !self.state.loading_directories.insert(path.clone()) {
            return;
        }
        tracing::info!(directory = %path.display(), "project directory load requested");

        let relative = path.clone();
        self.rebuild_tree();
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.project.load_directory(relative).await })
                    .await;
            let _ = this.update(cx, |this, _cx| {
                {
                    this.state.loading_directories.remove(&path);
                    if let Err(error) = result {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                                "目录读取失败：{error:#}"
                            ))),
                        ));
                    }
                    // Only directory changes rebuild the projection; typing never scans the tree.
                    this.rebuild_tree();
                    crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                        crate::event_bus::refresh_gui::GuiComponent::FileTree,
                        crate::event_bus::refresh_gui::GuiComponent::Editor,
                        crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                        crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                    ]);
                }
            });
        })
        .detach();
    }

    pub(in crate::gui::workspace) fn rebuild_tree(&mut self) {
        let (revision, snapshots) = DATA_CONTEXT.project.snapshots();
        if !self.state.file_operation_running {
            self.state.project_selection.retain(|path| {
                !snapshots
                    .get(path.parent().unwrap_or(Path::new("")))
                    .is_some_and(|snapshot| {
                        snapshot.error.is_none()
                            && !snapshot
                                .entries
                                .iter()
                                .any(|entry| entry.relative_path == *path)
                    })
            });
        }
        let mut rows = Vec::new();
        if self.state.expanded_directories.contains(Path::new("")) {
            append_directory(
                Path::new(""),
                1,
                &snapshots,
                &self.state.expanded_directories,
                &self.state.loading_directories,
                &mut rows,
            );
        }
        if let Some(edit) = &self.state.inline_file_edit {
            if matches!(
                edit.action,
                FileTreeMenuAction::NewFile | FileTreeMenuAction::NewDirectory
            ) {
                rows.retain(|row| !matches!(row, TreeRow::Message { directory, text, .. } if directory == &edit.directory && text == "空目录"));
                let position = if edit.directory.as_os_str().is_empty() {
                    Some((0, 1))
                } else {
                    rows.iter().enumerate().find_map(|(index, row)| match row {
                        TreeRow::Entry(entry, depth) if entry.relative_path == edit.directory => {
                            Some((index + 1, depth + 1))
                        }
                        _ => None,
                    })
                };
                if let Some((index, depth)) = position {
                    rows.insert(index, TreeRow::InlineFile(depth));
                }
            }
        }
        self.state.tree_rows = rows;
        self.state.project_revision = revision;
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

impl FileTree {
    pub(in crate::gui::workspace) fn apply_project_move(
        &mut self,
        source: &Path,
        destination: &Path,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let remap = |path: &Path| {
            path.strip_prefix(source)
                .map(|suffix| destination.join(suffix))
                .unwrap_or_else(|_| path.to_owned())
        };
        self.state.project_selection = self
            .state
            .project_selection
            .iter()
            .map(|path| remap(path))
            .collect();
        self.state.expanded_directories = self
            .state
            .expanded_directories
            .iter()
            .map(|path| remap(path))
            .collect();
        self.state
            .expanded_directories
            .insert(destination.parent().unwrap_or(Path::new("")).to_owned());
        self.state.expanded_directories.insert(PathBuf::new());
        self.send(crate::event_bus::gui_event::GuiEvent::Editor(
            crate::gui::workspace::editor::Event::RemapFiles(
                source.to_owned(),
                destination.to_owned(),
            ),
        ));
    }
}
