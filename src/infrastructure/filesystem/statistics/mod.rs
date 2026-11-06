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
    path: PathBuf,
    tasks: Vec<ScanTask>,
    skipped: u64,
}

enum ScanResult {
    Directory(DirectoryScan),
    File {
        directory: PathBuf,
        binary: bool,
        extension: String,
        bytes: u64,
        lines: u64,
        code: u64,
    },
    Skipped,
}

#[derive(Default)]
struct DirectoryGroups {
    extensions: std::collections::BTreeMap<String, crate::domain::statistics::ExtensionStatistics>,
    total: crate::domain::statistics::ExtensionStatistics,
}
