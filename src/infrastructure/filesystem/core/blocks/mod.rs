mod io;
mod journal;

use super::path::resolve;
use crate::infrastructure::filesystem::FileBlockEdit;
use anyhow::{Context, Result, bail};
use std::{fs::OpenOptions, path::Path};

pub(in crate::infrastructure::filesystem) fn modify_blocks(
    root: &Path,
    relative: &Path,
    edits: Vec<FileBlockEdit>,
) -> Result<()> {
    apply(root, relative, edits, None)
}

pub(in crate::infrastructure::filesystem) fn write_changed_block(
    root: &Path,
    relative: &Path,
    contents: &[u8],
    expected: &[u8],
) -> Result<()> {
    let prefix = expected
        .iter()
        .zip(contents)
        .take_while(|(old, new)| old == new)
        .count();
    let suffix = expected[prefix..]
        .iter()
        .rev()
        .zip(contents[prefix..].iter().rev())
        .take_while(|(old, new)| old == new)
        .count();
    let edit = FileBlockEdit {
        offset: prefix as u64,
        expected: expected[prefix..expected.len() - suffix].to_vec(),
        replacement: contents[prefix..contents.len() - suffix].to_vec(),
    };
    apply(root, relative, vec![edit], Some(expected))
}

fn apply(
    root: &Path,
    relative: &Path,
    mut edits: Vec<FileBlockEdit>,
    baseline: Option<&[u8]>,
) -> Result<()> {
    let path = resolve(root, relative, false)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1); // Permit readers, deny concurrent writes and deletion.
    }
    let mut file = options
        .open(&path)
        .context("无法取得文件写入句柄，文件可能只读或正在被其他程序写入")?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.permissions().readonly() {
        bail!("目标不是可写普通文件");
    }
    let original_len = metadata.len();
    if let Some(baseline) = baseline {
        if baseline.len() as u64 != original_len || !io::matches(&mut file, 0, baseline)? {
            bail!("文件已发生变化，停止修改");
        }
    }
    edits.sort_by_key(|edit| edit.offset);
    let mut previous: Option<(u64, u64)> = None;
    let mut final_len = original_len as i128;
    for edit in &edits {
        let end = edit
            .offset
            .checked_add(edit.expected.len() as u64)
            .context("块范围溢出")?;
        if end > original_len {
            bail!("内容块超出文件范围");
        }
        if let Some((offset, previous_end)) = previous {
            if edit.offset == offset || edit.offset < previous_end {
                bail!("内容块重叠或插入位置冲突");
            }
        }
        if !io::matches(&mut file, edit.offset, &edit.expected)? {
            bail!("目标内容块已变化，停止修改");
        }
        final_len += edit.replacement.len() as i128 - edit.expected.len() as i128;
        previous = Some((edit.offset, end));
    }
    if !(0..=u64::MAX as i128).contains(&final_len) {
        bail!("修改后的文件长度无效");
    }
    edits.retain(|edit| edit.expected != edit.replacement);
    if edits.is_empty() {
        return Ok(());
    }
    let equal_lengths = edits
        .iter()
        .all(|edit| edit.expected.len() == edit.replacement.len());
    let regions = if equal_lengths {
        edits
            .iter()
            .map(|edit| (edit.offset, edit.expected.len() as u64))
            .collect()
    } else {
        vec![(edits[0].offset, original_len - edits[0].offset)]
    };
    let mut backup = journal::Journal::create(&path, &mut file, original_len, regions)?;
    tracing::debug!(path = %path.display(), blocks = edits.len(), equal_lengths, backup = %backup.path.display(), "file block modification starting");
    let result = (|| -> Result<()> {
        for edit in edits.iter().rev() {
            io::replace(&mut file, edit)?;
        }
        file.sync_all()?;
        Ok(())
    })();
    if let Err(error) = result {
        if let Err(rollback) = backup.restore(&mut file) {
            tracing::error!(path = %path.display(), backup = %backup.path.display(), error = %error, rollback = %rollback, "file block rollback failed; recovery journal retained");
            bail!(
                "块修改失败：{error:#}；回滚失败：{rollback:#}；恢复备份保留在 {}",
                backup.path.display()
            );
        }
        backup.discard();
        return Err(error).context("块修改失败，已恢复原文件");
    }
    backup.discard();
    tracing::debug!(path = %path.display(), blocks = edits.len(), original_len, final_len = final_len as u64, "file blocks modified without replacing the file");
    Ok(())
}
