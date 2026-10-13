use super::io::BUFFER_SIZE;
use anyhow::{Context, Result};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) struct Journal {
    pub(super) path: PathBuf,
    file: File,
    original_len: u64,
    regions: Vec<(u64, u64, u64)>,
}

impl Journal {
    pub(super) fn create(
        target: &Path,
        original: &mut File,
        original_len: u64,
        regions: Vec<(u64, u64)>,
    ) -> Result<Self> {
        let path = target.parent().context("文件没有父目录")?.join(format!(
            ".gpui-patch-{}-{}.bak",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&path)?;
        let mut this = Self {
            path,
            file,
            original_len,
            regions: Vec::new(),
        };
        let result = (|| -> Result<()> {
            this.file.write_all(b"GPUI-BLOCK-1\0")?;
            this.file.write_all(&original_len.to_le_bytes())?;
            let target = target.to_string_lossy();
            this.file.write_all(&(target.len() as u64).to_le_bytes())?;
            this.file.write_all(target.as_bytes())?;
            this.file.write_all(&(regions.len() as u64).to_le_bytes())?;
            for (offset, count) in regions {
                this.file.write_all(&offset.to_le_bytes())?;
                this.file.write_all(&count.to_le_bytes())?;
                let backup_offset = this.file.stream_position()?;
                original.seek(SeekFrom::Start(offset))?;
                copy_exact(original, &mut this.file, count)?;
                this.regions.push((offset, count, backup_offset));
            }
            this.file.sync_all()?;
            Ok(())
        })();
        if let Err(error) = result {
            this.discard();
            return Err(error).context("创建局部备份失败，文件尚未修改");
        }
        Ok(this)
    }

    pub(super) fn restore(&mut self, original: &mut File) -> Result<()> {
        original.set_len(self.original_len)?;
        for &(offset, count, backup_offset) in &self.regions {
            self.file.seek(SeekFrom::Start(backup_offset))?;
            original.seek(SeekFrom::Start(offset))?;
            copy_exact(&mut self.file, original, count)?;
        }
        original.sync_all()?;
        Ok(())
    }

    pub(super) fn discard(self) {
        drop(self.file);
        if let Err(error) = fs::remove_file(&self.path) {
            tracing::warn!(backup = %self.path.display(), error = %error, "local block backup cleanup failed");
        }
    }
}

fn copy_exact(source: &mut File, destination: &mut File, mut remaining: u64) -> Result<()> {
    let mut buffer = [0u8; BUFFER_SIZE];
    while remaining > 0 {
        let count = remaining.min(BUFFER_SIZE as u64) as usize;
        source.read_exact(&mut buffer[..count])?;
        destination.write_all(&buffer[..count])?;
        remaining -= count as u64;
    }
    Ok(())
}
