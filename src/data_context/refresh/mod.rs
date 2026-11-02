mod core;
mod external;

use crate::domain::{
    document::DocumentDataContextSnapshot, project::DirectoryDataContextSnapshot,
    statistics::StatisticsDataContextSnapshot,
};
use std::{path::PathBuf, sync::Arc};

#[derive(Default)]
pub(crate) struct FileRefreshBatch {
    pub(crate) documents: Vec<(
        Arc<DocumentDataContextSnapshot>,
        Result<DocumentDataContextSnapshot, String>,
    )>,
    pub(crate) directories: Vec<(PathBuf, DirectoryDataContextSnapshot)>,
    pub(crate) statistics: Option<StatisticsDataContextSnapshot>,
    pub(crate) image_updated: bool,
}
