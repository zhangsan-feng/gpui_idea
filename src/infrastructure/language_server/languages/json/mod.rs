mod core;

use super::Language;
pub(super) const LANGUAGE: Language = Language {
    family: "json",
    key: "lsp-web",
    language: "JSON",
    name: "vscode-json-language-server",
    managed_name: "VS Code Language Servers",
    managed_languages: "HTML / CSS / JSON",
    markers: &["package.json"],
    sections: &["json"],
    prepare: core::prepare,
    install: core::install,
    download: None,
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: None,
    detect_workspace: false,
    workspace_markers: &[],
};
