use super::{read_file, write_file};
use crate::infrastructure::filesystem::{LineEdit, MAX_TEXT_BYTES, MAX_TEXT_LINES};
use anyhow::{Context, Result, bail};
use std::{ops::Range, path::Path};

pub(in crate::infrastructure::filesystem) fn modify_lines(
    root: &Path,
    relative: &Path,
    expected: &[u8],
    edits: Vec<LineEdit>,
) -> Result<Vec<u8>> {
    let original = read_file(root, relative, MAX_TEXT_BYTES)?;
    if original != expected {
        bail!("文件已发生变化，停止按行修改");
    }
    if edits.is_empty() {
        bail!("没有指定行修改操作");
    }
    if original.contains(&0) {
        bail!("二进制文件不支持按行修改");
    }
    let bom = original.starts_with(&[0xef, 0xbb, 0xbf]);
    let text = std::str::from_utf8(if bom { &original[3..] } else { &original })
        .context("按行修改只支持 UTF-8 文本")?;
    let crlf = text.matches("\r\n").count();
    if crlf != 0 && crlf != text.matches('\n').count() {
        bail!("不支持混合 LF/CRLF 换行");
    }
    let normalized = text.replace("\r\n", "\n");
    if normalized.contains('\r') {
        bail!("不支持单独 CR 换行");
    }
    let final_newline = normalized.ends_with('\n');
    // A final terminator does not create an extra logical line.
    let mut lines: Vec<String> = normalized
        .split_terminator('\n')
        .map(str::to_owned)
        .collect();
    if lines.len() > MAX_TEXT_LINES {
        bail!("文件超过 50000 行");
    }
    let original_count = lines.len();
    let mut changes = Vec::with_capacity(edits.len());
    for edit in edits {
        let (range, replacement) = match edit {
            LineEdit::Insert { before_line, lines } => {
                if before_line == 0 || before_line > original_count + 1 {
                    bail!("插入行号越界");
                }
                (before_line - 1..before_line - 1, lines)
            }
            LineEdit::Replace {
                start_line,
                end_line,
                lines,
            } => (line_range(start_line, end_line, original_count)?, lines),
            LineEdit::Delete {
                start_line,
                end_line,
            } => (
                line_range(start_line, end_line, original_count)?,
                Vec::new(),
            ),
        };
        if replacement
            .iter()
            .any(|line| line.contains(['\r', '\n', '\0']))
        {
            bail!("每个替换项必须是一行文字，不能包含换行符或 NUL");
        }
        changes.push((range, replacement));
    }
    changes.sort_by_key(|(range, _)| (range.start, range.end));
    for pair in changes.windows(2) {
        if pair[0].0.start == pair[1].0.start || pair[0].0.end > pair[1].0.start {
            bail!("行修改范围重叠或插入位置冲突，请合并后重试");
        }
    }
    let final_count = changes
        .iter()
        .fold(original_count as i128, |count, (range, replacement)| {
            count - range.len() as i128 + replacement.len() as i128
        });
    if final_count > MAX_TEXT_LINES as i128 {
        bail!("修改后的文件超过 50000 行");
    }
    // Highest original line first, so earlier changes do not shift later positions.
    for (range, replacement) in changes.into_iter().rev() {
        lines.splice(range, replacement);
    }
    let separator = if crlf == 0 { "\n" } else { "\r\n" };
    let mut modified = lines.join(separator);
    if final_newline && !lines.is_empty() {
        modified.push_str(separator);
    }
    let mut bytes = Vec::with_capacity(modified.len() + 3);
    if bom {
        bytes.extend_from_slice(&[0xef, 0xbb, 0xbf]);
    }
    bytes.extend_from_slice(modified.as_bytes());
    if bytes.len() > MAX_TEXT_BYTES {
        bail!("修改后的文件超过 2 MiB");
    }
    // The common writer rechecks the baseline and applies only the changed block.
    write_file(root, relative, &bytes, &original)?;
    Ok(bytes)
}

fn line_range(start: usize, end: usize, count: usize) -> Result<Range<usize>> {
    if start == 0 || end < start || end > count {
        bail!("行范围无效，行号从 1 开始且不能超出文件");
    }
    Ok(start - 1..end)
}
