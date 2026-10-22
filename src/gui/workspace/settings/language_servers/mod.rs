mod core;
mod ui;

pub(super) enum ServerOperationState {
    Busy(&'static str),
    Success(String),
    Error(String),
}
