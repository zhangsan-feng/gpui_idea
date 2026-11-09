use anyhow::{Context as _, Result, ensure};
use serde_json::Value;

/// Strip JSONC comments/trailing commas without touching strings or offsets.
pub(super) fn parse(bytes: &[u8]) -> Result<Value> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let mut clean = bytes.to_vec();
    let mut i = 0;
    let mut quoted = false;
    while i < clean.len() {
        if quoted {
            if clean[i] == b'\\' {
                i += 2;
                continue;
            }
            if clean[i] == b'"' {
                quoted = false;
            }
        } else if clean[i] == b'"' {
            quoted = true;
        } else if clean.get(i..i + 2) == Some(b"//") {
            while i < clean.len() && clean[i] != b'\n' {
                clean[i] = b' ';
                i += 1;
            }
            continue;
        } else if clean.get(i..i + 2) == Some(b"/*") {
            clean[i] = b' ';
            clean[i + 1] = b' ';
            i += 2;
            while i + 1 < clean.len() && &clean[i..i + 2] != b"*/" {
                if !matches!(clean[i], b'\r' | b'\n') {
                    clean[i] = b' ';
                }
                i += 1;
            }
            ensure!(i + 1 < clean.len(), "主题 JSON 注释未闭合");
            clean[i] = b' ';
            clean[i + 1] = b' ';
            i += 2;
            continue;
        }
        i += 1;
    }
    quoted = false;
    i = 0;
    while i < clean.len() {
        if quoted {
            if clean[i] == b'\\' {
                i += 2;
                continue;
            }
            if clean[i] == b'"' {
                quoted = false;
            }
        } else if clean[i] == b'"' {
            quoted = true;
        } else if clean[i] == b',' {
            let next = clean[i + 1..]
                .iter()
                .find(|byte| !byte.is_ascii_whitespace());
            if matches!(next, Some(b'}' | b']')) {
                clean[i] = b' ';
            }
        }
        i += 1;
    }
    serde_json::from_slice(&clean).context("主题 JSON 格式无效")
}
