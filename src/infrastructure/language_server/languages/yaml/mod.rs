mod core;

use super::Language;
pub(super) const LANGUAGE: Language = Language {
    family: "yaml",
    key: "lsp-yaml",
    language: "YAML",
    name: "yaml-language-server",
    managed_name: "YAML Language Server",
    managed_languages: "YAML",
    markers: &[
        ".yamllint",
        ".yamllint.yaml",
        "package.json",
        "pyproject.toml",
    ],
    sections: &["yaml"],
    prepare: core::prepare,
    install: core::install,
    download: None,
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: None,
    detect_workspace: false,
    workspace_markers: &[],
};
