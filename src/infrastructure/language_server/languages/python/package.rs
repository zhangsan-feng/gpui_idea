use super::super::CodePackageSupport;
use std::path::PathBuf;

pub(super) const SUPPORT: CodePackageSupport = CodePackageSupport {
    detect,
    markers: &["__init__.py", "__init__.pyi"],
};

fn detect(files: &[PathBuf]) -> bool {
    // Explicit Python packages and stub packages. Cache folders and arbitrary
    // directories are not classified as namespace packages by a name heuristic.
    super::super::package::has_marker(files, SUPPORT.markers)
}
