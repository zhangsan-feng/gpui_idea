mod core;

use super::Language;
pub(super) const LANGUAGE: Language = Language {
    family: "css",
    key: "lsp-web",
    language: "CSS",
    name: "vscode-css-language-server",
    managed_name: "VS Code Language Servers",
    managed_languages: "HTML / CSS / JSON",
    markers: &["package.json"],
    sections: &["css"],
    prepare: core::prepare,
    install: core::install,
    download: None,
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: None,
    detect_workspace: false,
    workspace_markers: &[],
};
