use crate::domain::language_server::{LanguageServerInfo, LanguageServerPhase};
use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::sync::watch;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

// Owned by the stdout task: EOF, protocol failure and task cancellation all
// remove the process from the running inventory, independently of session locks.
pub(super) struct Registration {
    pub(super) id: u64,
    inventory: watch::Sender<Vec<LanguageServerInfo>>,
    remove_on_drop: bool,
}

impl Registration {
    pub(super) fn new(
        inventory: watch::Sender<Vec<LanguageServerInfo>>,
        root: &Path,
        family: &str,
        version: Option<String>,
    ) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let (language, name) = match family {
            "rust" => ("Rust", "rust-analyzer"),
            "go" => ("Go", "gopls"),
            "python" => ("Python", "Pyright"),
            "javascript" => (
                "JavaScript / TypeScript / JSX / TSX",
                "typescript-language-server",
            ),
            "java" => ("Java", "Eclipse JDT LS"),
            "toml" => ("TOML", "Taplo"),
            "yaml" => ("YAML", "yaml-language-server"),
            "markdown" => ("Markdown", "Marksman"),
            "html" => ("HTML", "vscode-html-language-server"),
            "css" => ("CSS", "vscode-css-language-server"),
            "json" => ("JSON", "vscode-json-language-server"),
            _ => (family, family),
        };
        inventory.send_modify(|servers| {
            servers.retain(|server| {
                !(server.root == root
                    && server.language == language
                    && server.phase == LanguageServerPhase::Failed)
            });
            servers.push(LanguageServerInfo {
                id,
                language: language.into(),
                name: name.into(),
                version,
                root: root.to_owned(),
                initialized: false,
                phase: LanguageServerPhase::Starting,
                detail: "正在检查语言服务".into(),
                selection_note: None,
                install_directory: None,
                executable: None,
            })
        });
        Self {
            id,
            inventory,
            remove_on_drop: true,
        }
    }

    pub(super) fn failed(&mut self, error: &anyhow::Error) {
        self.remove_on_drop = false;
        self.inventory.send_modify(|servers| {
            if let Some(server) = servers.iter_mut().find(|server| server.id == self.id) {
                server.phase = LanguageServerPhase::Failed;
                server.detail = format!("{error:#}");
            }
        });
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        if !self.remove_on_drop {
            return;
        }
        self.inventory
            .send_modify(|servers| servers.retain(|server| server.id != self.id));
    }
}

pub(super) fn initialized(
    inventory: &watch::Sender<Vec<LanguageServerInfo>>,
    id: u64,
    response: &serde_json::Value,
) {
    inventory.send_modify(|servers| {
        let Some(server) = servers.iter_mut().find(|server| server.id == id) else { return };
        if let Some(name) = response.pointer("/serverInfo/name").and_then(|value| value.as_str()).filter(|value| !value.trim().is_empty()) {
            server.name = name.to_owned();
        }
        if let Some(version) = response.pointer("/serverInfo/version").and_then(|value| value.as_str()).filter(|value| !value.trim().is_empty()) {
            server.version = Some(version.to_owned());
        }
        server.initialized = true;
        server.phase = LanguageServerPhase::Ready;
        server.detail = server.selection_note.as_ref()
            .map(|note| format!("语言服务已就绪；{note}"))
            .unwrap_or_else(|| "语言服务已就绪".into());
        tracing::info!(language = server.language, name = server.name, version = ?server.version, root = %server.root.display(), "Language server initialized");
    });
}
