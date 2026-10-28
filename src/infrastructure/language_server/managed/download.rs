use super::{InstallContext, Installed, update};
use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
};

pub(in crate::infrastructure::language_server) fn download_server(
    server: &str,
    context: &InstallContext,
) -> Result<PathBuf> {
    anyhow::ensure!(cfg!(windows), "{server} 自动下载当前仅支持 Windows");
    let binary = update::ensure(
        server,
        server,
        context,
        || version(server, context),
        |stage, version| {
            let (source, entry) =
                super::super::languages::download_release(server, stage, version, context)?;
            Ok(Installed {
                entry: entry.strip_prefix(stage)?.to_owned(),
                version: version.into(),
                source,
                selection: None,
            })
        },
    )?;
    Ok(binary.path)
}

pub(in crate::infrastructure::language_server) fn architecture() -> Result<&'static str> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("x86_64"),
        "aarch64" => Ok("aarch64"),
        other => anyhow::bail!("不支持的服务器架构：{other}"),
    }
}

fn version(server: &str, context: &InstallContext) -> Result<String> {
    if let Some(version) = &context.requested_release {
        return Ok(version.clone());
    }
    super::super::languages::latest_release(server, context)
}

fn safe_path(path: &Path) -> Result<()> {
    anyhow::ensure!(
        path.components()
            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir)),
        "压缩包包含非法路径：{}",
        path.display()
    );
    Ok(())
}

pub(in crate::infrastructure::language_server) fn copy_limited(
    input: impl Read,
    path: &Path,
    context: &InstallContext,
) -> Result<()> {
    use std::io::Write;
    let mut input = input;
    let mut output = File::create_new(path)?;
    let mut total = 0u64;
    let mut buffer = [0; 65536];
    loop {
        context.check()?;
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        anyhow::ensure!(total <= 1024 * 1024 * 1024, "解压文件超过 1 GiB");
        output.write_all(&buffer[..count])?;
    }
    Ok(())
}

pub(in crate::infrastructure::language_server) fn extract_zip(
    archive: &Path,
    stage: &Path,
    context: &InstallContext,
) -> Result<()> {
    context.progress("正在解压语言服务器…")?;
    let mut archive = zip::ZipArchive::new(File::open(archive)?)?;
    let mut size = 0u64;
    for index in 0..archive.len() {
        context.check()?;
        let mut entry = archive.by_index(index)?;
        let path = entry.enclosed_name().context("ZIP 包含非法路径")?;
        safe_path(&path)?;
        anyhow::ensure!(!entry.is_symlink(), "ZIP 不允许符号链接");
        size = size.checked_add(entry.size()).context("ZIP 大小溢出")?;
        anyhow::ensure!(size <= 1024 * 1024 * 1024, "解压总大小超过 1 GiB");
        let target = stage.join(path);
        if entry.is_dir() {
            fs::create_dir_all(target)?;
        } else {
            fs::create_dir_all(target.parent().context("ZIP 路径无效")?)?;
            copy_limited(&mut entry, &target, context)?;
        }
    }
    Ok(())
}

pub(in crate::infrastructure::language_server) fn extract_tar(
    archive: &Path,
    stage: &Path,
    context: &InstallContext,
) -> Result<()> {
    context.progress("正在解压语言服务器…")?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(File::open(archive)?));
    let mut size = 0u64;
    for entry in archive.entries()? {
        context.check()?;
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        safe_path(&path)?;
        let kind = entry.header().entry_type();
        anyhow::ensure!(kind.is_file() || kind.is_dir(), "TAR 不允许链接或特殊文件");
        size = size.checked_add(entry.size()).context("TAR 大小溢出")?;
        anyhow::ensure!(size <= 1024 * 1024 * 1024, "解压总大小超过 1 GiB");
        let target = stage.join(path);
        if kind.is_dir() {
            fs::create_dir_all(target)?;
        } else {
            fs::create_dir_all(target.parent().context("TAR 路径无效")?)?;
            copy_limited(&mut entry, &target, context)?;
        }
    }
    Ok(())
}
