mod configuration;
mod start;
mod sync;
use super::launch;
use crate::domain::language_server::LanguageServerStatus;
use anyhow::{Context, Result, bail};
use configuration::configuration;
use lsp_types::{GotoDefinitionResponse, Location, Position};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use sync::incremental_change;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout},
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_MESSAGE: usize = 32 * 1024 * 1024;
type Input = mpsc::UnboundedSender<(Value, oneshot::Sender<Result<()>>)>;

#[derive(Debug)]
pub(super) struct LspResponseError {
    pub(super) code: i64,
    message: String,
}

impl std::fmt::Display for LspResponseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "语言服务请求错误（{}）：{}",
            self.code, self.message
        )
    }
}

impl std::error::Error for LspResponseError {}

#[derive(Debug)]
pub(super) struct SessionFault(anyhow::Error);

impl std::fmt::Display for SessionFault {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "语言服务通信失败：{}", self.0)
    }
}

impl std::error::Error for SessionFault {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

#[derive(Clone, Copy, Default)]
struct Readiness {
    quiescent: Option<bool>,
    progress: usize,
}

impl Readiness {
    fn busy(self) -> bool {
        self.quiescent == Some(false) || self.progress > 0
    }
}

pub(super) struct Session {
    child: Child,
    input: Input,
    messages: mpsc::Receiver<Result<Value>>,
    reader: JoinHandle<()>,
    writer: JoinHandle<()>,
    stderr: JoinHandle<()>,
    sequence: u64,
    sync_kind: u64,
    completion_supported: bool,
    definition_supported: bool,
    documents: HashMap<PathBuf, (i32, String)>,
    readiness: watch::Receiver<Readiness>,
    rust: bool,
    diagnosed: bool,
    status: watch::Sender<LanguageServerStatus>,
    warming: Arc<AtomicBool>,
    warmed: HashSet<PathBuf>,
}

fn uri(path: &Path) -> Result<String> {
    url::Url::from_file_path(path)
        .map(|uri| uri.to_string())
        .map_err(|_| anyhow::anyhow!("无法转换文件 URI：{}", path.display()))
}

async fn write_message(writer: &mut ChildStdin, message: Value) -> Result<()> {
    let bytes = serde_json::to_vec(&message)?;
    writer
        .write_all(format!("Content-Length: {}\r\n\r\n", bytes.len()).as_bytes())
        .await?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}

// The writer owns each whole frame: cancelling a click cannot interrupt a
// partially written JSON-RPC message or reorder it against the next click.
async fn send(input: &Input, message: Value) -> Result<()> {
    let (sent, received) = oneshot::channel();
    input
        .send((message, sent))
        .map_err(|_| anyhow::anyhow!("语言服务器写入通道已关闭"))?;
    received.await.context("语言服务器写入任务已结束")?
}

async fn read_message(reader: &mut BufReader<ChildStdout>) -> Result<Value> {
    let mut length = None;
    let mut header_size = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            bail!("语言服务器已关闭输出流");
        }
        header_size += line.len();
        if header_size > 8192 {
            bail!("语言服务器响应头过大");
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                length = Some(value.trim().parse::<usize>()?);
            }
        }
    }
    let length = length.context("语言服务器响应缺少 Content-Length")?;
    if length > MAX_MESSAGE {
        bail!("语言服务器响应超过 32MB");
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;
    Ok(serde_json::from_slice(&body)?)
}

