use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub(super) fn has_marker(files: &[PathBuf], markers: &[&str]) -> bool {
    files.iter().any(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| markers.contains(&name))
    })
}

pub(super) fn has_source(files: &[PathBuf], extensions: &[&str]) -> bool {
    files
        .iter()
        .any(|path| supports_extension(path, extensions))
}

pub(super) fn supports_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extensions
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
}

pub(super) fn source_declares(
    files: &[PathBuf],
    extensions: &[&str],
    pattern: &regex::Regex,
) -> bool {
    files
        .iter()
        .filter(|path| supports_extension(path, extensions))
        .any(|path| {
            read_header(path)
                .is_some_and(|source| pattern.is_match(source.trim_start_matches('\u{feff}')))
        })
}

pub(super) fn read_header(path: &Path) -> Option<String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(64 * 1024)
        .read_to_end(&mut bytes)
        .ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}
