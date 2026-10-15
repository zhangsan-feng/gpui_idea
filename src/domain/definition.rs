use crate::domain::project::ContentMatch;

pub(crate) struct DefinitionResult {
    pub symbol: String,
    pub targets: Vec<ContentMatch>,
    pub truncated: bool,
}
