pub(super) mod code_formatting;
mod core;
pub(super) mod document;
mod external;
pub(crate) mod highlights;
pub(crate) mod markdown;

use std::ops::Range;

#[cfg(windows)]
pub(crate) use external::clipboard_text_with_normalized_breaks;
pub(crate) use external::{outdent_edit, web_link_at_offset, web_link_spans};

pub(crate) const DOCUMENT_TAB_SIZE: usize = 4;

pub(crate) struct OutdentEdit {
    pub(crate) range: Range<usize>,
    pub(crate) replacement: String,
    pub(crate) selection: Range<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkScope {
    Line(usize),
    Markdown,
}
