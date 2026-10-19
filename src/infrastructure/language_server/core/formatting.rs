use super::*;

impl Session {
    pub(in crate::infrastructure::language_server) async fn format_document(
        &mut self,
        path: &Path,
        text: String,
        language: &str,
    ) -> Result<Vec<lsp_types::TextEdit>> {
        if !self.formatting_supported {
            tracing::debug!(%language, "LSP does not support document formatting");
            return Ok(Vec::new());
        }
        self.sync_document(path.to_owned(), text, language).await?;
        self.wait_for_index().await?;
        let value = self
            .request(
                "textDocument/formatting",
                json!({
                    "textDocument": {"uri": uri(path)?},
                    "options": {"tabSize": 4, "insertSpaces": true}
                }),
            )
            .await?;
        if value.is_null() {
            return Ok(Vec::new());
        }
        serde_json::from_value(value).context("无法解析 LSP 格式化结果")
    }
}
