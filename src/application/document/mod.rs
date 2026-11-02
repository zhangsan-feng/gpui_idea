mod core;
mod external;
mod formatting;
mod image_preview;
mod location;
use crate::domain::document::DocumentDataContextSnapshot;
use std::sync::Arc;

pub struct DocumentService;
impl DocumentService {
    pub(super) fn new() -> Self {
        Self
    }
}

pub struct OpenedDocument {
    pub snapshot: Arc<DocumentDataContextSnapshot>,
    pub initial_text: String,
}
