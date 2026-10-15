mod core;
mod directory;
mod document;
mod external;
mod watcher;
pub(crate) use watcher::FileSubscription;

use std::{path::PathBuf, time::SystemTime};

#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub relative_path: PathBuf,
    pub is_directory: bool,
    pub bytes: u64,
    pub readonly: bool,
    pub modified: Option<SystemTime>,
}

/// All positions refer to the original file, using one-based line numbers.
/// Each string is a single line without CR/LF. Conflicting ranges are rejected.
#[derive(Clone, Debug)]
pub enum LineEdit {
    /// `before_line = line_count + 1` appends; an empty file accepts position 1.
    Insert {
        before_line: usize,
        lines: Vec<String>,
    },
    /// Both endpoints are inclusive.
    Replace {
        start_line: usize,
        end_line: usize,
        lines: Vec<String>,
    },
    Delete {
        start_line: usize,
        end_line: usize,
    },
}

const MAX_TEXT_BYTES: usize = 2 * 1024 * 1024;

/// Offset in the original file. Empty expected bytes insert; empty replacement deletes.
#[derive(Clone, Debug)]
pub struct FileBlockEdit {
    pub offset: u64,
    pub expected: Vec<u8>,
    pub replacement: Vec<u8>,
}
const MAX_TEXT_LINES: usize = 50_000;

pub(crate) struct FileSystem;

impl FileSystem {
    pub(super) fn new() -> Self {
        Self
    }
}
