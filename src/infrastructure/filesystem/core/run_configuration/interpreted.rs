use super::common::*;
use crate::domain::run::RunConfiguration;
use std::path::Path;

pub(super) fn resolve(
    directory: &Path,
    root: &Path,
    file: &Path,
    extension: &str,
) -> RunConfiguration {
    let file_arg = quote(file);
    let (label, command, build, debug) = match extension {
        "java" => ("Java", format!("java {file_arg}"), None, None),
        "py" | "pyw" => {
            let interpreter = [
                ".venv/Scripts/python.exe",
                "venv/Scripts/python.exe",
                ".venv/bin/python",
                "venv/bin/python",
            ]
            .into_iter()
            .find_map(|name| nearest(directory, root, name));
            let runner = interpreter.map(quote).unwrap_or_else(|| {
                if executable("python").is_some() {
                    "python".into()
                } else if cfg!(windows) {
                    "py -3".into()
                } else {
                    "python3".into()
                }
            });
            (
                "Python",
                format!("{runner} {file_arg}"),
                Some(format!("{runner} -m py_compile {file_arg}")),
                Some(format!("{runner} -m pdb {file_arg}")),
            )
        }
        "rb" => (
            "Ruby",
            format!("ruby {file_arg}"),
            Some(format!("ruby -c {file_arg}")),
            None,
        ),
        "php" => (
            "PHP",
            format!("php {file_arg}"),
            Some(format!("php -l {file_arg}")),
            None,
        ),
        "ps1" => {
            let runner = if executable("pwsh").is_some() || !cfg!(windows) {
                "pwsh"
            } else {
                "powershell.exe"
            };
            (
                "PowerShell",
                format!("{runner} -NoLogo -File {file_arg}"),
                None,
                None,
            )
        }
        "sh" => (
            "Shell",
            format!("bash {file_arg}"),
            Some(format!("bash -n {file_arg}")),
            Some(format!("bash -x {file_arg}")),
        ),
        "lua" => ("Lua", format!("lua {file_arg}"), None, None),
        "pl" => (
            "Perl",
            format!("perl {file_arg}"),
            Some(format!("perl -c {file_arg}")),
            Some(format!("perl -d {file_arg}")),
        ),
        _ => unreachable!(),
    };
    configuration(
        format!(
            "{label} · {}",
            file.file_name().unwrap_or_default().to_string_lossy()
        ),
        in_directory(directory, &command),
        build.map(|c| in_directory(directory, &c)),
        debug.map(|c| in_directory(directory, &c)),
    )
}
