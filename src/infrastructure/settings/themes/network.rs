use super::{MAX_PACKAGE, core};
use crate::domain::theme::{ImportedTheme, OnlineTheme, ThemeSearch};
use anyhow::{Context as _, Result, ensure};
use serde_json::Value;
use std::{io::Read, time::Duration};

fn client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent("gpui-idea-theme-browser")
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(90))
        .build()
        .context("无法创建主题下载客户端")
}

fn fetch(url: reqwest::Url, limit: u64) -> Result<Vec<u8>> {
    ensure!(
        url.scheme() == "https" && url.host_str() == Some("open-vsx.org"),
        "主题下载地址无效"
    );
    fetch_with_client(&client()?, url, limit)
}

fn fetch_with_client(
    client: &reqwest::blocking::Client,
    url: reqwest::Url,
    limit: u64,
) -> Result<Vec<u8>> {
    ensure!(
        url.scheme() == "https" && url.host_str() == Some("open-vsx.org"),
        "主题下载地址无效"
    );
    let response = client.get(url).send()?.error_for_status()?;
    ensure!(
        response.content_length().unwrap_or(0) <= limit,
        "主题响应过大"
    );
    let mut data = Vec::new();
    response.take(limit + 1).read_to_end(&mut data)?;
    ensure!(data.len() as u64 <= limit, "主题响应过大");
    Ok(data)
}

/// Category membership includes icon packs and language tools. Verify the actual manifest.
fn color_theme_package(client: &reqwest::blocking::Client, item: &Value) -> Result<bool> {
    let Some(download) = item.pointer("/files/download").and_then(Value::as_str) else {
        return Ok(false);
    };
    let manifest_url = reqwest::Url::parse(download)?.join("package.json")?;
    let bytes = match fetch_with_client(client, manifest_url, 2 * 1024 * 1024) {
        Ok(bytes) => bytes,
        Err(error)
            if error
                .downcast_ref::<reqwest::Error>()
                .and_then(|error| error.status())
                .is_some_and(|status| matches!(status.as_u16(), 404 | 410)) =>
        {
            tracing::debug!(name = ?item["name"], "theme extension manifest unavailable; filtered");
            return Ok(false);
        }
        Err(error) => return Err(error).context("无法检查颜色主题清单，请重试"),
    };
    let manifest: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(name = ?item["name"], %error, "invalid theme extension manifest; filtered");
            return Ok(false);
        }
    };
    let only_themes = manifest["categories"].as_array().is_some_and(|categories| {
        !categories.is_empty()
            && categories
                .iter()
                .all(|category| category.as_str() == Some("Themes"))
    });
    let colors = manifest
        .pointer("/contributes/themes")
        .and_then(Value::as_array)
        .is_some_and(|themes| {
            themes
                .iter()
                .any(|theme| theme["path"].as_str().is_some_and(|path| !path.is_empty()))
        });
    let language_tools = ["languages", "grammars", "debuggers", "snippets"]
        .iter()
        .any(|key| {
            manifest["contributes"][*key]
                .as_array()
                .is_some_and(|entries| !entries.is_empty())
        });
    let accepted = only_themes && colors && !language_tools;
    tracing::debug!(name = ?item["name"], accepted, only_themes, colors, language_tools, "online color theme manifest checked");
    Ok(accepted)
}

fn matches_query(item: &Value, terms: &[String]) -> bool {
    let searchable = format!(
        "{} {} {}.{}",
        item["displayName"].as_str().unwrap_or_default(),
        item["name"].as_str().unwrap_or_default(),
        item["namespace"].as_str().unwrap_or_default(),
        item["name"].as_str().unwrap_or_default(),
    )
    .to_lowercase();
    terms.iter().all(|term| searchable.contains(term))
}

