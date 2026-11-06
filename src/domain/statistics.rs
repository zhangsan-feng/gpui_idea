#[derive(Clone, Debug, Default)]
pub struct ExtensionStatistics {
    pub extension: String,
    pub count: u64,
    pub bytes: u64,
    pub min_bytes: u64,
    pub max_bytes: u64,
    pub lines: u64,
    pub min_lines: u64,
    pub max_lines: u64,
    pub code_lines: u64,
    pub binary_files: u64,
}
impl ExtensionStatistics {
    pub fn add(&mut self, bytes: u64, lines: u64, code: u64) {
        if self.count == 0 {
            self.min_bytes = bytes;
            self.min_lines = lines;
        }
        self.count += 1;
        self.bytes += bytes;
        self.lines += lines;
        self.code_lines += code;
        self.min_bytes = self.min_bytes.min(bytes);
        self.max_bytes = self.max_bytes.max(bytes);
        self.min_lines = self.min_lines.min(lines);
        self.max_lines = self.max_lines.max(lines);
    }
}
#[derive(Clone, Debug, Default)]
pub struct DirectoryStatistics {
    /// A first-level folder relative to the project root.
    pub path: std::path::PathBuf,
    pub extensions: Vec<ExtensionStatistics>,
    /// Includes files in all descendant directories.
    pub total: ExtensionStatistics,
}

#[derive(Clone, Default)]
pub struct StatisticsDataContextSnapshot {
    pub directories: Vec<DirectoryStatistics>,
    pub total: ExtensionStatistics,
    pub skipped_files: u64,
    pub scanned_at: String,
    pub error: Option<String>,
}
