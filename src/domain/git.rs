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
pub struct GitSnapshot {
    pub is_repository: bool,
    pub is_detached: bool,
    pub branch: String,
    pub commits: Vec<GitCommit>,
    pub status: String,
    pub error: Option<String>,
}
#[derive(Clone, Default)]
pub struct GitFileChange {
    pub status: String,
    pub path: PathBuf,
}
#[derive(Clone, Default)]
pub struct GitDetails {
    pub hash: String,
    pub files: Vec<GitFileChange>,
    pub error: Option<String>,
}
