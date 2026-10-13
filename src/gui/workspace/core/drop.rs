use super::super::{ProjectFile, Workspace};
use crate::data_context::DATA_CONTEXT;
use gpui_kit::{Context, ExternalPaths, Window};
use std::path::{Path, PathBuf};

const MAX_DROP_FILES: usize = 16;

impl Workspace {
    pub(in crate::gui::workspace) fn open_dropped_files(
        &mut self,
        paths: &ExternalPaths,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing || self.close_prompt_open {
            return;
        }
        tracing::info!(
            count = paths.paths().len(),
            "external files dropped into workspace"
        );
        for path in paths.paths().iter().take(MAX_DROP_FILES) {
            let file = ProjectFile(project_relative_or_absolute(
                DATA_CONTEXT.project_root(),
                path,
            ));
            self.select_file(file, window, cx);
        }
        if paths.paths().len() > MAX_DROP_FILES {
            self.notice = Some("一次最多打开 16 个拖入文件，其余文件未打开".into());
        }
        cx.notify();
    }
}

fn project_relative_or_absolute(root: &Path, path: &Path) -> PathBuf {
    let root_parts: Vec<_> = root.components().collect();
    let path_parts: Vec<_> = path.components().collect();
    let in_project = path_parts.len() > root_parts.len()
        && root_parts.iter().zip(&path_parts).all(|(root, path)| {
            #[cfg(windows)]
            {
                root.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&path.as_os_str().to_string_lossy())
            }
            #[cfg(not(windows))]
            {
                root == path
            }
        });
    if in_project {
        path_parts[root_parts.len()..]
            .iter()
            .map(|part| part.as_os_str())
            .collect()
    } else {
        path.to_owned()
    }
}
