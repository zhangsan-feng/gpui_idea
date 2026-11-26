use super::core;
use crate::infrastructure::INFRASTRUCTURE;
use anyhow::{Context, Result, ensure};
use std::path::PathBuf;

pub(crate) async fn file_references(document: PathBuf, files: Vec<PathBuf>) -> Result<String> {
    ensure!(files.len() <= 16, "一次最多插入 16 个文件引用");
    let mut references = Vec::new();
    for file in files {
        let parent = file.parent().context("拖入文件路径无效")?;
        let name = file.file_name().context("拖入文件缺少文件名")?;
        let metadata = INFRASTRUCTURE
            .filesystem
            .metadata(parent.to_owned(), name.into())
            .await
            .with_context(|| format!("无法引用 {}", file.display()))?;
        if metadata.is_directory {
            continue;
        }
        references.push(core::file_reference(&document, &file)?);
    }
    ensure!(
        !references.is_empty(),
        "请拖入图片或文件，文件夹不能插入 Markdown"
    );
    Ok(references.join("\n\n"))
}
