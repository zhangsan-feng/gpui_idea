mod core;
mod external;

use crate::domain::{
    document::DocumentSnapshot, project::DirectorySnapshot, statistics::StatisticsSnapshot,
};
use std::{path::PathBuf, sync::Arc};

#[derive(Default)]
pub(crate) struct FileRefreshBatch {
    pub(crate) documents: Vec<(Arc<DocumentSnapshot>, Result<DocumentSnapshot, String>)>,
    pub(crate) directories: Vec<(PathBuf, DirectorySnapshot)>,
    pub(crate) statistics: Option<StatisticsSnapshot>,
    pub(crate) image_updated: bool,
}
