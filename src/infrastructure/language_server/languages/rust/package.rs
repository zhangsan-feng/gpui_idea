use super::super::{CodePackageSupport, package};
use std::{path::PathBuf, sync::LazyLock};

pub(super) const SUPPORT: CodePackageSupport = CodePackageSupport {
    detect,
    markers: &["Cargo.toml"],
};

fn detect(files: &[PathBuf]) -> bool {
    // Rust module directories contain source files; package roots declare
    // [package]. A workspace-only Cargo.toml does not define a package.
    if package::has_source(files, &["rs"]) {
        return true;
    }
    static PACKAGE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?m)^\s*\[package\]\s*(?:#.*)?$").unwrap());
    files
        .iter()
        .filter(|path| path.file_name().is_some_and(|name| name == "Cargo.toml"))
        .any(|path| package::read_header(path).is_some_and(|source| PACKAGE.is_match(&source)))
}
