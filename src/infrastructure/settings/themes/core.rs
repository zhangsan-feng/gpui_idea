use super::{MAX_FILE, MAX_PACKAGE, jsonc, palette};
use crate::domain::theme::ImportedTheme;
use anyhow::{Context as _, Result, bail, ensure};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
};

pub(super) fn import(path: &Path) -> Result<Vec<ImportedTheme>> {
    let bytes = read_file(path, MAX_PACKAGE)?;
    let source = path.file_name().unwrap_or_default().to_string_lossy();
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("vsix"))
    {
        package(&bytes, &source)
    } else {
        ensure!(bytes.len() as u64 <= MAX_FILE, "主题文件超过 16 MB");
        let root = path.parent().context("无法定位主题目录")?.canonicalize()?;
        let mut read = |relative: &str| {
            let candidate = root
                .join(relative)
                .canonicalize()
                .with_context(|| format!("无法读取主题依赖：{relative}"))?;
            ensure!(candidate.starts_with(&root), "主题依赖必须位于导入目录内");
            read_file(&candidate, MAX_FILE)
        };
        let file = path.file_name().context("无效主题路径")?.to_string_lossy();
        let mut stack = Vec::new();
        let value = resolve(&file, &mut read, &mut stack)?;
        Ok(vec![palette::convert(&value, &source, None, None)?])
    }
}

pub(super) fn read_file(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).with_context(|| format!("无法打开 {}", path.display()))?;
    ensure!(file.metadata()?.len() <= limit, "主题文件过大");
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= limit, "主题文件过大");
    Ok(bytes)
}

pub(super) fn package(bytes: &[u8], source: &str) -> Result<Vec<ImportedTheme>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).context("无效的 VSIX 压缩包")?;
    ensure!(archive.len() <= 10000, "VSIX 包含过多文件");
    let mut files = BTreeMap::new();
    let mut total = 0;
    for index in 0..archive.len() {
        let file = archive.by_index(index)?;
        let name = file.name()?.replace('\\', "/");
        if file.is_dir() || !name.ends_with(".json") && !name.ends_with(".jsonc") {
            continue;
        }
        ensure!(file.size() <= MAX_FILE, "VSIX 中的 JSON 文件过大");
        total += file.size();
        ensure!(total <= MAX_PACKAGE, "VSIX 主题数据超过 64 MB");
        let mut data = Vec::new();
        file.take(MAX_FILE + 1).read_to_end(&mut data)?;
        ensure!(data.len() as u64 <= MAX_FILE, "VSIX 文件解压后过大");
        ensure!(
            files.insert(normalize(&name)?, data).is_none(),
            "VSIX 包含重复文件"
        );
    }
    let manifest = jsonc::parse(
        files
            .get("extension/package.json")
            .context("VSIX 缺少 extension/package.json")?,
    )?;
    let themes = manifest
        .pointer("/contributes/themes")
        .and_then(Value::as_array)
        .context("此扩展未提供颜色主题（可能仅提供图标主题）")?;
    ensure!(
        !themes.is_empty() && themes.len() <= 200,
        "主题包的配色数量无效"
    );
    let mut result = Vec::new();
    for entry in themes {
        let path = entry["path"].as_str().context("主题包缺少配色文件路径")?;
        let path = normalize(&format!("extension/{path}"))?;
        ensure!(path.starts_with("extension/"), "主题路径超出扩展目录");
        let mut read = |name: &str| {
            files
                .get(name)
                .cloned()
                .with_context(|| format!("VSIX 缺少主题依赖：{name}"))
        };
        let value = resolve(&path, &mut read, &mut Vec::new())?;
        let mut theme = palette::convert(
            &value,
            source,
            entry["label"].as_str(),
            entry["uiTheme"].as_str(),
        )?;
        theme.package_name = manifest["displayName"]
            .as_str()
            .filter(|name| !name.starts_with('%'))
            .or(manifest["name"].as_str())
            .unwrap_or(source)
            .to_owned();
        result.push(theme);
    }
    Ok(result)
}

fn normalize(path: &str) -> Result<String> {
    let path = path.replace('\\', "/");
    let mut parts = Vec::new();
    for part in Path::new(&path).components() {
        match part {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                ensure!(parts.pop().is_some(), "主题路径超出根目录");
            }
            _ => bail!("主题依赖必须使用相对路径"),
        }
    }
    Ok(parts.join("/"))
}

fn resolve(
    path: &str,
    read: &mut impl FnMut(&str) -> Result<Vec<u8>>,
    stack: &mut Vec<String>,
) -> Result<Value> {
    ensure!(
        stack.len() < 16 && !stack.iter().any(|item| item == path),
        "主题 include 循环或嵌套过深"
    );
    stack.push(path.to_owned());
    let mut current = jsonc::parse(&read(path)?)?;
    ensure!(current.is_object(), "主题必须是 JSON 对象");
    if let Some(include) = current.get("include").and_then(Value::as_str) {
        let parent = PathBuf::from(path)
            .parent()
            .unwrap_or(Path::new(""))
            .join(include);
        let include = normalize(&parent.to_string_lossy())?;
        let mut base = resolve(&include, read, stack)?;
        merge(&mut base, current);
        current = base;
    }
    // VS Code also permits tokenColors to refer to an external TextMate plist.
    ensure!(
        !current["tokenColors"].is_string(),
        "暂不支持外部 .tmTheme 文件，请使用 tokenColors 为数组的 JSON 主题"
    );
    stack.pop();
    Ok(current)
}

fn merge(base: &mut Value, next: Value) {
    if let (Some(base), Some(next)) = (base.as_object_mut(), next.as_object()) {
        for (key, value) in next {
            if key == "tokenColors" {
                if let (Some(old), Some(new)) = (
                    base.get_mut(key).and_then(Value::as_array_mut),
                    value.as_array(),
                ) {
                    old.extend(new.iter().cloned());
                    continue;
                }
            }
            if matches!(key.as_str(), "colors" | "semanticTokenColors")
                && base.get(key).is_some_and(Value::is_object)
                && value.is_object()
            {
                merge(base.get_mut(key).unwrap(), value.clone());
            } else {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}
