#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct LanguageServerInfo {
    pub(crate) id: u64,
    pub(crate) language: String,
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) root: std::path::PathBuf,
    pub(crate) initialized: bool,
    pub(crate) phase: LanguageServerPhase,
    pub(crate) detail: String,
    pub(crate) selection_note: Option<String>,
    pub(crate) install_directory: Option<std::path::PathBuf>,
    pub(crate) executable: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum LanguageServerPhase {
    Installing,
    #[default]
    Starting,
    Ready,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum LanguageServerStatus {
    #[default]
    Stopped,
    Starting,
    Preparing(String),
    Indexing,
    Warming,
    Ready,
    Warning(String),
    Error(String),
}

pub(crate) fn family(language: &str) -> Option<&'static str> {
    match language {
        "rust" => Some("rust"),
        "python" => Some("python"),
        "javascript" | "javascriptreact" | "typescript" | "typescriptreact" | "tsx" => {
            Some("javascript")
        }
        "go" => Some("go"),
        "java" => Some("java"),
        "toml" => Some("toml"),
        "yaml" => Some("yaml"),
        "markdown" => Some("markdown"),
        "html" => Some("html"),
        "css" => Some("css"),
        "json" => Some("json"),
        _ => None,
    }
}

pub(crate) fn language_for_path(path: &std::path::Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "rs" => Some("rust"),
        "py" | "pyw" | "pyi" => Some("python"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "jsx" => Some("javascriptreact"),
        "ts" | "mts" | "cts" => Some("typescript"),
        "tsx" => Some("typescriptreact"),
        "go" => Some("go"),
        "java" => Some("java"),
        "toml" => Some("toml"),
        "yaml" | "yml" => Some("yaml"),
        "md" | "markdown" | "mdown" | "mkd" => Some("markdown"),
        "html" | "htm" => Some("html"),
        "css" => Some("css"),
        "json" => Some("json"),
        _ => None,
    }
}
