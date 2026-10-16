use super::{Binary, InstallContext, Installed};
use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn root() -> PathBuf {
    crate::infrastructure::paths::get().tools.join("managed")
}

pub(super) fn cached(directory: &Path) -> Option<Binary> {
    let installed: Installed =
        serde_json::from_slice(&fs::read(directory.join("installed.json")).ok()?).ok()?;
    if !installed
        .entry
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return None;
    }
    let path = directory.join(installed.entry);
    path.is_file().then_some(Binary {
        path,
        version: installed.version,
        selection: installed.selection,
    })
}

// The OS lock releases automatically on process exit, including crashes.
// Only a completed staging directory is renamed into the reusable cache.
pub(super) fn install(
    key: &str,
    label: &str,
    context: &InstallContext,
    prepare: impl FnOnce(&Path) -> Result<Installed>,
) -> Result<Binary> {
    context.check()?;
    let base = root();
    let target = base.join(key);
    context.target(target.clone());
    if let Some(binary) = cached(&target) {
        return Ok(binary);
    }
    fs::create_dir_all(&base).with_context(|| format!("无法写入工具目录 {}", base.display()))?;
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(base.join(format!("{key}.lock")))?;
    context.progress(format!("正在准备 {label}，安装目录：{}", target.display()))?;
    let wait = Instant::now();
    loop {
        context.check()?;
        match lock.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) => {
                anyhow::ensure!(
                    wait.elapsed() < Duration::from_secs(900),
                    "等待语言服务安装锁超时"
                );
                std::thread::sleep(Duration::from_millis(150));
            }
            Err(error) => return Err(error.into()),
        }
    }
    if let Some(binary) = cached(&target) {
        return Ok(binary);
    }
    let staging = base.join(format!(
        ".install-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&staging)?;
    let cleanup = Staging(staging.clone());
    let result = (|| {
        let mut installed = prepare(&staging)?;
        installed.selection = context.selected_version();
        context.check()?;
        anyhow::ensure!(
            installed
                .entry
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
                && staging.join(&installed.entry).is_file(),
            "{label} 安装后缺少入口程序"
        );
        let mut manifest = File::create(staging.join("installed.json"))?;
        manifest.write_all(&serde_json::to_vec_pretty(&installed)?)?;
        manifest.sync_all()?;
        drop(manifest);
        if target.exists() {
            let backup = base.join(format!(
                ".incomplete-{key}-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::rename(&target, backup).context("无法隔离不完整的工具缓存")?;
        }
        fs::rename(&staging, &target).context("无法提交语言服务安装目录")?;
        tracing::info!(label, version = installed.version, source = installed.source, path = %target.display(), "Managed language server installed");
        cached(&target).context("语言服务缓存校验失败")
    })();
    drop(cleanup);
    result.with_context(|| format!("准备 {label} 失败，可重试；安装目录：{}", target.display()))
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        // This path was created exclusively above, within the managed cache.
        if self.0.parent().is_some_and(|parent| parent == root()) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

pub(super) fn run(command: Command, context: &InstallContext, label: &str) -> Result<PathBuf> {
    run_output(command, context, label, false)
}

pub(super) fn capture(command: Command, context: &InstallContext, label: &str) -> Result<PathBuf> {
    run_output(command, context, label, true)
}

fn run_output(
    mut command: Command,
    context: &InstallContext,
    label: &str,
    capture: bool,
) -> Result<PathBuf> {
    context.check()?;
    let base = root();
    fs::create_dir_all(&base)?;
    let output = crate::infrastructure::paths::get().logs.join(format!(
        "lsp-install-{}-{}.log",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let log = File::create(&output)?;
    let captured = output.with_extension("stdout.json");
    command
        .stdin(Stdio::null())
        .stdout(if capture {
            File::create(&captured)?
        } else {
            log.try_clone()?
        })
        .stderr(log);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("无法运行 {label}"))?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            if !status.success() {
                let detail = failure_detail(&output);
                anyhow::bail!(
                    "{label} 执行失败（{status}）：{detail}；安装日志：{}",
                    output.display()
                );
            }
            return Ok(if capture { captured } else { output });
        }
        if context.check().is_err() || start.elapsed() > Duration::from_secs(600) {
            let _ = child.kill();
            let _ = child.wait();
            context.check()?;
            anyhow::bail!("{label} 执行超时，安装日志：{}", output.display());
        }
        std::thread::sleep(Duration::from_millis(150));
    }
}

fn failure_detail(path: &Path) -> String {
    let tail = (|| -> std::io::Result<String> {
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();
        file.seek(SeekFrom::Start(length.saturating_sub(8192)))?;
        let mut bytes = Vec::new();
        file.take(8192).read_to_end(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    })()
    .unwrap_or_default();
    if tail.contains("EBADENGINE") {
        let required = tail
            .lines()
            .find_map(|line| line.split_once("Required:").map(|(_, value)| value.trim()));
        let actual = tail
            .lines()
            .find_map(|line| line.split_once("Actual:").map(|(_, value)| value.trim()));
        return format!(
            "Node.js/npm 版本不兼容，请升级 Node.js 后重试。要求：{}；当前：{}",
            required.unwrap_or("详见安装日志"),
            actual.unwrap_or("详见安装日志")
        );
    }
    let lines = tail
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    lines[lines.len().saturating_sub(8)..].join("\n")
}
