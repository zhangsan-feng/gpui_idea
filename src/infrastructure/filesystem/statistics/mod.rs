mod core;
mod external;
mod measure;

use std::path::PathBuf;

enum ScanTask {
    Directory(PathBuf),
    File(PathBuf),
}

#[derive(Default)]
struct DirectoryScan {
    tasks: Vec<ScanTask>,
    skipped: u64,
}

enum ScanResult {
    Directory(DirectoryScan),
    File {
        extension: String,
        bytes: u64,
        lines: u64,
        code: u64,
    },
    Skipped,
}
