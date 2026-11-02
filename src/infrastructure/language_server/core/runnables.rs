use super::*;

impl Session {
    pub(in crate::infrastructure::language_server) fn run_supported(&self) -> bool {
        self.file_run_supported
    }

    pub(in crate::infrastructure::language_server) async fn file_runnables(
        &mut self,
        method: &str,
        parameters: fn(&str) -> Value,
        path: PathBuf,
        text: Option<String>,
        language: &str,
    ) -> Result<Value> {
        if let Some(text) = text {
            self.sync_document(path.clone(), text, language).await?;
        }
        self.wait_for_index().await?;
        let params = parameters(&uri(&path)?);
        let mut attempts = 0;
        loop {
            match self.request(method, params.clone()).await {
                Err(error)
                    if error
                        .downcast_ref::<LspResponseError>()
                        .is_some_and(|error| matches!(error.code, -32800 | -32801 | -32802))
                        && attempts < 2 =>
                {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                result => return result,
            }
        }
    }
}
