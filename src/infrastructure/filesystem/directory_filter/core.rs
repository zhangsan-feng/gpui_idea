use super::DirectoryFilter;
use std::{
    ffi::OsStr,
    path::{Component, Path},
};

impl DirectoryFilter {
    pub(in crate::infrastructure::filesystem) fn matches_name(&self, name: &OsStr) -> bool {
        self.names.iter().any(|ignored| {
            if cfg!(windows) {
                name.to_string_lossy().eq_ignore_ascii_case(ignored)
            } else {
                name == OsStr::new(ignored)
            }
        })
    }

    pub(in crate::infrastructure::filesystem) fn matches_path(&self, relative: &Path) -> bool {
        relative
            .components()
            .any(|part| matches!(part, Component::Normal(name) if self.matches_name(name)))
    }
}
