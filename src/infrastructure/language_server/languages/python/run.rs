use super::super::FileRunSupport;
use crate::{
    domain::run::RunConfigurationDataContextSnapshot, infrastructure::filesystem::run_tools,
};
use anyhow::{Context, Result};
use std::path::Path;

pub(super) const SUPPORT: FileRunSupport = FileRunSupport {
    accepts_file,
    resolve,
    server: None,
};

fn accepts_file(file: &Path) -> bool {
    file.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("py") || ext.eq_ignore_ascii_case("pyw"))
}

fn resolve(root: &Path, file: &Path) -> Result<RunConfigurationDataContextSnapshot> {
    anyhow::ensure!(
        !file
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pyi")),
        "Python 类型声明文件不可运行"
    );
    Ok(run_tools::interpreted_file(
        file.parent().context("文件没有父目录")?,
        root,
        file,
        "py",
    ))
}
