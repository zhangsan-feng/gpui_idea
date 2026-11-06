use super::super::{CodePackageSupport, package};
use std::{path::PathBuf, sync::LazyLock};

pub(super) const SUPPORT: CodePackageSupport = CodePackageSupport {
    detect,
    markers: &["composer.json"],
};

fn detect(files: &[PathBuf]) -> bool {
    if package::has_marker(files, SUPPORT.markers) {
        return true;
    }
    static DECLARATION: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?si)\A\s*<\?php(?:\s|//[^\n]*(?:\n|$)|\#[^\n]*(?:\n|$)|/\*.*?\*/|declare\s*\([^;]*?\)\s*;)*namespace\s+[\p{L}_][\p{L}\p{N}_\\]*\s*[;{]").unwrap()
    });
    package::source_declares(files, super::LANGUAGE.file_extensions, &DECLARATION)
}
