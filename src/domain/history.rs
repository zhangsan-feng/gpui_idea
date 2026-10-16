use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(crate) fn project_identity(path: &std::path::Path) -> String {
    let path = path.to_string_lossy();
    #[cfg(windows)]
    {
        let path = if let Some(network) = path.strip_prefix("\\\\?\\UNC\\") {
            format!("\\\\{network}")
        } else {
            path.trim_start_matches("\\\\?\\").to_owned()
        };
        path.to_lowercase()
    }
    #[cfg(not(windows))]
    {
        path.into_owned()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenKind {
    Project,
    Directory,
    File,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolWindow {
    #[default]
    Run,
    Statistics,
    Git,
    Terminal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenHistoryEntry {
    pub kind: OpenKind,
    pub path: PathBuf,
    pub opened_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceSession {
    pub project_root: PathBuf,
    pub open_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub expanded_directories: Vec<PathBuf>,
    pub project_panel_open: bool,
    pub run_panel_open: bool,
    #[serde(default)]
    pub active_tool: ToolWindow,
    #[serde(default)]
    pub run_command: String,
    #[serde(default = "default_run_automatic")]
    pub run_automatic: bool,
    #[serde(default)]
    pub run_profile: String,
    #[serde(default)]
    pub run_configurations: Vec<crate::domain::run::SavedRunConfiguration>,
    #[serde(default)]
    pub selected_run_configuration: u64,
    #[serde(default)]
    pub project_width: Option<f32>,
    #[serde(default)]
    pub run_height: Option<f32>,
    #[serde(default)]
    pub file_views: Vec<FileViewState>,
    #[serde(default)]
    pub window_view: Option<WindowViewState>,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowViewState {
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}
fn default_run_automatic() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileViewState {
    pub path: PathBuf,
    pub cursor_line: u32,
    pub cursor_character: u32,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenHistory {
    pub version: u32,
    pub entries: Vec<OpenHistoryEntry>,
    pub last_project: Option<PathBuf>,
    #[serde(default)]
    pub sessions: Vec<WorkspaceSession>,
}

impl Default for OpenHistory {
    fn default() -> Self {
        Self {
            version: 2,
            entries: Vec::new(),
            last_project: None,
            sessions: Vec::new(),
        }
    }
}
