use super::super::{CodePackageSupport, package};
use std::{path::PathBuf, sync::LazyLock};

pub(super) const SUPPORT: CodePackageSupport = CodePackageSupport {
    detect,
    markers: &[],
};

fn detect(files: &[PathBuf]) -> bool {
    // A default-package source folder has no named Java package to display.
    static DECLARATION: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r"(?s)\A(?:\s|//[^\n]*(?:\n|$)|/\*.*?\*/)*package\s+[\p{L}_$][\p{L}\p{N}_$.]*\s*;",
        )
        .unwrap()
    });
    package::source_declares(files, &["java"], &DECLARATION)
}
