use anyhow::Result;
use ropey::Rope;
use std::{ops::Range, path::PathBuf};

pub(crate) async fn resolve(
    path: PathBuf,
    text: Rope,
    selection: Range<usize>,
    cursor: usize,
    cancelled: tokio::sync::watch::Receiver<bool>,
) -> Result<Vec<Range<usize>>> {
    super::core::resolve(path, text, selection, cursor, cancelled).await
}
