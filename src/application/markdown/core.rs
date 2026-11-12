use anyhow::{Context, Result};
use std::path::Path;

pub(super) fn file_reference(document: &Path, file: &Path) -> Result<String> {
    let base =
        url::Url::from_file_path(document).map_err(|_| anyhow::anyhow!("Markdown 文档路径无效"))?;
    let target = url::Url::from_file_path(file).map_err(|_| anyhow::anyhow!("拖入文件路径无效"))?;
    let destination = base
        .make_relative(&target)
        .filter(|relative| {
            !relative.is_empty() && base.join(relative).is_ok_and(|resolved| resolved == target)
        })
        .unwrap_or_else(|| target.to_string());
    // Parentheses are legal in file URLs, but delimit Markdown destinations.
    let destination = destination.replace('(', "%28").replace(')', "%29");
    let name = file
        .file_name()
        .context("拖入文件缺少文件名")?
        .to_string_lossy();
    let label = name
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace(['\r', '\n'], " ");
    let prefix = if crate::domain::image_preview::PreviewImageFormat::for_path(file).is_some() {
        "!"
    } else {
        ""
    };
    Ok(format!("{prefix}[{label}]({destination})"))
}