fn verify_candidates(
    client: &reqwest::blocking::Client,
    items: &[Value],
    terms: &[String],
) -> Result<Vec<bool>> {
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|worker| {
                scope.spawn(move || {
                    (worker..items.len())
                        .step_by(4)
                        .map(|index| {
                            (
                                index,
                                if matches_query(&items[index], terms) {
                                    color_theme_package(client, &items[index])
                                } else {
                                    Ok(false)
                                },
                            )
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let mut accepted = vec![false; items.len()];
        for worker in workers {
            for (index, result) in worker
                .join()
                .map_err(|_| anyhow::anyhow!("颜色主题清单检查任务异常"))?
            {
                accepted[index] = result?;
            }
        }
        Ok(accepted)
    })
}

pub(super) fn search(query: String, offset: usize) -> Result<ThemeSearch> {
    let query = query.trim().to_owned();
    let terms: Vec<_> = query.split_whitespace().map(str::to_lowercase).collect();
    let sort_by = if terms.is_empty() {
        "downloadCount"
    } else {
        "relevance"
    };
    tracing::info!(%query, offset, sort_by, "searching color themes by name or extension id");
    let client = client()?;
    let mut result = ThemeSearch::default();
    let mut cursor = offset;
    // Bound the number of manifests per request, while filling pages after filtering.
    for _ in 0..5 {
        // Open VSX's non-aligned offsets can skip/duplicate entries. Fetch whole
        // upstream pages and consume the remaining suffix locally instead.
        let page_offset = cursor / 20 * 20;
        let mut url = reqwest::Url::parse("https://open-vsx.org/api/-/search")?;
        url.query_pairs_mut().extend_pairs([
            ("category", "Themes"),
            ("query", query.as_str()),
            ("size", "20"),
            ("offset", &page_offset.to_string()),
            ("sortBy", sort_by),
            ("sortOrder", "desc"),
        ]);
        let value: Value =
            serde_json::from_slice(&fetch_with_client(&client, url, 2 * 1024 * 1024)?)?;
        let items = value["extensions"]
            .as_array()
            .context("在线主题列表格式无效")?;
        let items = items.get(cursor - page_offset..).unwrap_or_default();
        let total = value["totalSize"]
            .as_u64()
            .context("在线主题列表缺少分页信息")? as usize;
        if items.is_empty() {
            result.next_offset = None;
            return Ok(result);
        }
        let accepted = verify_candidates(&client, items, &terms)?;
        for (index, item) in items.iter().enumerate() {
            cursor += 1;
            if !accepted[index] {
                continue;
            }
            result.extensions.push(OnlineTheme {
                id: format!(
                    "{}.{}",
                    item["namespace"].as_str().unwrap_or_default(),
                    item["name"].as_str().unwrap_or_default()
                ),
                name: item["displayName"]
                    .as_str()
                    .or(item["name"].as_str())
                    .unwrap_or("未命名主题")
                    .into(),
                description: item["description"].as_str().unwrap_or_default().into(),
                version: item["version"].as_str().unwrap_or_default().into(),
                downloads: item["downloadCount"].as_u64().unwrap_or(0),
                download_url: item["files"]["download"]
                    .as_str()
                    .unwrap_or_default()
                    .into(),
            });
            if result.extensions.len() == 20 {
                result.next_offset = (cursor < total).then_some(cursor);
                tracing::debug!(%query, offset, cursor, count = result.extensions.len(), "filtered color theme page ready");
                return Ok(result);
            }
        }
        result.next_offset = (cursor < total).then_some(cursor);
        if result.next_offset.is_none() {
            tracing::info!(%query, offset, cursor, count = result.extensions.len(), "color theme search completed");
            return Ok(result);
        }
    }
    tracing::info!(%query, offset, cursor, count = result.extensions.len(), "color theme search completed");
    Ok(result)
}

pub(super) fn download(extension: OnlineTheme) -> Result<Vec<ImportedTheme>> {
    let data = fetch(reqwest::Url::parse(&extension.download_url)?, MAX_PACKAGE)?;
    core::package(&data, &extension.id)
}
