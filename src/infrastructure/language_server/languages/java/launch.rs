use anyhow::{Context, Result};
use std::{
    hash::{Hash, Hasher},
    path::Path,
};
use tokio::process::Command;

// Run JDT LS through Java directly, avoiding platform-specific wrapper scripts.
// Its Eclipse workspace must be distinct for each project and application process.
pub(in crate::infrastructure::language_server) fn command(
    root: &Path,
    preparation: &crate::infrastructure::language_server::managed::InstallContext,
) -> Result<Command> {
    let (java, major) = super::runtime::java_runtime()?;
    let launcher =
        crate::infrastructure::language_server::managed::download_server("jdtls", preparation)?;
    let home = launcher
        .parent()
        .and_then(Path::parent)
        .context("JDT LS 缓存目录无效")?;
    let required = super::runtime::java_requirement(home)?;
    anyhow::ensure!(
        major >= required,
        "JDT LS 与本机 Java 环境不兼容：服务器要求 JDK {required}+，当前 Java {major}；请升级 JDK 并配置 JAVA_HOME 或 PATH 后重试"
    );
    let config = home.join(if cfg!(windows) {
        "config_win"
    } else if cfg!(target_os = "macos") {
        "config_mac"
    } else {
        "config_linux"
    });
    anyhow::ensure!(
        config.is_dir(),
        "JDT LS 配置目录不存在：{}",
        config.display()
    );
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    root.hash(&mut hash);
    let state = crate::infrastructure::paths::get()
        .data
        .join("lsp/jdtls")
        .join(std::process::id().to_string())
        .join(format!("{:016x}", hash.finish()));
    std::fs::create_dir_all(&state)?;
    let writable_config = state.join("config");
    std::fs::create_dir_all(&writable_config)?;
    if !writable_config.join("config.ini").is_file() {
        std::fs::copy(
            config.join("config.ini"),
            writable_config.join("config.ini"),
        )
        .context("复制 JDT LS 运行配置失败")?;
    }
    let mut command = Command::new(java);
    command
        .args([
            "-Declipse.application=org.eclipse.jdt.ls.core.id1",
            "-Dosgi.bundles.defaultStartLevel=4",
            "-Declipse.product=org.eclipse.jdt.ls.core.product",
            "-Xmx1G",
            "--add-modules=ALL-SYSTEM",
            "--add-opens",
            "java.base/java.util=ALL-UNNAMED",
            "--add-opens",
            "java.base/java.lang=ALL-UNNAMED",
        ])
        .arg("-jar")
        .arg(launcher)
        .arg("-configuration")
        .arg(writable_config)
        .arg("-data")
        .arg(state.join("workspace"))
        .env_remove("CLIENT_PORT")
        .env_remove("CLIENT_HOST")
        .env_remove("CLIENT_PIPE");
    Ok(command)
}
