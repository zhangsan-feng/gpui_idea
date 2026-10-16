use anyhow::{Context, Result};
use std::{path::PathBuf, sync::OnceLock};

pub(crate) struct AppPaths {
    pub(crate) root: PathBuf,
    pub(crate) data: PathBuf,
    pub(crate) assets: PathBuf,
    pub(crate) logs: PathBuf,
    pub(crate) tools: PathBuf,
}

static PATHS: OnceLock<AppPaths> = OnceLock::new();

pub(crate) fn initialize() -> Result<&'static AppPaths> {
    if let Some(paths) = PATHS.get() {
        return Ok(paths);
    }
    let root = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    } else {
        std::env::current_exe()
            .context("无法定位应用程序")?
            .parent()
            .context("应用程序没有父目录")?
            .to_owned()
    };
    let paths = AppPaths {
        data: root.join("data"),
        assets: root.join("assets"),
        logs: root.join("logs"),
        tools: root.join("tools"),
        root,
    };
    for directory in [&paths.data, &paths.assets, &paths.logs, &paths.tools] {
        std::fs::create_dir_all(directory).with_context(|| {
            format!(
                "无法创建应用目录 {}，请确认安装目录可写",
                directory.display()
            )
        })?;
    }
    let _ = PATHS.set(paths);
    Ok(get())
}

pub(crate) fn get() -> &'static AppPaths {
    PATHS
        .get()
        .expect("application paths must be initialized before services")
}
