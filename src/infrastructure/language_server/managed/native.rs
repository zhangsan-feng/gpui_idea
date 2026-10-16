use super::{InstallContext, download};
use anyhow::Result;
use std::path::PathBuf;

pub(in crate::infrastructure::language_server) fn rust_server(
    context: &InstallContext,
) -> Result<PathBuf> {
    download::download_server("rust-analyzer", context)
}
