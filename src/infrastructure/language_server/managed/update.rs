use super::{Binary, InstallContext, Installed, core};
use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
use std::{fs, path::Path};

type CheckedRelease = Arc<Mutex<Option<Binary>>>;
static CHECKED: LazyLock<Mutex<HashMap<String, CheckedRelease>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(super) fn existing(key: &str) -> Option<Binary> {
    let mut directories = fs::read_dir(core::root())
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name == key || name.starts_with(&format!("{key}--"))
        })
        .collect::<Vec<_>>();
    directories
        .sort_by_key(|entry| std::cmp::Reverse(entry.metadata().and_then(|m| m.modified()).ok()));
    directories
        .into_iter()
        .find_map(|entry| core::cached(&entry.path()))
}

// Versioned directories keep a running server usable while a new release installs.
// A failed check/download never replaces the previous complete installation.
pub(super) fn ensure(
    key: &str,
    label: &str,
    context: &InstallContext,
    resolve: impl FnOnce() -> Result<String>,
    prepare: impl FnOnce(&Path, &str) -> Result<Installed>,
) -> Result<Binary> {
    let slot = CHECKED
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .entry(key.to_owned())
        .or_insert_with(|| Arc::new(Mutex::new(None)))
        .clone();
    let mut checked = slot.lock().unwrap_or_else(|error| error.into_inner());
    context.check()?;
    if let Some(binary) = checked.as_ref() {
        if binary.path.is_file() {
            if let Some(note) = &binary.selection {
                context.selection(note.clone())?;
            }
            if let Some(directory) = binary
                .path
                .ancestors()
                .find(|path| path.join("installed.json").is_file())
            {
                context.target(directory.to_owned());
            }
            return Ok(binary.clone());
        }
    }
    context.progress(format!("正在检查 {label} 更新…"))?;
    let result = (|| {
        let version = resolve()?;
        anyhow::ensure!(
            !version.is_empty()
                && version
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)),
            "无效服务器版本：{version}"
        );
        tracing::info!(label, %version, "LSP latest release resolved");
        core::install(&format!("{key}--{version}"), label, context, |stage| {
            prepare(stage, &version)
        })
    })();
    match result {
        Ok(binary) => {
            if let Some(note) = binary
                .selection
                .as_ref()
                .filter(|_| context.selected_version().is_none())
            {
                context.selection(note.clone())?;
            }
            *checked = Some(binary.clone());
            Ok(binary)
        }
        Err(error) => {
            context.check()?;
            if let Some(binary) = existing(key) {
                if let Some(note) = &binary.selection {
                    context.selection(note.clone())?;
                }
                tracing::warn!(label, error = %format!("{error:#}"), version = binary.version, "LSP update failed; using cached release");
                context.progress(format!(
                    "{label} 更新失败，继续使用已安装版本 {}",
                    binary.version
                ))?;
                if let Some(directory) = binary
                    .path
                    .ancestors()
                    .find(|path| path.join("installed.json").is_file())
                {
                    context.target(directory.to_owned());
                }
                *checked = Some(binary.clone());
                Ok(binary)
            } else {
                Err(error)
            }
        }
    }
}

pub(in crate::infrastructure::language_server) fn refresh(
    root: std::path::PathBuf,
    inventory: tokio::sync::watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
    stopping: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    // Background maintenance publishes inventory details without overwriting a
    // running session's Ready/Indexing/Warming status.
    let status =
        tokio::sync::watch::channel(crate::domain::language_server::LanguageServerStatus::Stopped)
            .0;
    for (key, family, package, bin) in [
        ("rust-analyzer", "rust", "", ""),
        ("taplo", "toml", "", ""),
        ("marksman", "markdown", "", ""),
        ("jdtls", "java", "", ""),
        ("gopls", "go", "", ""),
        ("lsp-python", "python", "pyright", "pyright-langserver"),
        (
            "lsp-typescript",
            "javascript",
            "typescript-language-server",
            "typescript-language-server",
        ),
        (
            "lsp-web",
            "html",
            "vscode-langservers-extracted",
            "vscode-html-language-server",
        ),
        (
            "lsp-yaml",
            "yaml",
            "yaml-language-server",
            "yaml-language-server",
        ),
    ] {
        if stopping.load(std::sync::atomic::Ordering::Acquire) {
            break;
        }
        if existing(key).is_none() {
            continue;
        }
        let mut registration =
            super::super::inventory::Registration::new(inventory.clone(), &root, family, None);
        let context = InstallContext::new(
            status.clone(),
            stopping.clone(),
            inventory.clone(),
            registration.id,
        );
        let result = match key {
            "gopls" => super::go_server(&context).map(|_| ()),
            _ if !package.is_empty() => super::node_server(package, bin, &context).map(|_| ()),
            _ => super::download_server(key, &context).map(|_| ()),
        };
        if let Err(error) = result {
            tracing::warn!(key, error = %format!("{error:#}"), "Startup LSP update failed");
            registration.failed(&error);
        }
    }
}
