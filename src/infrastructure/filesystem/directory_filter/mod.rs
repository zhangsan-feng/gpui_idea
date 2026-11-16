mod core;
mod external;

/// Shared matching rules for native events and project statistics.
#[derive(Default)]
pub(super) struct DirectoryFilter {
    names: Vec<String>,
}
