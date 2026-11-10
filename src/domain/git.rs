use std::path::PathBuf;
#[derive(Clone, Default)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub author: String,
    pub date: String,
    pub references: String,
}
#[derive(Clone, Default)]
pub struct GitDataContextSnapshot {
    pub root: PathBuf,
    pub changes: Vec<GitWorkingChange>,
    pub branches: Vec<String>,
    pub remotes: Vec<String>,
    pub is_repository: bool,
    pub is_detached: bool,
    pub branch: String,
    pub commits: Vec<GitCommit>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Clone, Default)]
pub struct GitWorkingChange {
    pub path: PathBuf,
    pub index: char,
    pub worktree: char,
}

#[derive(Clone)]
pub enum GitAction {
    Ignore {
        repository: PathBuf,
        path: PathBuf,
        directory: bool,
    },
    Commit {
        summary: String,
        description: String,
        paths: Vec<PathBuf>,
    },
    UpdateFromRemote,
    Switch(String),
}

pub(crate) enum GitCreateRequest {
    Branch {
        name: String,
        remote: Option<String>,
    },
}

#[derive(Clone, Default)]
pub struct GitDiffSnapshot {
    pub key: String,
    pub lines: Vec<String>,
    pub error: Option<String>,
}
#[derive(Clone, Default)]
pub struct GitFileChange {
    pub original_path: Option<PathBuf>,
    pub status: String,
    pub path: PathBuf,
}
#[derive(Clone, Default)]
pub struct GitDetailsDataContextSnapshot {
    pub message: String,
    pub hash: String,
    pub files: Vec<GitFileChange>,
    pub error: Option<String>,
}
