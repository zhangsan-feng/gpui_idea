use super::{InstallContext, core, update};
use crate::domain::language_server::{ManagedServer, ManagedVersion};
use anyhow::{Context, Result};
use std::fs;

// HTML, CSS and JSON share one immutable npm installation.
pub(super) const SERVERS: &[(&str, &str, &str)] = &[
    ("rust-analyzer", "rust-analyzer", "Rust"),
    ("gopls", "gopls", "Go"),
    ("jdtls", "Eclipse JDT LS", "Java"),
    ("taplo", "Taplo", "TOML"),
    ("marksman", "Marksman", "Markdown"),
    ("lsp-python", "Pyright", "Python"),
    ("lsp-php", "Intelephense", "PHP"),
    (
        "lsp-typescript",
        "TypeScript Language Server",
        "JavaScript / TypeScript",
    ),
    ("lsp-web", "VS Code Language Servers", "HTML / CSS / JSON"),
    ("lsp-yaml", "YAML Language Server", "YAML"),
];

fn validate_key(key: &str) -> Result<()> {
    anyhow::ensure!(
        SERVERS.iter().any(|server| server.0 == key),
        "未知语言服务器：{key}"
    );
    Ok(())
}

pub(in crate::infrastructure::language_server) use super::cleanup::clean_history;

pub(super) fn pin(key: &str) -> Result<Option<String>> {
    validate_key(key)?;
    let path = core::root().join(format!("{key}.selected.json"));
    match fs::read(path) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes).context("无法读取 LSP 版本配置")?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn selected(key: &str, directory: &str) -> Result<super::Binary> {
    anyhow::ensure!(
        (directory == key || directory.starts_with(&format!("{key}--")))
            && std::path::Path::new(directory).components().count() == 1
            && !directory.contains(['/', '\\', ':']),
        "无效的服务器版本目录"
    );
    core::cached(&core::root().join(directory)).context("所选版本不完整或已被移除，请重新选择版本")
}

pub(in crate::infrastructure::language_server) fn snapshot() -> Result<Vec<ManagedServer>> {
    let directories = match fs::read_dir(core::root()) {
        Ok(entries) => entries.collect::<std::io::Result<Vec<_>>>()?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.into()),
    };
    SERVERS
        .iter()
        .map(|(key, name, languages)| {
            let mut versions = directories
                .iter()
                .filter_map(|entry| {
                    let directory = entry.file_name().to_string_lossy().into_owned();
                    if directory != *key && !directory.starts_with(&format!("{key}--")) {
                        return None;
                    }
                    let binary = core::cached(&entry.path())?;
                    Some(ManagedVersion {
                        directory,
                        version: binary.version,
                    })
                })
                .collect::<Vec<_>>();
            versions.sort_by(|a, b| a.directory.cmp(&b.directory));
            let (pinned, configuration_error) = match pin(key) {
                Ok(pinned) => (pinned, None),
                Err(error) => {
                    tracing::warn!(key, %error, "LSP version preference is unreadable");
                    (
                        None,
                        Some(format!(
                            "版本配置读取失败：{error:#}；可恢复自动更新以重置。"
                        )),
                    )
                }
            };
            Ok(ManagedServer {
                key: (*key).into(),
                name: (*name).into(),
                languages: (*languages).into(),
                pinned,
                configuration_error,
                versions,
            })
        })
        .collect()
}

pub(in crate::infrastructure::language_server) fn select(
    key: &str,
    directory: Option<String>,
) -> Result<()> {
    validate_key(key)?;
    let slot = update::slot(key);
    let mut checked = slot.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(directory) = &directory {
        selected(key, directory)?;
    }
    save_pin(key, &directory)?;
    *checked = None;
    tracing::info!(key, ?directory, "LSP version preference saved");
    Ok(())
}

// Callers hold the server's update slot while changing the preference.
pub(super) fn save_pin(key: &str, directory: &Option<String>) -> Result<()> {
    fs::create_dir_all(core::root())?;
    let path = core::root().join(format!("{key}.selected.json"));
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&temporary, serde_json::to_vec_pretty(&directory)?)?;
    fs::rename(&temporary, &path).context("保存 LSP 版本选择失败")?;
    Ok(())
}

pub(in crate::infrastructure::language_server) fn install_latest(
    key: &str,
    context: &InstallContext,
) -> Result<()> {
    validate_key(key)?;
    match key {
        "gopls" => super::go_server(context).map(|_| ()),
        "lsp-python" => super::node_server("pyright", "pyright-langserver", context).map(|_| ()),
        "lsp-php" => super::node_server("intelephense", "intelephense", context).map(|_| ()),
        "lsp-typescript" => super::node_server(
            "typescript-language-server",
            "typescript-language-server",
            context,
        )
        .map(|_| ()),
        "lsp-web" => super::node_server(
            "vscode-langservers-extracted",
            "vscode-html-language-server",
            context,
        )
        .map(|_| ()),
        "lsp-yaml" => {
            super::node_server("yaml-language-server", "yaml-language-server", context).map(|_| ())
        }
        _ => super::download_server(key, context).map(|_| ()),
    }
}

pub(in crate::infrastructure::language_server) fn install_context(
    version: Option<String>,
) -> Result<InstallContext> {
    if let Some(version) = &version {
        anyhow::ensure!(
            !version.is_empty()
                && version.len() <= 100
                && version
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)),
            "版本号只能包含字母、数字、点、下划线和连字符"
        );
    }
    let mut context = InstallContext::new(
        super::super::status::StatusRegistry::new().session(),
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        tokio::sync::watch::channel(Vec::new()).0,
        0,
    );
    context.force_update = true;
    context.requested_release = version;
    Ok(context)
}
