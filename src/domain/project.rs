use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectEntry {
    pub relative_path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    // Links are displayed without following them, avoiding loops and root escapes.
    pub is_link: bool,
    pub runnable: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DirectorySnapshot {
    pub entries: Arc<[ProjectEntry]>,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ContentMatch {
    pub path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub preview: String,
}

#[derive(Clone, Default)]
pub struct ContentSearchSnapshot {
    pub request: u64,
    pub matches: Vec<ContentMatch>,
    pub truncated: bool,
    pub error: Option<String>,
}
