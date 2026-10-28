mod core;

use super::Language;
pub(super) const LANGUAGE: Language = Language {
    family: "python",
    key: "lsp-python",
    language: "Python",
    name: "Pyright",
    managed_name: "Pyright",
    managed_languages: "Python",
    markers: &[
        "pyrightconfig.json",
        "pyproject.toml",
        "setup.py",
        "requirements.txt",
    ],
    sections: &["python", "python.analysis"],
    prepare: core::prepare,
    install: core::install,
    download: None,
    configuration: core::configuration,
    initialization_options: core::initialization_options,
    native_version: None,
    detect_workspace: true,
    workspace_markers: &[],
};
