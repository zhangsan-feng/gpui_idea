use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectClipboard {
    pub kind: String,
    pub project_root: PathBuf,
    pub source: PathBuf,
}

pub enum FileTreeOperation {
    CreateDirectory {
        directory: PathBuf,
        name: String,
    },
    Create {
        directory: PathBuf,
        name: String,
    },
    Rename {
        source: PathBuf,
        name: String,
    },
    Delete {
        source: PathBuf,
    },
    Reveal {
        source: PathBuf,
    },
    Copy {
        source: PathBuf,
    },
    Paste {
        clipboard: ProjectClipboard,
        directory: PathBuf,
    },
}

pub enum FileTreeOutcome {
    DirectoryCreated(PathBuf),
    Created(PathBuf),
    Renamed {
        source: PathBuf,
        destination: PathBuf,
    },
    Deleted(PathBuf),
    Revealed,
    Copied(ProjectClipboard),
    Pasted(PathBuf),
}
