use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug)]
pub struct ProjectEntry {
    pub relative_path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    // Links are displayed without following them, avoiding loops and root escapes.
    pub is_link: bool,
}

#[derive(Clone)]
pub struct DirectorySnapshot {
    pub entries: Arc<[ProjectEntry]>,
    pub error: Option<String>,
}
