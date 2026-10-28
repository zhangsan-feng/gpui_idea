mod core;
mod ui;

pub(in crate::gui) enum ServerOperationState {
    Busy(&'static str),
    Success(String),
    Error(String),
}
