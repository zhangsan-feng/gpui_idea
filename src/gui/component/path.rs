use std::path::Path;

/// Hide Windows verbatim path prefixes in labels and tooltips.
pub(crate) fn display_path(path: &Path) -> String {
    let path = path.to_string_lossy();
    if cfg!(windows) {
        if let Some(network) = path.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{network}");
        }
        return path.strip_prefix(r"\\?\").unwrap_or(&path).to_owned();
    }
    path.into_owned()
}
