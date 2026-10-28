use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::{Context, Result};
use managed::compatible::numeric;
use managed::{download::extract_tar, http};
use std::path::Path;
use std::{fs, path::PathBuf};

pub(in crate::infrastructure::language_server) fn latest_release(
    context: &InstallContext,
) -> Result<String> {
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
    compatible_release(
        &versions
            .into_iter()
            .map(|(_, version)| version)
            .collect::<Vec<_>>(),
        super::runtime::java_runtime()?.1,
        context,
    )
}
pub(in crate::infrastructure::language_server) fn download_release(
    stage: &Path,
    version: &str,
    context: &InstallContext,
) -> Result<(String, PathBuf)> {
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
                name.starts_with("org.eclipse.equinox.launcher_") && name.ends_with(".jar")
            })
        })
        .context("JDT LS 缺少 launcher jar")?;
    let (_, current) = super::runtime::java_runtime()?;
    let required = super::runtime::java_requirement(stage)?;
    anyhow::ensure!(
        current >= required,
        "JDT LS {version} 要求 JDK {required}+，当前 Java {current}；请升级 JDK 后重试"
    );
    Ok((source, entry))
}

pub(in crate::infrastructure::language_server) fn compatible_release(
    versions: &[String],
    current: u32,
    context: &InstallContext,
) -> Result<String> {
    let latest = versions.last().context("Eclipse 目录没有 JDT LS 版本")?;
    if current >= 25 {
        return Ok(latest.clone());
    }
    // Eclipse release READMEs can lag behind platform bundle JVM requirements.
    // Select a known compatible release line directly from the installed JDK.
    let ceiling = if current >= 21 {
        "1.60.0"
    } else if current >= 17 {
        "1.43.0"
    } else {
        "1.26.0"
    };
    let ceiling = numeric(ceiling)?;
    let selected = versions
        .iter()
        .rev()
        .find(|version| numeric(version).is_ok_and(|version| version <= ceiling))
        .context("Eclipse 目录没有兼容本机 JDK 的 JDT LS 版本")?;
    context.selection(format!(
        "当前 JDK {current}，使用匹配版本 JDT LS {selected}（最新发行版 {latest}）"
    ))?;
    Ok(selected.clone())
}
