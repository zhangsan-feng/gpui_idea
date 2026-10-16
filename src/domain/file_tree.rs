use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(crate) fn selection_roots(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort_by(|left, right| {
        left.components()
            .count()
            .cmp(&right.components().count())
            .then(left.cmp(right))
    });
    let mut roots: Vec<PathBuf> = Vec::new();
    for path in paths {
        if !roots.iter().any(|parent| path.starts_with(parent)) {
            roots.push(path);
        }
    }
    roots
}

pub(crate) struct FileTreeBatchOutcome {
    pub(crate) completed: Vec<FileTreeOutcome>,
    pub(crate) failure: Option<String>,
}

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
