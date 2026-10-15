mod core;
mod external;
use crate::domain::document::DocumentSnapshot;
pub use external::{read, save};

const BUFFER_SIZE: usize = 64 * 1024;

struct TextDecoder {
    builder: ropey::RopeBuilder,
    initial_text: String,
    at_start: bool,
    bom: bool,
    pending_cr: bool,
    saw_lf: bool,
    saw_crlf: bool,
    line_breaks: usize,
    has_tail: bool,
}

pub struct LoadedFile {
    pub document: DocumentSnapshot,
    pub initial_text: String,
}
