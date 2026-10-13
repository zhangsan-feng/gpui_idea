mod core;
mod external;

pub struct HistoryService {
    operations: tokio::sync::Mutex<()>,
}
impl HistoryService {
    pub(super) fn new() -> Self {
        Self {
            operations: tokio::sync::Mutex::new(()),
        }
    }
}
