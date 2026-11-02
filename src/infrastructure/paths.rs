use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// Stable identity shared by project state and language-server caches.
pub(crate) fn project_key(project: &Path) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in crate::domain::history::project_identity(project).bytes() {
        hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub(crate) fn project_data_directory(data: &Path, project: &Path) -> PathBuf {
    // Use normalized spelling so Windows case and extended-path aliases agree.
    let identity = crate::domain::history::project_identity(project);
    let name = Path::new(&identity)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let name: String = name
        .chars()
        .take(64)
        .map(|character| {
            if character.is_control() || "<>:\"/\\|?*".contains(character) {
                '_'
            } else {
                character
            }
        })
        .collect();
    let name = name.trim_matches([' ', '.']);
    let name = if name.is_empty() { "project" } else { name };
    data.join("projects")
        .join(format!("{name}-{}", project_key(project)))
}

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
