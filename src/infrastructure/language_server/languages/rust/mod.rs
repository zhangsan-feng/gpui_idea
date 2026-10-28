mod core;

use super::{Download, Language};
pub(super) const LANGUAGE: Language = Language {
    family: "rust",
    key: "rust-analyzer",
    language: "Rust",
    name: "rust-analyzer",
    managed_name: "rust-analyzer",
    managed_languages: "Rust",
    markers: &["Cargo.toml"],
    sections: &["rust-analyzer"],
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
