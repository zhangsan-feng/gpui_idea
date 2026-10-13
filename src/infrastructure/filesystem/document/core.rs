use super::LoadedFile;
use crate::domain::document::{DocumentSnapshot, LineEnding, TextFormat};
use anyhow::{Context, Result, bail};
use ropey::Rope;
use std::{fs, io::Read, path::Path, sync::Arc};

use super::super::{MAX_TEXT_BYTES as MAX_BYTES, MAX_TEXT_LINES as MAX_LINES};

fn validate(root: &Path, path: &Path) -> Result<()> {
    let relative = path.strip_prefix(root).context("文件不在当前项目目录内")?;
    super::super::core::path::resolve(root, relative, false)?;
    Ok(())
}
pub(super) fn read(root: &Path, path: &Path) -> Result<LoadedFile> {
    validate(root, path)?;
    let mut file = fs::File::open(path).with_context(|| format!("无法打开 {}", path.display()))?;
    if !file.metadata()?.is_file() {
        bail!("请拖入普通文本文件，目录暂不支持拖入打开");
    }
    if file.metadata()?.len() > MAX_BYTES as u64 {
        bail!("首版编辑器暂支持不超过 2 MiB 的文本文件");
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        bail!("文件超过 2 MiB");
    }
    if bytes.contains(&0) {
        bail!("二进制文件不能作为文本编辑");
    }
    let bom = bytes.starts_with(&[0xef, 0xbb, 0xbf]);
    let raw = std::str::from_utf8(if bom { &bytes[3..] } else { &bytes })
        .context("当前只支持 UTF-8 文本")?;
    let crlf = raw.matches("\r\n").count();
    let newlines = raw.matches('\n').count();
    if crlf > 0 && crlf != newlines {
        bail!("文件混用了 LF 和 CRLF，请先统一换行格式");
    }
    let format = TextFormat {
        bom,
        line_ending: if crlf > 0 {
            LineEnding::CrLf
        } else {
            LineEnding::Lf
        },
    };
    let initial_text = raw.replace("\r\n", "\n");
    if initial_text.contains('\r') {
        bail!("当前不支持单独 CR 换行，请先转换为 LF 或 CRLF");
    }
    if initial_text.lines().count() > MAX_LINES {
        bail!("首版编辑器暂支持不超过 50000 行的文件");
    }
    let document = DocumentSnapshot {
        path: path.to_path_buf(),
        scope_root: root.to_path_buf(),
        text: Rope::from_str(&initial_text),
        revision: 0,
        saved_revision: 0,
        saving: false,
        save_error: None,
        disk_bytes: bytes.into(),
        format,
    };
    Ok(LoadedFile {
        document,
        initial_text,
    })
}
pub(super) fn save(root: &Path, document: &DocumentSnapshot) -> Result<Arc<[u8]>> {
    validate(root, &document.path)?;
    let metadata = fs::metadata(&document.path)?;
    if metadata.permissions().readonly() {
        bail!("文件为只读，未写入");
    }
    if fs::read(&document.path)?.as_slice() != document.disk_bytes.as_ref() {
        bail!("文件在磁盘上已被其他程序修改，停止保存以避免覆盖");
    }
    let text = document.text.to_string();
    if text.lines().count() > MAX_LINES {
        bail!("编辑内容超过 50000 行，正文仍保留在内存中");
    }
    let text = match document.format.line_ending {
        LineEnding::Lf => text,
        LineEnding::CrLf => text.replace('\n', "\r\n"),
    };
    let mut bytes = Vec::with_capacity(text.len() + 3);
    if document.format.bom {
        bytes.extend_from_slice(&[0xef, 0xbb, 0xbf]);
    }
    bytes.extend_from_slice(text.as_bytes());
    if bytes.len() > MAX_BYTES {
        bail!("保存内容超过首版 2 MiB 限制，编辑内容仍保留在内存中");
    }
    let relative = document
        .path
        .strip_prefix(root)
        .context("文件不在项目目录内")?;
    super::super::core::write_file(root, relative, &bytes, &document.disk_bytes)?;
    Ok(bytes.into())
}
