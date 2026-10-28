mod core;

use super::{Download, Language};
pub(super) const LANGUAGE: Language = Language {
    family: "toml",
    key: "taplo",
    language: "TOML",
    name: "Taplo",
    managed_name: "Taplo",
    managed_languages: "TOML",
    markers: &["taplo.toml", ".taplo.toml", "Cargo.toml", "pyproject.toml"],
    sections: &[],
    prepare: core::prepare,
    install: core::install,
    download: Some(Download {
        latest: core::latest_release,
        release: core::download_release,
    }),
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: Some("--version"),
    detect_workspace: true,
    workspace_markers: &[],
};
