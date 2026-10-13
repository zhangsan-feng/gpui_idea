mod core;
mod external;
use crate::domain::document::DocumentSnapshot;
pub use external::{read, save};

pub struct LoadedFile {
    pub document: DocumentSnapshot,
    pub initial_text: String,
}
