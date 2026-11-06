use super::super::{CodePackageSupport, package};
use std::{path::PathBuf, sync::LazyLock};

pub(super) const SUPPORT: CodePackageSupport = CodePackageSupport {
    detect,
    markers: &[],
};

fn detect(files: &[PathBuf]) -> bool {
    static DECLARATION: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?s)\A(?:\s|//[^\n]*(?:\n|$)|/\*.*?\*/)*package(?:\s|//[^\n]*(?:\n|$)|/\*.*?\*/)+[\p{L}_][\p{L}\p{N}_]*\b").unwrap()
    });
    package::source_declares(files, &["go"], &DECLARATION)
}
