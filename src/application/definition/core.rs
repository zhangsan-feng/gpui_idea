use crate::{
    data_context::DATA_CONTEXT,
    domain::{definition::DefinitionResult, project::ContentMatch},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Context as _, Result};
use lsp_types::Position;
use ropey::{LineType, Rope};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

// Keep the workspace spelling of Windows paths: servers may lowercase drive
// letters or omit verbatim prefixes. This preserves existing tabs and buffers.
fn normal_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let path = path.to_string_lossy();
        if let Some(rest) = path.strip_prefix("\\\\?\\UNC\\") {
            return PathBuf::from(format!("\\\\{rest}"));
        }
        if let Some(rest) = path.strip_prefix("\\\\?\\") {
            return PathBuf::from(rest);
        }
    }
    path.to_owned()
}

fn path_part_eq(a: &std::ffi::OsStr, b: &std::ffi::OsStr) -> bool {
    #[cfg(windows)]
    {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

fn align_target(path: PathBuf, root: &Path, known: &[PathBuf]) -> PathBuf {
    let normalized = normal_path(&path);
    if let Some(known) = known
        .iter()
        .find(|known| path_part_eq(normal_path(known).as_os_str(), normalized.as_os_str()))
    {
        return known.clone();
    }
    let root_normal = normal_path(root);
    let root_parts: Vec<_> = root_normal.components().collect();
    let parts: Vec<_> = normalized.components().collect();
    if parts.len() > root_parts.len()
        && root_parts
            .iter()
            .zip(&parts)
            .all(|(a, b)| path_part_eq(a.as_os_str(), b.as_os_str()))
    {
        return root.join(
            parts[root_parts.len()..]
                .iter()
                .map(|part| part.as_os_str())
                .collect::<PathBuf>(),
        );
    }
    path
}

fn identifier(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

pub(super) async fn resolve(
    path: PathBuf,
    text: Rope,
    offset: usize,
    language: &'static str,
    cancelled: tokio::sync::watch::Receiver<bool>,
) -> Result<DefinitionResult> {
    let language = crate::domain::language_server::language_for_path(&path).unwrap_or(language);
    anyhow::ensure!(
        crate::domain::language_server::family(language).is_some(),
        "尚未接入 {language} 语言服务"
    );
    let started = std::time::Instant::now();
    let snapshots = DATA_CONTEXT.documents.snapshots();
    let known_paths: Vec<_> = snapshots
        .iter()
        .map(|snapshot| snapshot.path.clone())
        .collect();
    let current = path.clone();
    let buffer = text.clone();
    let (symbol, position, source, overlays) = tokio::task::spawn_blocking(move || -> Result<_> {
        anyhow::ensure!(
            offset <= text.len() && text.is_char_boundary(offset),
            "点击位置无效"
        );
        let before: usize = text
            .chars_at(offset)
            .reversed()
            .take_while(|ch| identifier(*ch))
            .map(char::len_utf8)
            .sum();
        let after: usize = text
            .chars_at(offset)
            .take_while(|ch| identifier(*ch))
            .map(char::len_utf8)
            .sum();
        let symbol = text.slice(offset - before..offset + after).to_string();
        anyhow::ensure!(!symbol.is_empty(), "请按住 Ctrl 点击函数或类型名称");
        let line = text.byte_to_line_idx(offset, LineType::LF);
        let start = text.line_to_byte_idx(line, LineType::LF);
        let character = text
            .slice(start..offset)
            .chars()
            .map(char::len_utf16)
            .sum::<usize>();
        let position = Position::new(line as u32, character as u32);
        let overlays = snapshots
            .into_iter()
            .filter(|snapshot| {
                snapshot.path != current
                    && crate::domain::language_server::language_for_path(&snapshot.path)
                        .and_then(crate::domain::language_server::family)
                        == crate::domain::language_server::family(language)
            })
            .map(|snapshot| (snapshot.path.clone(), snapshot.text.to_string()))
            .collect::<Vec<_>>();
        Ok((symbol, position, text.to_string(), overlays))
    })
    .await??;
    let root = DATA_CONTEXT.project_root().to_owned();
    let locations = INFRASTRUCTURE
        .language_servers
        .definitions(
            root.clone(),
            path.clone(),
            source,
            position,
            language,
            overlays,
            cancelled,
        )
        .await?;
    let mut targets = Vec::new();
    let mut buffers = HashMap::from([(path, buffer)]);
    let truncated = locations.len() > 100;
    for location in locations.into_iter().take(100) {
        let uri = url::Url::parse(location.uri.as_str()).context("语言服务返回了无效源码地址")?;
        let target = uri
            .to_file_path()
            .map_err(|_| anyhow::anyhow!("语言服务返回的源码地址暂不支持：{}", uri.scheme()))?;
        let target = align_target(target, &root, &known_paths);
        if !buffers.contains_key(&target) {
            let buffer = if let Some(snapshot) = DATA_CONTEXT.documents.snapshot(&target) {
                snapshot.text.clone()
            } else {
                let parent = target.parent().context("源码地址没有父目录")?.to_owned();
                INFRASTRUCTURE
                    .filesystem
                    .read_document(parent, target.clone())
                    .await
                    .with_context(|| {
                        format!(
                            "无法读取目标源码 {}，请确认对应语言的标准库/依赖源码已安装",
                            target.display()
                        )
                    })?
                    .document
                    .text
            };
            buffers.insert(target.clone(), buffer);
        }
        let buffer = buffers[&target].clone();
        let position = location.range.start;
        let path = target.clone();
        let result = tokio::task::spawn_blocking(move || -> Result<_> {
            anyhow::ensure!(
                (position.line as usize) < buffer.len_lines(LineType::LF),
                "语言服务返回的源码行号超出文件范围"
            );
            let start = buffer.line_to_byte_idx(position.line as usize, LineType::LF);
            let end = if (position.line as usize + 1) < buffer.len_lines(LineType::LF) {
                buffer.line_to_byte_idx(position.line as usize + 1, LineType::LF)
            } else {
                buffer.len()
            };
            let line = buffer.slice(start..end);
            let mut utf16 = 0;
            let mut column = 0;
            for ch in line.chars() {
                if utf16 + ch.len_utf16() > position.character as usize {
                    break;
                }
                utf16 += ch.len_utf16();
                column += 1;
            }
            Ok(ContentMatch {
                path,
                line: position.line + 1,
                column: column + 1,
                preview: line.chars().take(180).collect(),
            })
        })
        .await??;
        if !targets.iter().any(|item: &ContentMatch| {
            item.path == result.path && item.line == result.line && item.column == result.column
        }) {
            targets.push(result);
        }
    }
    tracing::info!(%symbol, count = targets.len(), elapsed_ms = started.elapsed().as_millis(), "LSP definitions resolved");
    Ok(DefinitionResult {
        symbol,
        targets,
        truncated,
    })
}
