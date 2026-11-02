use super::super::{CodePackageSupport, package};
use std::path::PathBuf;

pub(super) const SUPPORT: CodePackageSupport = CodePackageSupport {
    detect,
    markers: &["package.json"],
};

fn detect(files: &[PathBuf]) -> bool {
    // Includes JavaScript/TypeScript workspace packages and scoped packages.
    package::has_marker(files, SUPPORT.markers)
}
