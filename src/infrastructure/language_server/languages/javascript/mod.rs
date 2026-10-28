mod core;

use super::Language;
pub(super) const LANGUAGE: Language = Language {
    family: "javascript",
    key: "lsp-typescript",
    language: "JavaScript / TypeScript / JSX / TSX",
    name: "typescript-language-server",
    managed_name: "TypeScript Language Server",
    managed_languages: "JavaScript / TypeScript",
    markers: &["tsconfig.json", "jsconfig.json", "package.json"],
    sections: &["javascript", "typescript"],
    prepare: core::prepare,
    install: core::install,
    download: None,
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: None,
    detect_workspace: true,
    workspace_markers: &[],
};
