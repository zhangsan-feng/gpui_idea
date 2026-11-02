mod core;
mod package;
mod run;

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
    detect_workspace: core::detect_workspace,
    file_run: Some(run::SUPPORT),
    file_extensions: &[
        "php", "phtml", "php3", "php4", "php5", "php7", "php8", "phps",
    ],
    code_package: Some(package::SUPPORT),
};
