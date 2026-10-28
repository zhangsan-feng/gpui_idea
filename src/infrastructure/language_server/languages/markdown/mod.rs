mod core;

use super::{Download, Language};
pub(super) const LANGUAGE: Language = Language {
    family: "markdown",
    key: "marksman",
    language: "Markdown",
    name: "Marksman",
    managed_name: "Marksman",
    managed_languages: "Markdown",
    markers: &[".marksman.toml", "mkdocs.yml", "package.json"],
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
