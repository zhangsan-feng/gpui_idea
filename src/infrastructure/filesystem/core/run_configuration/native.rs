use super::common::*;
use crate::domain::run::RunConfiguration;
use anyhow::Result;
use std::path::Path;

pub(super) fn rust_file(file: &Path) -> RunConfiguration {
    compiled_file(file, "Rust · 文件", "rustc", "-g")
}

pub(super) fn go(
    directory: &Path,
    manifest: Option<&Path>,
    file: Option<&Path>,
) -> RunConfiguration {
    let target = if manifest.is_some() {
        ".".into()
    } else {
        quote(file.unwrap())
    };
    let debug = executable("dlv").map(|_| in_directory(directory, &format!("dlv debug {target}")));
    let output = directory
        .join(".gpui-run")
        .join(if cfg!(windows) { "go.exe" } else { "go" });
    let build = with_output_directory(
        output.parent().unwrap(),
        &format!("go build -o {} {target}", quote(&output)),
    );
    let mut result = configuration(
        if manifest.is_some() {
            "Go · 包"
        } else {
            "Go · 文件"
        },
        in_directory(directory, &format!("go run {target}")),
        Some(in_directory(directory, &build)),
        debug,
    );
    result.profiles[0].debug_argument_prefix = "--".into();
    result
}

pub(super) fn cpp(
    directory: &Path,
    root: &Path,
    file: &Path,
    extension: &str,
) -> Result<RunConfiguration> {
    if let Some(manifest) = nearest(directory, root, "CMakeLists.txt") {
        return cmake(manifest.parent().unwrap());
    }
    let c = extension == "c";
    let compiler = if c {
        ["clang", "gcc"]
    } else {
        ["clang++", "g++"]
    };
    let compiler = compiler
        .into_iter()
        .find(|name| executable(name).is_some())
        .unwrap_or(if c { "clang" } else { "clang++" });
    Ok(compiled_file(
        file,
        if c { "C · 文件" } else { "C++ · 文件" },
        compiler,
        "-g",
    ))
}

fn compiled_file(file: &Path, label: &str, compiler: &str, flags: &str) -> RunConfiguration {
    let directory = file.parent().unwrap();
    let mut output = directory.join(".gpui-run").join(file.file_stem().unwrap());
    if cfg!(windows) {
        output.set_extension("exe");
    }
    let build = with_output_directory(
        output.parent().unwrap(),
        &format!("{compiler} {flags} {} -o {}", quote(file), quote(&output)),
    );
    let run = format!("{build} && {}", quote(&output));
    let debug = executable("gdb")
        .map(|_| in_directory(directory, &format!("{build} && gdb {}", quote(&output))));
    let mut result = configuration(
        label,
        in_directory(directory, &run),
        Some(in_directory(directory, &build)),
        debug,
    );
    result.profiles[0].argument_template = Some(in_directory(
        directory,
        &run.replace(
            &format!("&& {}", quote(&output)),
            &format!("&& {} __GPUI_PROGRAM_ARGUMENTS__", quote(&output)),
        ),
    ));
    result.profiles[0].debug_argument_template =
        result.profiles[0].debug_command.as_ref().map(|command| {
            command.replace(
                &format!("gdb {}", quote(&output)),
                &format!("gdb --args {} __GPUI_PROGRAM_ARGUMENTS__", quote(&output)),
            )
        });
    result
}

pub(super) fn cmake(directory: &Path) -> Result<RunConfiguration> {
    // CMake does not declare a unique runtime entry. Expose only its build capability.
    Ok(configuration(
        "C/C++ · CMake",
        String::new(),
        Some(in_directory(
            directory,
            "cmake -S . -B .gpui-run/cmake && cmake --build .gpui-run/cmake",
        )),
        None,
    ))
}
