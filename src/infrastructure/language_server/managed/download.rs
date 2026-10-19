use super::{InstallContext, Installed, compatible, http, update};
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
            let (source, entry) = match server {
                "rust-analyzer" => {
                    let arch = architecture()?;
                    let source = format!(
                        "https://github.com/rust-lang/rust-analyzer/releases/download/{version}/rust-analyzer-{arch}-pc-windows-msvc.zip"
                    );
                    let archive = stage.join("release.zip");
                    http::download(&source, &archive, context)?;
                    extract_zip(&archive, stage, context)?;
                    fs::remove_file(archive)?;
                    (source, stage.join("rust-analyzer.exe"))
                }
                "taplo" => {
                    let arch = architecture()?;
                    let source = format!(
                        "https://github.com/tamasfe/taplo/releases/download/{version}/taplo-windows-{arch}.gz"
                    );
                    let archive = stage.join("release.gz");
                    http::download(&source, &archive, context)?;
                    let entry = stage.join("taplo.exe");
                    copy_limited(
                        flate2::read::GzDecoder::new(File::open(&archive)?),
                        &entry,
                        context,
                    )?;
                    fs::remove_file(archive)?;
                    (source, entry)
                }
                "marksman" => {
                    anyhow::ensure!(
                        std::env::consts::ARCH == "x86_64",
                        "Marksman Windows 自动安装当前仅支持 x64"
                    );
                    let source = format!(
                        "https://github.com/artempyanykh/marksman/releases/download/{version}/marksman.exe"
                    );
                    let entry = stage.join("marksman.exe");
                    http::download(&source, &entry, context)?;
                    (source, entry)
                }
                "jdtls" => {
                    let base = format!("https://download.eclipse.org/jdtls/milestones/{version}/");
                    let listing = http::text(&base)?;
                    let pattern = regex::Regex::new(r"jdt-language-server-[\w.\-]+\.tar\.gz")?;
                    let name = pattern
                        .find(&listing)
                        .context("Eclipse 发行目录缺少 JDT LS 压缩包")?
                        .as_str();
                    let source = format!("{base}{name}");
                    let archive = stage.join("release.tar.gz");
                    http::download(&source, &archive, context)?;
                    extract_tar(&archive, stage, context)?;
                    fs::remove_file(archive)?;
                    anyhow::ensure!(
                        stage.join("config_win/config.ini").is_file(),
                        "JDT LS 缺少 Windows 配置"
                    );
                    let entry = fs::read_dir(stage.join("plugins"))?
                        .filter_map(Result::ok)
                        .map(|entry| entry.path())
                        .find(|path| {
                            path.file_name().is_some_and(|name| {
                                let name = name.to_string_lossy();
                                name.starts_with("org.eclipse.equinox.launcher_")
                                    && name.ends_with(".jar")
                            })
                        })
                        .context("JDT LS 缺少 launcher jar")?;
                    let (_, current) = super::java_runtime()?;
                    let required = super::java_requirement(stage)?;
                    anyhow::ensure!(
                        current >= required,
                        "JDT LS {version} 要求 JDK {required}+，当前 Java {current}；请升级 JDK 后重试"
                    );
                    (source, entry)
                }
                _ => anyhow::bail!("未知服务器：{server}"),
            };
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

fn architecture() -> Result<&'static str> {
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
    match server {
        "rust-analyzer" => http::latest("rust-lang/rust-analyzer"),
        "taplo" => http::latest("tamasfe/taplo"),
        "marksman" => http::latest("artempyanykh/marksman"),
        "jdtls" => {
            let listing = http::text("https://download.eclipse.org/jdtls/milestones/")?;
            let pattern = regex::Regex::new(r#"href=['"](?:[^'"?#]*/)?(\d+\.\d+\.\d+)/?['"]"#)?;
            let mut versions = pattern
                .captures_iter(&listing)
                .filter_map(|capture| {
                    let version = capture[1].to_owned();
                    let parts = version
                        .split('.')
                        .map(str::parse::<u32>)
                        .collect::<std::result::Result<Vec<_>, _>>()
                        .ok()?;
                    Some((parts, version))
                })
                .collect::<Vec<_>>();
            versions.sort_by(|a, b| a.0.cmp(&b.0));
            versions.dedup_by(|a, b| a.0 == b.0);
            compatible::java(
                &versions
                    .into_iter()
                    .map(|(_, version)| version)
                    .collect::<Vec<_>>(),
                super::java_runtime()?.1,
                context,
            )
        }
        _ => anyhow::bail!("未知服务器：{server}"),
    }
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

fn copy_limited(input: impl Read, path: &Path, context: &InstallContext) -> Result<()> {
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

fn extract_zip(archive: &Path, stage: &Path, context: &InstallContext) -> Result<()> {
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

fn extract_tar(archive: &Path, stage: &Path, context: &InstallContext) -> Result<()> {
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
