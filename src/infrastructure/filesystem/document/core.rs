use super::{BUFFER_SIZE, LoadedFile, TextDecoder};
use crate::domain::document::{DocumentDataContextSnapshot, LineEnding, TextFormat};
use anyhow::{Context, Result, bail};
use ropey::RopeBuilder;
use std::{
    fs,
    io::{BufReader, ErrorKind, Read},
    path::Path,
    sync::Arc,
};

use super::super::MAX_TEXT_LINES as MAX_LINES;

fn validate(root: &Path, path: &Path) -> Result<()> {
    let relative = path.strip_prefix(root).context("文件不在当前项目目录内")?;
    super::super::core::path::resolve(root, relative, false)?;
    Ok(())
}

impl TextDecoder {
    fn new() -> Self {
        Self {
            builder: RopeBuilder::new(),
            initial_text: String::new(),
            at_start: true,
            bom: false,
            pending_cr: false,
            lf_count: 0,
            crlf_count: 0,
            line_breaks: 0,
            has_tail: false,
        }
    }

    fn append(&mut self, text: &str) -> Result<()> {
        let mut normalized = String::with_capacity(text.len());
        for ch in text.chars() {
            if self.at_start {
                self.at_start = false;
                if ch == '\u{feff}' {
                    self.bom = true;
                    continue;
                }
            }
            if ch == '\0' {
                bail!("二进制文件不能作为文本编辑");
            }
            if self.pending_cr {
                if ch != '\n' {
                    bail!("当前不支持单独 CR 换行，请先转换为 LF 或 CRLF");
                }
                self.pending_cr = false;
                self.crlf_count += 1;
                normalized.push('\n');
                self.line_breaks += 1;
                self.has_tail = false;
            } else {
                match ch {
                    '\r' => self.pending_cr = true,
                    '\n' => {
                        self.lf_count += 1;
                        normalized.push(ch);
                        self.line_breaks += 1;
                        self.has_tail = false;
                    }
                    _ => {
                        normalized.push(ch);
                        self.has_tail = true;
                    }
                }
            }
            if self.line_breaks + usize::from(self.has_tail) > MAX_LINES {
                bail!("编辑器暂支持不超过 50000 行的文件");
            }
        }
        self.builder.append(&normalized);
        self.initial_text.push_str(&normalized);
        Ok(())
    }
}

pub(super) fn read(root: &Path, path: &Path) -> Result<LoadedFile> {
    validate(root, path)?;
    let file = fs::File::open(path).with_context(|| format!("无法打开 {}", path.display()))?;
    if !file.metadata()?.is_file() {
        bail!("请拖入普通文本文件，目录暂不支持拖入打开");
    }
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut carry = 0;
    let mut bytes = Vec::new();
    let mut decoder = TextDecoder::new();
    let mut chunks = 0;
    loop {
        let count = match reader.read(&mut buffer[carry..]) {
            Ok(count) => count,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error).context("分块读取文档失败"),
        };
        if count == 0 {
            if carry != 0 {
                bail!("当前只支持 UTF-8 文本，文件末尾包含不完整字符");
            }
            break;
        }
        bytes
            .try_reserve(count)
            .context("文档过大，无法分配读取缓冲")?;
        // Carry bytes are already in the baseline; append only newly read bytes.
        bytes.extend_from_slice(&buffer[carry..carry + count]);
        let total = carry + count;
        let valid = match std::str::from_utf8(&buffer[..total]) {
            Ok(_) => total,
            Err(error) if error.error_len().is_none() => error.valid_up_to(),
            Err(error) => return Err(error).context("当前只支持 UTF-8 文本"),
        };
        decoder.append(std::str::from_utf8(&buffer[..valid])?)?;
        carry = total - valid;
        buffer.copy_within(valid..total, 0);
        chunks += 1;
    }
    if decoder.pending_cr {
        bail!("当前不支持单独 CR 换行，请先转换为 LF 或 CRLF");
    }
    let format = TextFormat {
        bom: decoder.bom,
        line_ending: LineEnding::Lf,
    };
    if decoder.lf_count > 0 && decoder.crlf_count > 0 {
        tracing::info!(path = %path.display(), lf = decoder.lf_count,
            crlf = decoder.crlf_count, line_ending = ?format.line_ending,
            "mixed line endings normalized for editing");
    }
    tracing::debug!(path = %path.display(), bytes = bytes.len(), chunks,
        buffer_bytes = BUFFER_SIZE, "document loaded with buffered chunks");
    let text = decoder.builder.finish();
    let document = DocumentDataContextSnapshot {
        formatted_revision: None,
        path: path.to_path_buf(),
        scope_root: root.to_path_buf(),
        saved_text: text.clone(),
        text,
        revision: 0,
        saved_revision: 0,
        saving: false,
        save_error: None,
        disk_bytes: bytes.into(),
        format,
    };
    Ok(LoadedFile {
        document,
        initial_text: decoder.initial_text,
    })
}

pub(super) fn save(root: &Path, document: &DocumentDataContextSnapshot) -> Result<Arc<[u8]>> {
    validate(root, &document.path)?;
    let mut bytes = Vec::new();
    if document.format.bom {
        bytes.extend_from_slice(&[0xef, 0xbb, 0xbf]);
    }
    let mut line_breaks = 0;
    // Encode Rope chunks without allocating a second whole-document String.
    for chunk in document.text.chunks() {
        if chunk.is_empty() {
            continue;
        }
        let newlines = chunk.bytes().filter(|byte| *byte == b'\n').count();
        line_breaks += newlines;
        let has_tail = !chunk.ends_with('\n');
        if line_breaks + usize::from(has_tail) > MAX_LINES {
            bail!("编辑内容超过 50000 行，正文仍保留在内存中");
        }
        bytes
            .try_reserve(chunk.len())
            .context("文档过大，无法分配保存缓冲")?;
        // Always write LF, including carriage returns introduced by pasted text.
        bytes.extend(chunk.bytes().filter(|byte| *byte != b'\r'));
    }
    let relative = document
        .path
        .strip_prefix(root)
        .context("文件不在项目目录内")?;
    // The common block writer rechecks the entire disk baseline in 64 KiB blocks
    // under its write handle, then patches changed regions with rollback support.
    super::super::core::write_file(root, relative, &bytes, &document.disk_bytes)?;
    tracing::debug!(path = %document.path.display(), bytes = bytes.len(),
        "document saved from rope chunks");
    Ok(bytes.into())
}
