#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TerminalId {
    Shell,
    Run,
    RunInstance(u64),
    Local(u64),
}

impl TerminalId {
    pub fn is_run(self) -> bool {
        matches!(self, Self::Run | Self::RunInstance(_))
    }
}

#[derive(Clone, Default)]
pub struct TerminalCell {
    pub text: String,
    pub foreground: u32,
    pub background: u32,
    pub wide: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Clone)]
pub struct TerminalSnapshot {
    pub command: Option<String>,
    pub exit_code: Option<i32>,
    pub generation: u64,
    pub cells: Vec<TerminalCell>,
    pub columns: usize,
    pub lines: usize,
    pub history_size: usize,
    pub scroll_offset: usize,
    pub cursor: Option<(usize, usize)>,
    pub title: String,
    pub exited: bool,
    pub application_cursor: bool,
    pub bracketed_paste: bool,
}

impl Default for TerminalSnapshot {
    fn default() -> Self {
        Self {
            command: None,
            exit_code: None,
            generation: 0,
            cells: Vec::new(),
            columns: 100,
            lines: 12,
            history_size: 0,
            scroll_offset: 0,
            cursor: None,
            title: "PowerShell".into(),
            exited: false,
            application_cursor: false,
            bracketed_paste: false,
        }
    }
}
