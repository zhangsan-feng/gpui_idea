use ropey::Rope;
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Copy, Debug)]
pub enum LineEnding {
    Lf,
    CrLf,
}

#[derive(Clone, Copy, Debug)]
pub struct TextFormat {
    pub bom: bool,
    pub line_ending: LineEnding,
}

#[derive(Clone)]
pub struct DocumentSnapshot {
    pub path: PathBuf,
    pub scope_root: PathBuf,
    pub text: Rope,
    pub revision: u64,
    pub(crate) formatted_revision: Option<u64>,
    pub saved_revision: u64,
    pub saving: bool,
    pub save_error: Option<String>,
    pub disk_bytes: Arc<[u8]>,
    pub format: TextFormat,
}
impl DocumentSnapshot {
    pub fn dirty(&self) -> bool {
        self.revision != self.saved_revision
    }
}
