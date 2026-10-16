use crate::domain::run::{RunConfiguration, RunProfile};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn nearest(start: &Path, root: &Path, name: &str) -> Option<PathBuf> {
    start
        .ancestors()
        .take_while(|directory| directory.starts_with(root))
        .map(|directory| directory.join(name))
        .find(|path| {
            fs::symlink_metadata(path)
                .is_ok_and(|metadata| metadata.is_file() && !super::super::path::is_link(&metadata))
        })
}

pub(super) fn quote(path: impl AsRef<Path>) -> String {
    let path = path.as_ref().to_string_lossy();
    let path = if let Some(unc) = path.strip_prefix("\\\\?\\UNC\\") {
        format!("\\\\{unc}")
    } else {
        path.strip_prefix("\\\\?\\").unwrap_or(&path).to_string()
    };
    if cfg!(windows) {
        // Generated commands execute in a batch file; protect percent expansion.
        format!("\"{}\"", path.replace('%', "%%").replace('"', "\"\""))
    } else {
        format!("'{}'", path.replace('\'', "'\"'\"'"))
    }
}

pub(super) fn in_directory(directory: &Path, command: &str) -> String {
    let cd = if cfg!(windows) { "cd /d" } else { "cd" };
    format!("{cd} {} && {command}", quote(directory))
}

pub(super) fn with_output_directory(directory: &Path, command: &str) -> String {
    let directory = quote(directory);
    if cfg!(windows) {
        format!("(if not exist {directory} mkdir {directory}) && {command}")
    } else {
        format!("mkdir -p {directory} && {command}")
    }
}

pub(super) fn configuration(
    label: impl Into<String>,
    command: String,
    build_command: Option<String>,
    debug_command: Option<String>,
) -> RunConfiguration {
    RunConfiguration {
        label: label.into(),
        command: command.clone(),
        profiles: vec![RunProfile {
            name: "default".into(),
            command,
            build_command,
            debug_command,
            argument_prefix: String::new(),
            argument_template: None,
            debug_argument_prefix: String::new(),
            debug_argument_template: None,
        }],
        ..Default::default()
    }
}

pub(super) fn executable(name: &str) -> Option<PathBuf> {
    let suffixes: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
            .split(';')
            .map(str::to_owned)
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&std::env::var_os("PATH")?).find_map(|directory| {
        std::iter::once(String::new())
            .chain(suffixes.iter().cloned())
            .map(|suffix| directory.join(format!("{name}{suffix}")))
            .find(|path| path.is_file())
    })
}
