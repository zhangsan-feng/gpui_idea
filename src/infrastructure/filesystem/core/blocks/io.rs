use crate::infrastructure::filesystem::FileBlockEdit;
use anyhow::Result;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

pub(super) const BUFFER_SIZE: usize = 64 * 1024;

pub(super) fn matches(file: &mut File, offset: u64, expected: &[u8]) -> Result<bool> {
    file.seek(SeekFrom::Start(offset))?;
    let mut buffer = [0u8; BUFFER_SIZE];
    for chunk in expected.chunks(BUFFER_SIZE) {
        file.read_exact(&mut buffer[..chunk.len()])?;
        if &buffer[..chunk.len()] != chunk {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn replace(file: &mut File, edit: &FileBlockEdit) -> Result<()> {
    let old_len = file.metadata()?.len();
    let removed = edit.expected.len() as u64;
    let inserted = edit.replacement.len() as u64;
    let tail_start = edit.offset + removed;
    let mut buffer = [0u8; BUFFER_SIZE];
    if inserted > removed {
        let growth = inserted - removed;
        let new_len = old_len
            .checked_add(growth)
            .ok_or_else(|| anyhow::anyhow!("文件长度溢出"))?;
        file.set_len(new_len)?;
        let mut end = old_len;
        while end > tail_start {
            let start = tail_start.max(end.saturating_sub(BUFFER_SIZE as u64));
            let count = (end - start) as usize;
            move_chunk(file, &mut buffer[..count], start, start + growth)?;
            end = start;
        }
    } else if inserted < removed {
        let shrink = removed - inserted;
        let mut start = tail_start;
        while start < old_len {
            let count = (old_len - start).min(BUFFER_SIZE as u64) as usize;
            move_chunk(file, &mut buffer[..count], start, start - shrink)?;
            start += count as u64;
        }
        file.set_len(old_len - shrink)?;
    }
    // Equal-length modifications only write the requested block.
    file.seek(SeekFrom::Start(edit.offset))?;
    file.write_all(&edit.replacement)?;
    Ok(())
}

fn move_chunk(file: &mut File, buffer: &mut [u8], source: u64, destination: u64) -> Result<()> {
    file.seek(SeekFrom::Start(source))?;
    file.read_exact(buffer)?;
    file.seek(SeekFrom::Start(destination))?;
    file.write_all(buffer)?;
    Ok(())
}
