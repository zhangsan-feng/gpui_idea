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
    pub fn average_bytes(&self) -> f64 {
        self.bytes as f64 / self.count.max(1) as f64
    }
    pub fn average_lines(&self) -> f64 {
        self.lines as f64 / self.count.max(1) as f64
    }
}
#[derive(Clone, Default)]
pub struct StatisticsSnapshot {
    pub extensions: Vec<ExtensionStatistics>,
    pub total: ExtensionStatistics,
    pub skipped_files: u64,
    pub scanned_at: String,
    pub error: Option<String>,
}