async fn answer_server_request(
    input: &Input,
    message: &Value,
    workspace_folders: &Value,
) -> Result<()> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let result = match method {
        "workspace/workspaceFolders" => workspace_folders.clone(),
        "workspace/configuration" => {
            let values = message.pointer("/params/items").and_then(Value::as_array)
                .map(|items| items.iter().map(|item| configuration(item.get("section").and_then(Value::as_str).unwrap_or_default())).collect::<Vec<_>>())
                .unwrap_or_default();
            json!(values)
        }
        "client/registerCapability" | "client/unregisterCapability" | "window/workDoneProgress/create" => Value::Null,
        "workspace/applyEdit" => json!({"applied": false, "failureReason": "Definition navigation does not apply workspace edits"}),
        "window/showMessageRequest" => Value::Null,
        _ => return send(input, json!({"jsonrpc":"2.0", "id":id, "error":{"code":-32601,"message":"Unsupported client method"}})).await,
    };
    send(input, json!({"jsonrpc":"2.0", "id":id, "result":result})).await
}

impl Session {
    pub(super) async fn completions(
        &mut self,
        path: PathBuf,
        text: String,
        position: Position,
        language: &'static str,
        overlays: Vec<(PathBuf, String)>,
    ) -> Result<lsp_types::CompletionResponse> {
        if !self.completion_supported {
            return Ok(lsp_types::CompletionResponse::Array(vec![]));
        }
        self.sync_documents(&path, text, language, overlays).await?;
        self.wait_for_index().await?;
        let params = json!({"textDocument":{"uri":uri(&path)?},"position":position,"context":{"triggerKind":1}});
        let mut attempts = 0;
        loop {
            match self
                .request("textDocument/completion", params.clone())
                .await
            {
                Ok(value) => {
                    return if value.is_null() {
                        Ok(lsp_types::CompletionResponse::Array(vec![]))
                    } else {
                        serde_json::from_value(value).context("无法解析语义补全响应")
                    };
                }
                Err(error)
                    if error
                        .downcast_ref::<LspResponseError>()
                        .is_some_and(|error| matches!(error.code, -32800 | -32801 | -32802))
                        && attempts < 2 =>
                {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None)) && !self.reader.is_finished()
    }

    pub(super) fn indexing(&self) -> bool {
        self.rust && self.readiness.borrow().busy()
    }

    async fn wait_for_index(&mut self) -> Result<()> {
        if !self.rust {
            return Ok(());
        }
        if self.readiness.borrow().quiescent.is_none() && self.readiness.borrow().progress == 0 {
            let _ =
                tokio::time::timeout(Duration::from_millis(750), self.readiness.changed()).await;
        }
        if self.readiness.borrow().busy() {
            tracing::debug!("Waiting for language server indexing before definition");
        }
        while self.readiness.borrow().busy() {
            self.readiness
                .changed()
                .await
                .context("语言服务器索引状态通道已关闭")?;
        }
        Ok(())
    }

    async fn notify(&self, method: &str, params: Value) -> Result<()> {
        send(
            &self.input,
            json!({"jsonrpc":"2.0","method":method,"params":params}),
        )
        .await
        .map_err(|error| SessionFault(error).into())
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.sequence += 1;
        let id = self.sequence;
        let operation = async {
            send(
                &self.input,
                json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}),
            )
            .await?;
            loop {
                let response = self
                    .messages
                    .recv()
                    .await
                    .context("语言服务响应通道已关闭")??;
                if response.get("id").and_then(Value::as_u64) != Some(id) {
                    continue;
                }
                if let Some(error) = response.get("error") {
                    return Err(LspResponseError {
                        code: error
                            .get("code")
                            .and_then(Value::as_i64)
                            .context("语言服务错误响应缺少错误代码")?,
                        message: error
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("未知错误")
                            .to_owned(),
                    }
                    .into());
                }
                return Ok(response.get("result").cloned().unwrap_or(Value::Null));
            }
        };
        let result = tokio::time::timeout(REQUEST_TIMEOUT, operation)
            .await
            .with_context(|| format!("语言服务 {method} 请求超时（60秒）"))
            .and_then(|result| result);
        result.map_err(|error| {
            if error.is::<LspResponseError>() {
                error
            } else {
                SessionFault(error).into()
            }
        })
    }

    async fn sync_document(&mut self, path: PathBuf, text: String, language: &str) -> Result<()> {
        let document_uri = uri(&path)?;
        if let Some((version, previous)) = self.documents.get(&path) {
            if previous == &text {
                return Ok(());
            }
            let version = version + 1;
            if self.sync_kind == 0 {
                return Ok(());
            }
            let change = if self.sync_kind == 2 {
                incremental_change(previous, &text)
            } else {
                json!({"text":text})
            };
            self.documents.insert(path, (version, text));
            self.notify("textDocument/didChange", json!({"textDocument":{"uri":document_uri,"version":version},"contentChanges":[change]})).await?;
        } else {
            let params = json!({"textDocument":{"uri":document_uri,"languageId":language,"version":1,"text":text}});
            self.documents.insert(path, (1, text));
            self.notify("textDocument/didOpen", params).await?;
        }
        Ok(())
    }

    pub(super) async fn warm_document(
        &mut self,
        path: PathBuf,
        text: String,
        language: &'static str,
    ) -> Result<()> {
        // A queued open may carry an older snapshot than an intervening click.
        // Never roll back a document already synchronized by a definition request.
        if self
            .documents
            .get(&path)
            .is_some_and(|(_, previous)| previous != &text)
        {
            return Ok(());
        }
        if language != "rust" {
            return self.sync_document(path, text, language).await;
        }
        if self.warmed.contains(&path)
            && self
                .documents
                .get(&path)
                .is_some_and(|(_, previous)| previous == &text)
        {
            return Ok(());
        }
        let started = std::time::Instant::now();
        self.warming.store(true, Ordering::Release);
        self.status.send_replace(LanguageServerStatus::Warming);
        let result = async {
            self.sync_document(path.clone(), text, "rust").await?;
            self.wait_for_index().await?;
            let analysis_started = std::time::Instant::now();
            let params = json!({"textDocument":{"uri":uri(&path)?}});
            let mut attempts = 0;
            loop {
                match self.request("textDocument/semanticTokens/full", params.clone()).await {
                    Ok(_) => break,
                    Err(error) if error.downcast_ref::<LspResponseError>()
                        .is_some_and(|error| matches!(error.code, -32800 | -32801 | -32802))
                        && attempts < 3 => {
                        attempts += 1;
                        tokio::time::sleep(Duration::from_millis(100 * attempts)).await;
                        self.wait_for_index().await?;
                    }
                    Err(error) => return Err(error),
                }
            }
            self.warmed.insert(path.clone());
            tracing::debug!(path = %path.display(), analysis_ms = analysis_started.elapsed().as_millis(),
                elapsed_ms = started.elapsed().as_millis(), "LSP document semantic warmup completed");
            Ok(())
        }.await;
        self.finish_warmup();
        result
    }

    pub(super) fn finish_warmup(&self) {
        self.warming.store(false, Ordering::Release);
        // Server health notifications take precedence over a successful warmup.
        self.status.send_if_modified(|current| {
            if matches!(
                current,
                LanguageServerStatus::Warning(_) | LanguageServerStatus::Error(_)
            ) {
                return false;
            }
            *current = if self.indexing() {
                LanguageServerStatus::Indexing
            } else {
                LanguageServerStatus::Ready
            };
            true
        });
    }

    pub(super) async fn definitions(
        &mut self,
        path: &Path,
        text: String,
        position: Position,
        language: &str,
        overlays: Vec<(PathBuf, String)>,
    ) -> Result<Vec<Location>> {
        if !self.definition_supported {
            return Ok(Vec::new());
        }
        let started = std::time::Instant::now();
        self.sync_documents(path, text, language, overlays).await?;
        let sync_ms = started.elapsed().as_millis();
        self.wait_for_index().await?;
        let ready_ms = started.elapsed().as_millis();
        let params = json!({"textDocument":{"uri":uri(path)?},"position":position});
        let mut ready_attempts = 0;
        let mut transient_attempts = 0;
        let result = loop {
            let result = match self
                .request("textDocument/definition", params.clone())
                .await
            {
                Ok(result) => result,
                Err(error)
                    if error
                        .downcast_ref::<LspResponseError>()
                        .is_some_and(|error| matches!(error.code, -32800 | -32801 | -32802))
                        && transient_attempts < 3 =>
                {
                    transient_attempts += 1;
                    tracing::debug!(attempt = transient_attempts, %error, "Retrying transient language server definition response");
                    tokio::time::sleep(Duration::from_millis(100 * transient_attempts)).await;
                    self.wait_for_index().await?;
                    continue;
                }
                Err(error) => return Err(error),
            };
            if !result.is_null() && !result.as_array().is_some_and(Vec::is_empty) {
                break result;
            }
            if self.rust && self.readiness.borrow().busy() {
                self.wait_for_index().await?;
                continue;
            }
            ready_attempts += 1;
            if ready_attempts >= 3 {
                break result;
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        };
        tracing::debug!(path = %path.display(), sync_ms, index_wait_ms = ready_ms - sync_ms,
            request_ms = started.elapsed().as_millis() - ready_ms,
            "LSP definition request stages");
        if self.rust
            && !self.diagnosed
            && (result.is_null() || result.as_array().is_some_and(Vec::is_empty))
        {
            self.diagnosed = true;
            match tokio::time::timeout(
                Duration::from_secs(3),
                self.request("rust-analyzer/analyzerStatus", json!({})),
            )
            .await
            {
                Ok(Ok(status)) => {
                    tracing::debug!(status = %status.to_string().chars().take(6000).collect::<String>(), path = %path.display(), ?position, "Rust definition empty after indexing; analyzer diagnostics")
                }
                other => tracing::debug!(?other, "Rust analyzer diagnostic unavailable"),
            }
        }
        if result.is_null() {
            return Ok(Vec::new());
        }
        let response: GotoDefinitionResponse = serde_json::from_value(result)
            .context("语言服务器定义响应格式无效")
            .map_err(SessionFault)?;
        let locations = match response {
            GotoDefinitionResponse::Scalar(location) => vec![location],
            GotoDefinitionResponse::Array(locations) => locations,
            GotoDefinitionResponse::Link(links) => links
                .into_iter()
                .map(|link| Location {
                    uri: link.target_uri,
                    range: link.target_selection_range,
                })
                .collect(),
        };
        if let Some(location) = locations.iter().find(|location| {
            !location
                .uri
                .scheme()
                .is_some_and(|scheme| scheme.as_str() == "file")
        }) {
            bail!(
                "语言服务器返回不可直接访问的源码 URI（{}）；请配置该语言服务器的本地标准库或依赖源码",
                location.uri.as_str()
            );
        }
        Ok(locations)
    }

    pub(super) async fn cancel_current_request(&self) {
        let _ = tokio::time::timeout(
            Duration::from_millis(500),
            self.notify("$/cancelRequest", json!({"id":self.sequence})),
        )
        .await;
    }

    pub(super) async fn stop(&mut self) {
        self.warming.store(false, Ordering::Release);
        self.status.send_replace(LanguageServerStatus::Stopped);
        // Avoid waiting for indexing servers during application shutdown.
        let _ = tokio::time::timeout(
            Duration::from_millis(500),
            self.request("shutdown", Value::Null),
        )
        .await;
        let _ = tokio::time::timeout(Duration::from_millis(100), self.notify("exit", Value::Null))
            .await;
        let _ = self.child.start_kill();
        self.reader.abort();
        self.writer.abort();
        self.stderr.abort();
        let _ = tokio::time::timeout(Duration::from_secs(2), self.child.wait()).await;
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.reader.abort();
        self.writer.abort();
        self.stderr.abort();
        let _ = self.child.start_kill();
    }
}

fn overlay_language(path: &Path) -> &'static str {
    crate::domain::language_server::language_for_path(path).unwrap_or_default()
}
