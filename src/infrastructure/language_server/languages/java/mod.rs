mod core;
mod install;
mod launch;
mod runtime;
use super::{Download, Language};
pub(super) const LANGUAGE: Language = Language {
    family: "java",
    key: "jdtls",
    language: "Java",
    name: "Eclipse JDT LS",
    managed_name: "Eclipse JDT LS",
    managed_languages: "Java",
    markers: &[
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "settings.gradle",
        "settings.gradle.kts",
        ".project",
    ],
    sections: &["java"],
    prepare: core::prepare,
    install: core::install,
    download: Some(Download {
        latest: install::latest_release,
        release: install::download_release,
    }),
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: None,
    detect_workspace: true,
    workspace_markers: &[],
};
