use super::InstallContext;
use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    time::{Duration, Instant},
};

pub(super) fn client() -> Result<reqwest::blocking::Client> {
    super::network::require_online()?;
    Ok(crate::infrastructure::INFRASTRUCTURE
        .proxy
        .lsp_http_client(
            reqwest::blocking::Client::builder()
                .user_agent("gpui-idea")
                .https_only(true)
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(300))
                .redirect(reqwest::redirect::Policy::limited(8)),
        )?
        .build()?)
}

pub(super) fn text(url: &str) -> Result<String> {
    read_text(url, 4 * 1024 * 1024, "*/*")
}

pub(super) fn text_limit(url: &str, limit: u64) -> Result<String> {
    read_text(url, limit, "application/vnd.npm.install-v1+json")
}

fn read_text(url: &str, limit: u64, accept: &str) -> Result<String> {
    let response = client()?
        .get(url)
        .header("Accept", accept)
        .timeout(Duration::from_secs(20))
        .send()?
        .error_for_status()?;
    let mut bytes = Vec::new();
    response.take(limit + 1).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() as u64 <= limit, "发行元数据超过大小限制");
    Ok(String::from_utf8(bytes)?)
}

pub(super) fn latest(repo: &str) -> Result<String> {
    let response = client()?
        .get(format!("https://github.com/{repo}/releases/latest"))
        .timeout(Duration::from_secs(20))
        .send()?
        .error_for_status()?;
    let tag = response
        .url()
        .path()
        .split_once("/releases/tag/")
        .map(|(_, tag)| tag.to_owned())
        .context("GitHub 最新发行版本无效")?;
    anyhow::ensure!(
        !tag.is_empty()
            && tag
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)),
        "无效发行版本"
    );
    Ok(tag)
}

pub(super) fn download(url: &str, path: &Path, context: &InstallContext) -> Result<()> {
    tracing::info!(url, "Downloading LSP release with reqwest");
    let mut response = client()?.get(url).send()?.error_for_status()?;
    let total = response.content_length();
    let limit = 512 * 1024 * 1024;
    anyhow::ensure!(
        response.content_length().unwrap_or(0) <= limit,
        "下载包超过 512 MiB"
    );
    let mut output = File::create_new(path)?;
    let mut buffer = [0; 65536];
    let mut received = 0u64;
    let mut reported = Instant::now();
    context.download_progress(0, total)?;
    loop {
        context.check()?;
        let count = response.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        received += count as u64;
        anyhow::ensure!(received <= limit, "下载包超过 512 MiB");
        output.write_all(&buffer[..count])?;
        if reported.elapsed() >= Duration::from_millis(250) {
            context.download_progress(received, total)?;
            reported = Instant::now();
        }
    }
    context.download_progress(received, total)?;
    output.sync_all()?;
    Ok(())
}
