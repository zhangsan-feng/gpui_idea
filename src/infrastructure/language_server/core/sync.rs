use lsp_types::Position;
use serde_json::{Value, json};

fn position_at(text: &str, byte: usize) -> Position {
    let prefix = &text[..byte];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    Position::new(line, prefix[line_start..].encode_utf16().count() as u32)
}

pub(super) fn incremental_change(previous: &str, current: &str) -> Value {
    let mut prefix = previous
        .bytes()
        .zip(current.bytes())
        .take_while(|(left, right)| left == right)
        .count();
    while !edit_boundary(previous, prefix) || !edit_boundary(current, prefix) {
        prefix -= 1;
    }
    let mut suffix = previous.as_bytes()[prefix..]
        .iter()
        .rev()
        .zip(current.as_bytes()[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    while !edit_boundary(previous, previous.len() - suffix)
        || !edit_boundary(current, current.len() - suffix)
    {
        suffix -= 1;
    }
    json!({"range":{"start":position_at(previous, prefix),"end":position_at(previous, previous.len()-suffix)},"text":&current[prefix..current.len()-suffix]})
}

fn edit_boundary(text: &str, byte: usize) -> bool {
    text.is_char_boundary(byte)
        && !(byte > 0
            && text.as_bytes().get(byte - 1) == Some(&b'\r')
            && text.as_bytes().get(byte) == Some(&b'\n'))
}

impl super::Session {
    pub(super) async fn sync_documents(
        &mut self,
        path: &std::path::Path,
        text: String,
        language: &str,
        overlays: Vec<(std::path::PathBuf, String)>,
    ) -> anyhow::Result<()> {
        let closed: Vec<_> = self
            .documents
            .keys()
            .filter(|known| {
                known.as_path() != path && !overlays.iter().any(|(overlay, _)| overlay == *known)
            })
            .cloned()
            .collect();
        for closed_path in closed {
            self.documents.remove(&closed_path);
            self.warmed.remove(&closed_path);
            self.notify(
                "textDocument/didClose",
                json!({"textDocument":{"uri":super::uri(&closed_path)?}}),
            )
            .await?;
        }
        for (overlay_path, overlay_text) in overlays {
            let overlay_language = super::overlay_language(&overlay_path);
            if overlay_path != path
                && super::launch::family(overlay_language).ok()
                    == super::launch::family(language).ok()
            {
                self.sync_document(overlay_path, overlay_text, overlay_language)
                    .await?;
            }
        }
        self.sync_document(path.to_owned(), text, language).await?;
        Ok(())
    }
}
