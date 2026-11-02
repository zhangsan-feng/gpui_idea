use super::super::FileRunSupport;
use crate::{
    domain::run::RunConfigurationDataContextSnapshot, infrastructure::filesystem::run_tools,
};
use anyhow::Result;
use std::path::Path;

pub(super) const SUPPORT: FileRunSupport = FileRunSupport {
    accepts_file,
    resolve,
    server: None,
};

fn accepts_file(file: &Path) -> bool {
    !file.file_name().is_some_and(|name| {
        let name = name.to_string_lossy().to_ascii_lowercase();
        name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
    })
}

fn resolve(root: &Path, file: &Path) -> Result<RunConfigurationDataContextSnapshot> {
    let extension = file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    anyhow::ensure!(
        !file.file_name().is_some_and(|name| {
            let name = name.to_string_lossy().to_ascii_lowercase();
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        }),
        "TypeScript 类型声明文件不可运行"
    );
    for directory in file
        .parent()
        .into_iter()
        .flat_map(Path::ancestors)
        .take_while(|path| path.starts_with(root))
    {
        let manifest = directory.join("package.json");
        if manifest.is_file() {
            if let Ok(configuration) = run_tools::javascript_file(file, &extension, Some(&manifest))
            {
                return Ok(configuration);
            }
        }
    }
    run_tools::javascript_file(file, &extension, None)
}
