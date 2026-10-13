mod external;
use anyhow::{Context, Result};
use tokio::runtime::Handle;

pub struct AppRuntime {
    handle: Handle,
}
impl AppRuntime {
    pub fn from_current() -> Result<Self> {
        Ok(Self {
            handle: Handle::try_current().context("应用服务必须在 Tokio runtime 内初始化")?,
        })
    }
}
