use super::AppRuntime;
use anyhow::{Context, Result};
use std::future::Future;

impl AppRuntime {
    pub async fn run<T: Send + 'static>(
        &self,
        operation: impl Future<Output = Result<T>> + Send + 'static,
    ) -> Result<T> {
        let task = {
            let _entered = self.handle.enter();
            tokio::spawn(operation)
        };
        task.await.context("后台任务执行失败")?
    }
}
