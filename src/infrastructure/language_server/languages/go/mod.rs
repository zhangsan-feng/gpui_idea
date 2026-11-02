mod core;
mod package;
mod run;

use super::Language;
pub(super) const LANGUAGE: Language = Language {
    family: "go",
    key: "gopls",
    language: "Go",
    name: "gopls",
    managed_name: "gopls",
    managed_languages: "Go",
    markers: &["go.work", "go.mod"],
    sections: &["gopls"],
    prepare: core::prepare,
    install: core::install,
    download: None,
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: Some("version"),
    detect_workspace: core::detect_workspace,
    file_run: Some(run::SUPPORT),
    file_extensions: &["go"],
    code_package: Some(package::SUPPORT),
};
