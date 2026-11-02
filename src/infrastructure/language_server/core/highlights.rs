use super::*;

impl Session {
    pub(in crate::infrastructure::language_server) async fn document_highlights(
        &mut self,
        path: &Path,
        text: String,
        position: Position,
        language: &str,
    ) -> Result<Vec<lsp_types::DocumentHighlight>> {
        if !self.highlight_supported {
            return Ok(Vec::new());
        }
        self.sync_document(path.to_owned(), text, language).await?;
        let result = self
            .request(
                "textDocument/documentHighlight",
                json!({
                    "textDocument": {"uri": uri(path)?}, "position": position
                }),
            )
            .await?;
        if result.is_null() {
            return Ok(Vec::new());
        }
        serde_json::from_value(result).context("无法解析 LSP 变量高亮结果")
    }
}
