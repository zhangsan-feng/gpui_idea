use crate::domain::project_creation::{ProjectLanguage, ToolchainInfo};
use anyhow::{Context as _, Result, bail, ensure};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

fn resolve(language: ProjectLanguage, raw: &str) -> Result<PathBuf> {
    let raw = raw.trim().trim_matches('"');
    let name = if raw.is_empty() {
        language.command()
    } else {
        raw
    };
    let path = PathBuf::from(name);
    let binary = if cfg!(windows) {
        format!("{}.exe", language.command())
    } else {
        language.command().into()
    };
    if path.is_dir() {
        return resolve(language, &path.join(binary).to_string_lossy());
    }
    if path.is_absolute() || path.components().count() > 1 {
        ensure!(path.is_file(), "工具链文件不存在：{}", path.display());
        return if path.is_absolute() {
            Ok(path)
        } else {
            path.canonicalize().context("无法访问工具链")
        };
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&paths) {
            for suffix in if cfg!(windows) {
                &[".exe", ""][..]
            } else {
                &[""][..]
            } {
                let candidate = directory.join(format!("{name}{suffix}"));
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }
    bail!("未找到 {name}，请选择工具链可执行文件或安装后加入 PATH")
}

pub(super) async fn run(
    executable: &Path,
    args: &[&str],
    root: Option<&Path>,
    environment: &[(String, String)],
) -> Result<String> {
    let mut command = tokio::process::Command::new(executable);
    crate::infrastructure::process_environment::sanitize(command.as_std_mut());
    command
        .args(args)
        .envs(environment.iter().map(|(key, value)| (key, value)))
        .stdin(Stdio::null())
        .kill_on_drop(true);
    if let Some(root) = root {
        command.current_dir(root);
    }
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let output = tokio::time::timeout(Duration::from_secs(60), command.output())
        .await
        .context("工具链执行超时（60 秒）")?
        .with_context(|| format!("无法启动 {}", executable.display()))?;
    ensure!(
        output.status.success(),
        "{}：{}{}",
        executable.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    Ok(String::from_utf8_lossy(text).trim().to_owned())
}

pub(super) async fn probe(language: ProjectLanguage, raw: String) -> Result<ToolchainInfo> {
    let executable = tokio::task::spawn_blocking(move || resolve(language, &raw)).await??;
    let args = if language == ProjectLanguage::Go {
        &["version"][..]
    } else {
        &["--version"][..]
    };
    let version = run(&executable, args, None, &[]).await?;
    let standard_library = if language == ProjectLanguage::Rust {
        let compiler = executable.with_file_name(if cfg!(windows) { "rustc.exe" } else { "rustc" });
        match run(&compiler, &["--print", "sysroot"], None, &[]).await {
            Ok(root) => {
                let path = PathBuf::from(root).join("lib/rustlib/src/rust");
                tokio::task::spawn_blocking(move || path.is_dir().then_some(path)).await?
            }
            Err(_) => None,
        }
    } else {
        None
    };
    Ok(ToolchainInfo {
        executable,
        version,
        standard_library,
    })
}
