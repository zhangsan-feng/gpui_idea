use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::{Child, Command};

mod native;

pub(super) fn family(language: &str) -> Result<&'static str> {
    crate::domain::language_server::family(language)
        .with_context(|| format!("尚未接入 {language} 语言服务"))
}

pub(super) fn executable(name: &str) -> Option<PathBuf> {
    for directory in std::env::split_paths(&std::env::var_os("PATH")?) {
        let candidate = directory.join(if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_owned()
        });
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub(super) fn spawn(
    root: &Path,
    language: &str,
    preparation: &super::managed::InstallContext,
) -> Result<(Child, Option<String>, Option<PathBuf>)> {
    let family = family(language)?;
    let mut installed_version = None;
    let mut command = match family {
        "rust" => Command::new(super::managed::rust_server(preparation)?),
        "go" => Command::new(super::managed::go_server(preparation)?),
        "toml" | "markdown" => {
            let name = match family {
                "toml" => "taplo",
                "markdown" => "marksman",
                _ => unreachable!(),
            };
            let program = super::managed::download_server(name, preparation)?;
            let mut command = Command::new(program);
            match family {
                "toml" => {
                    command.args(["lsp", "stdio"]);
                }
                "markdown" => {
                    command.arg("server");
                }
                _ => {}
            }
            command
        }
        "java" => native::java(root, preparation)?,
        "python" | "javascript" | "html" | "css" | "json" | "yaml" => {
            let (package, binary) = match family {
                "python" => ("pyright", "pyright-langserver"),
                "javascript" => ("typescript-language-server", "typescript-language-server"),
                "html" => (
                    "vscode-langservers-extracted",
                    "vscode-html-language-server",
                ),
                "css" => ("vscode-langservers-extracted", "vscode-css-language-server"),
                "json" => (
                    "vscode-langservers-extracted",
                    "vscode-json-language-server",
                ),
                "yaml" => ("yaml-language-server", "yaml-language-server"),
                _ => unreachable!(),
            };
            let (command, version) = super::managed::node_server(package, binary, preparation)?;
            installed_version = version;
            command
        }
        _ => bail!("不支持的语言服务：{family}"),
    };
    let version_program = matches!(family, "rust" | "go" | "toml" | "markdown")
        .then(|| PathBuf::from(command.as_std().get_program()));
    preparation.before_launch()?;
    preparation.launcher(
        PathBuf::from(command.as_std().get_program()),
        installed_version.clone(),
    );
    command
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let child = command
        .spawn()
        .with_context(|| format!("无法启动 {family} 语言服务"))?;
    tracing::debug!(root = %root.display(), language = family, "Starting language server");
    Ok((child, installed_version, version_program))
}

pub(super) async fn native_version(program: PathBuf, argument: &str) -> Option<String> {
    let mut command = Command::new(program);
    command
        .arg(argument)
        .stdin(Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let output = tokio::time::timeout(std::time::Duration::from_secs(3), command.output())
        .await
        .ok()?
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_owned())
}

fn markers(language: &str) -> &'static [&'static str] {
    match language {
        "rust" => &["Cargo.toml"],
        "go" => &["go.work", "go.mod"],
        "python" => &[
            "pyrightconfig.json",
            "pyproject.toml",
            "setup.py",
            "requirements.txt",
        ],
        "javascript" => &["tsconfig.json", "jsconfig.json", "package.json"],
        "java" => &[
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
            ".project",
        ],
        "toml" => &["taplo.toml", ".taplo.toml", "Cargo.toml", "pyproject.toml"],
        "yaml" => &[
            ".yamllint",
            ".yamllint.yaml",
            "package.json",
            "pyproject.toml",
        ],
        "markdown" => &[".marksman.toml", "mkdocs.yml", "package.json"],
        "html" | "css" | "json" => &["package.json"],
        _ => &[],
    }
}

pub(super) fn document_root(workspace: &Path, path: &Path, language: &str) -> PathBuf {
    if language == "go" {
        for directory in path.parent().into_iter().flat_map(Path::ancestors) {
            if !directory.starts_with(workspace) {
                break;
            }
            if directory.join("go.work").is_file() {
                return directory.to_owned();
            }
        }
    }
    for directory in path.parent().into_iter().flat_map(Path::ancestors) {
        if !directory.starts_with(workspace) {
            break;
        }
        if markers(language)
            .iter()
            .any(|marker| directory.join(marker).is_file())
        {
            return directory.to_owned();
        }
        if directory == workspace {
            break;
        }
    }
    workspace.to_owned()
}

pub(super) fn workspace_language(root: &Path) -> Option<&'static str> {
    [
        "rust",
        "go",
        "python",
        "javascript",
        "java",
        "toml",
        "markdown",
    ]
    .into_iter()
    .find(|language| {
        markers(language)
            .iter()
            .any(|marker| root.join(marker).is_file())
    })
}
