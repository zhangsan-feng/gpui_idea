use super::{LanguageServers, languages};
use std::path::Path;

// Called by directory loading inside spawn_blocking; never perform this IO
// from the GUI or while rendering an icon.
pub(in crate::infrastructure) fn is_code_package_directory(directory: &Path) -> bool {
    // Excluded build/cache directories retain their dedicated directory icons.
    if directory
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git"
                    | "target"
                    | "node_modules"
                    | "dist"
                    | "build"
                    | ".venv"
                    | "venv"
                    | "__pycache__"
            )
        })
    {
        return false;
    }
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::debug!(path = %directory.display(), %error, "code package directory inspection unavailable");
            return false;
        }
    };
    // Inspect direct regular files once for every language; never follow links
    // or recursively walk a subtree to decide a single row's icon.
    let files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
        .collect();
    let language = languages::ALL.iter().find(|language| {
        language
            .code_package
            .as_ref()
            .is_some_and(|support| (support.detect)(&files))
    });
    if let Some(language) = language {
        tracing::debug!(path = %directory.display(), family = language.family, "code package directory identified");
    }
    language.is_some()
}

impl LanguageServers {
    pub(crate) fn is_code_package_marker(&self, path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        languages::ALL.iter().any(|language| {
            language.code_package.as_ref().is_some_and(|support| {
                support.markers.contains(&name) || language.supports_file(path)
            })
        })
    }
}
