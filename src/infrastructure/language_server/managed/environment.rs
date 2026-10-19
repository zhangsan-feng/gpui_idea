use super::{InstallContext, Installed, compatible, core, update};
use anyhow::{Context, Result};
use std::{fs, io::Read, path::PathBuf, process::Command};

fn command(program: &std::path::Path) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}

pub(in crate::infrastructure::language_server) fn go_server(
    context: &InstallContext,
) -> Result<PathBuf> {
    let go = super::super::launch::executable("go")
        .context("未找到 Go 环境，请安装 Go SDK 并将 go 加入 PATH 后重试；程序会自动安装 gopls")?;
    let binary = update::ensure(
        "gopls",
        "gopls",
        context,
        || match &context.requested_release {
            Some(version) => Ok(version.clone()),
            None => compatible::go(&compatible::runtime(&go, "version")?, context),
        },
        |stage, version| {
            context.progress("正在使用本机 Go 安装 gopls…")?;
            fs::create_dir_all(stage.join("tmp"))?;
            let mut install = command(&go);
            install
                .args(["install", &format!("golang.org/x/tools/gopls@{version}")])
                .env("GOBIN", stage.join("bin"))
                .env("GOPATH", core::root().join("go-path"))
                .env("GOMODCACHE", core::root().join("go-mod-cache"))
                .env("GOCACHE", core::root().join("go-build-cache"))
                .env("GOTMPDIR", stage.join("tmp"))
                // Never let go install implicitly fetch another Go SDK.
                .env("GOTOOLCHAIN", "local")
                .env("GOWORK", "off")
                .env("GO111MODULE", "on")
                .env("GOFLAGS", "")
                .current_dir(stage);
            core::run(install, context, "Go 安装 gopls")
                .context("请确认本机 Go SDK 满足最新版 gopls 要求；版本过旧时请升级 Go 后重试")?;
            Ok(Installed {
                entry: if cfg!(windows) {
                    "bin/gopls.exe"
                } else {
                    "bin/gopls"
                }
                .into(),
                version: version.into(),
                source: format!("go:golang.org/x/tools/gopls@{version}"),
                selection: None,
            })
        },
    )?;
    Ok(binary.path)
}

pub(in crate::infrastructure::language_server) fn java_runtime() -> Result<(PathBuf, u32)> {
    let java = std::env::var_os("JAVA_HOME")
        .map(|home| PathBuf::from(home).join(if cfg!(windows) { "bin/java.exe" } else { "bin/java" }))
        .filter(|path| path.is_file())
        .or_else(|| super::super::launch::executable("java"))
        .context("未找到 Java 环境，请安装 JDK 11+ 并配置 JAVA_HOME 或 PATH 后重试；程序会选择兼容的 JDT LS")?;
    let output = command(&java)
        .arg("-version")
        .output()
        .context("无法运行 Java，请检查 JAVA_HOME 或 PATH")?;
    let version = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let major = version
        .split('"')
        .nth(1)
        .and_then(|version| version.split('.').next())
        .and_then(|major| major.parse::<u32>().ok());
    anyhow::ensure!(
        output.status.success() && major.is_some_and(|major| major >= 11),
        "Java 环境不兼容，请安装或升级到 JDK 11+ 后重试。当前：{}",
        version.lines().next().unwrap_or("无法识别 Java 版本")
    );
    Ok((java, major.unwrap()))
}

pub(in crate::infrastructure::language_server) fn java_requirement(
    home: &std::path::Path,
) -> Result<u32> {
    // The managed installation is immutable. Cache this scan next to its jars so
    // subsequent starts do not reopen every Eclipse dependency.
    let metadata = home.join("java-runtime-v3.json");
    if let Some(required) = fs::read(&metadata)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<u32>(&bytes).ok())
        .filter(|required| *required >= 11)
    {
        return Ok(required);
    }
    let mut required = 11;
    for plugin in fs::read_dir(home.join("plugins"))? {
        let plugin = plugin?.path();
        // javac is an optional compiler backend; its newer JVM does not raise
        // the minimum for the default ECJ backend. Other platform dependencies
        // (including core.filesystem) must satisfy the installed JVM.
        let name = plugin.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with("org.eclipse.jdt.core.javac_") {
            continue;
        }
        if plugin
            .extension()
            .is_none_or(|extension| extension != "jar")
        {
            continue;
        }
        let mut archive = zip::ZipArchive::new(fs::File::open(&plugin)?)
            .with_context(|| format!("无法读取 Java 服务器依赖：{}", plugin.display()))?;
        let mut manifest = String::new();
        archive
            .by_name("META-INF/MANIFEST.MF")?
            .take(2 * 1024 * 1024)
            .read_to_string(&mut manifest)?;
        let manifest = manifest.replace("\r\n ", "").replace("\n ", "");
        for line in manifest.lines() {
            let marker = if line.starts_with("Bundle-RequiredExecutionEnvironment:") {
                "JavaSE-"
            } else if line.starts_with("Require-Capability:") && line.contains("osgi.ee=JavaSE") {
                "version="
            } else {
                continue;
            };
            for value in line.split(marker).skip(1) {
                let major: String = value.chars().take_while(char::is_ascii_digit).collect();
                if let Ok(major) = major.parse::<u32>() {
                    required = required.max(major);
                }
            }
        }
    }
    if let Err(error) = fs::write(metadata, serde_json::to_vec(&required)?) {
        tracing::debug!(%error, "Could not cache JDT LS Java runtime requirement");
    }
    Ok(required)
}
