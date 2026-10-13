mod core;
mod external;
use crate::domain::document::DocumentSnapshot;
use std::sync::Arc;

pub struct DocumentService;
impl DocumentService {
    pub(super) fn new() -> Self {
        Self
    }
}

pub struct OpenedDocument {
    pub snapshot: Arc<DocumentSnapshot>,
    pub initial_text: String,
}
