mod core;

use super::Language;
pub(super) const LANGUAGE: Language = Language {
    family: "php",
    key: "lsp-php",
    language: "PHP",
    name: "Intelephense",
    managed_name: "Intelephense",
    managed_languages: "PHP",
    markers: &["composer.json", "composer.lock", ".phpactor.json"],
    sections: &["intelephense"],
    prepare: core::prepare,
    install: core::install,
    download: None,
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: None,
    detect_workspace: true,
    workspace_markers: &[],
};
