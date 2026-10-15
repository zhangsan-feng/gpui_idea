#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum LanguageServerStatus {
    #[default]
    Stopped,
    Starting,
    Indexing,
    Warming,
    Ready,
    Warning(String),
    Error(String),
}
