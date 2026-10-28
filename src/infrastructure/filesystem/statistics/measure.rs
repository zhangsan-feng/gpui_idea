use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

pub(super) fn file(path: &Path) -> Result<(u64, u64, u64)> {
    let file = File::open(path)?;
    let bytes = file.metadata()?.len();
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut line = Vec::new();
    let mut lines = 0;
    let mut non_empty_lines = 0;
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        // Binary files still contribute their type, count and size.
        // Detect them by content rather than a fixed extension list.
        let text = match std::str::from_utf8(&line) {
            Ok(text)
                if !text
                    .chars()
                    .any(|ch| ch.is_control() && !matches!(ch, '\t' | '\r' | '\n' | '\u{c}')) =>
            {
                text
            }
            _ => return Ok((bytes, 0, 0)),
        };
        lines += 1;
        if !text.trim().trim_start_matches('\u{feff}').trim().is_empty() {
            non_empty_lines += 1;
        }
    }
    Ok((bytes, lines, non_empty_lines))
}
