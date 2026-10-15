use super::launch;
use crate::domain::language_server::LanguageServerStatus;
use anyhow::{Context, Result, bail};
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

fn configuration(section: &str) -> Value {
    match section {
        "rust-analyzer" => json!({"cargo": {"sysroot": "discover"}, "checkOnSave": false}),
        _ => Value::Null,
    }
}

async fn answer_server_request(input: &Input, message: &Value) -> Result<()> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let result = match method {
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
    pub(super) async fn start(
        root: &Path,
        family: &str,
        status: watch::Sender<LanguageServerStatus>,
    ) -> Result<Self> {
        // PATH lookup and launcher inspection perform filesystem IO off the GUI thread.
        let root_path = root.to_owned();
        let family_name = family.to_owned();
        let mut child =
            tokio::task::spawn_blocking(move || launch::spawn(&root_path, &family_name)).await??;
        let mut stdin = child.stdin.take().context("语言服务器 stdin 不可用")?;
        let (input, mut writes) = mpsc::unbounded_channel::<(Value, oneshot::Sender<Result<()>>)>();
        let writer = tokio::spawn(async move {
            while let Some((message, response)) = writes.recv().await {
                let result = write_message(&mut stdin, message).await;
                let failed = result.is_err();
                let _ = response.send(result);
                if failed {
                    break;
                }
            }
        });
        let stdout = child.stdout.take().context("语言服务器 stdout 不可用")?;
        let stderr_pipe = child.stderr.take().context("语言服务器 stderr 不可用")?;
        let (sender, messages) = mpsc::channel(32);
        let (status_sender, readiness) = watch::channel(Readiness::default());
        let reader_input = input.clone();
        let reader_status = status.clone();
        let warming = Arc::new(AtomicBool::new(false));
        let reader_warming = warming.clone();
        let reader = tokio::spawn(async move {
            let mut stdout = BufReader::new(stdout);
            let mut progress = HashSet::new();
            let mut health_status = None;
            loop {
                let result = read_message(&mut stdout).await;
                match result {
                    Ok(message) if message.get("method").is_some() => {
                        if message.get("id").is_some() {
                            if let Err(error) = answer_server_request(&reader_input, &message).await
                            {
                                reader_status
                                    .send_replace(LanguageServerStatus::Error(error.to_string()));
                                let _ = sender.send(Err(error)).await;
                                break;
                            }
                        } else if message.get("method").and_then(Value::as_str)
                            == Some("experimental/serverStatus")
                        {
                            let health = message
                                .pointer("/params/health")
                                .and_then(Value::as_str)
                                .unwrap_or("unknown");
                            let quiescent = message
                                .pointer("/params/quiescent")
                                .and_then(Value::as_bool);
                            let detail = message
                                .pointer("/params/message")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            tracing::info!(
                                health,
                                ?quiescent,
                                message = detail,
                                "Language server status"
                            );
                            status_sender.send_modify(|status| status.quiescent = quiescent);
                            health_status = match health {
                                "error" => Some(LanguageServerStatus::Error(detail.to_owned())),
                                "warning" => Some(LanguageServerStatus::Warning(detail.to_owned())),
                                _ => None,
                            };
                        } else if message.get("method").and_then(Value::as_str)
                            == Some("$/progress")
                        {
                            let token = message
                                .pointer("/params/token")
                                .map(Value::to_string)
                                .unwrap_or_default();
                            let kind = message
                                .pointer("/params/value/kind")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            if kind == "begin" {
                                progress.insert(token.clone());
                            }
                            if kind == "end" {
                                progress.remove(&token);
                            }
                            if matches!(kind, "begin" | "end") {
                                tracing::info!(token, kind, progress = %message["params"]["value"], active = progress.len(), "Language server indexing progress");
                            }
                            status_sender.send_modify(|status| status.progress = progress.len());
                        } else if matches!(
                            message.get("method").and_then(Value::as_str),
                            Some("window/logMessage" | "window/showMessage")
                        ) {
                            if let Some(text) =
                                message.pointer("/params/message").and_then(Value::as_str)
                            {
                                tracing::debug!(message = text, "LSP server notification");
                            }
                        }
                        let state = *status_sender.borrow();
                        let next = health_status.clone().unwrap_or_else(|| {
                            if state.busy() || state.quiescent.is_none() {
                                LanguageServerStatus::Indexing
                            } else if reader_warming.load(Ordering::Acquire) {
                                LanguageServerStatus::Warming
                            } else {
                                LanguageServerStatus::Ready
                            }
                        });
                        reader_status.send_if_modified(|current| {
                            if *current == next {
                                false
                            } else {
                                *current = next;
                                true
                            }
                        });
                    }
                    Ok(message) => {
                        if sender.send(Ok(message)).await.is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        reader_status.send_replace(LanguageServerStatus::Error(error.to_string()));
                        let _ = sender.send(Err(error)).await;
                        break;
                    }
                }
            }
        });
        let stderr_family = family.to_owned();
        let stderr = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr_pipe);
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(length) => {
                        tracing::debug!(language = stderr_family, message = %String::from_utf8_lossy(&buffer[..length]), "Language server stderr")
                    }
                }
            }
        });
        let mut session = Self {
            child,
            input,
            messages,
            reader,
            writer,
            stderr,
            sequence: 0,
            sync_kind: 1,
            documents: HashMap::new(),
            readiness,
            rust: family == "rust",
            diagnosed: false,
            status,
            warming,
            warmed: HashSet::new(),
        };
        let root_uri = uri(root)?;
        let initialization_options = if family == "rust" {
            json!({"cargo":{"sysroot":"discover"},"checkOnSave":false})
        } else {
            json!({})
        };
        let initialized = session.request("initialize", json!({
            "processId": std::process::id(), "clientInfo": {"name":"gpui-idea"},
            "rootUri": root_uri, "workspaceFolders":[{"uri":root_uri,"name":root.file_name().unwrap_or_default().to_string_lossy()}],
            "capabilities": {
                "general":{"positionEncodings":["utf-16"]},
                "workspace":{"configuration":true,"workspaceFolders":true},
                "textDocument":{
                    "definition":{"linkSupport":true},"synchronization":{"didSave":true},
                    "semanticTokens":{
                        "requests":{"full":true},"formats":["relative"],
                        "tokenTypes":["namespace","type","struct","variable","function","method","macro","keyword","comment","string","number","operator"],
                        "tokenModifiers":[]
                    }
                },
                "window":{"workDoneProgress":true},
                "experimental":{"serverStatusNotification":true}
            },
            "initializationOptions": initialization_options
        })).await;
        let capabilities = match initialized {
            Ok(value) => value,
            Err(error) => {
                session.stop().await;
                return Err(error.context(format!("初始化 {family} 语言服务失败")));
            }
        };
        if let Some(encoding) = capabilities
            .pointer("/capabilities/positionEncoding")
            .and_then(Value::as_str)
        {
            if encoding != "utf-16" {
                session.stop().await;
                bail!("语言服务器未协商 UTF-16 位置编码：{encoding}");
            }
        }
        session.sync_kind = capabilities
            .pointer("/capabilities/textDocumentSync/change")
            .or_else(|| capabilities.pointer("/capabilities/textDocumentSync"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        session.notify("initialized", json!({})).await?;
        Ok(session)
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
            tracing::info!("Waiting for language server indexing before definition");
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

    pub(super) async fn warm_document(&mut self, path: PathBuf, text: String) -> Result<()> {
        // A queued open may carry an older snapshot than an intervening click.
        // Never roll back a document already synchronized by a definition request.
        if self
            .documents
            .get(&path)
            .is_some_and(|(_, previous)| previous != &text)
        {
            return Ok(());
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
            tracing::info!(path = %path.display(), analysis_ms = analysis_started.elapsed().as_millis(),
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
        let started = std::time::Instant::now();
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
                json!({"textDocument":{"uri":uri(&closed_path)?}}),
            )
            .await?;
        }
        for (overlay_path, overlay_text) in overlays {
            let overlay_language = overlay_language(&overlay_path);
            if overlay_path != path
                && launch::family(overlay_language).ok() == launch::family(language).ok()
            {
                self.sync_document(overlay_path, overlay_text, overlay_language)
                    .await?;
            }
        }
        self.sync_document(path.to_owned(), text, language).await?;
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
                    tracing::info!(attempt = transient_attempts, %error, "Retrying transient language server definition response");
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
        tracing::info!(path = %path.display(), sync_ms, index_wait_ms = ready_ms - sync_ms,
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
                    tracing::info!(status = %status.to_string().chars().take(6000).collect::<String>(), path = %path.display(), ?position, "Rust definition empty after indexing; analyzer diagnostics")
                }
                other => tracing::info!(?other, "Rust analyzer diagnostic unavailable"),
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

fn position_at(text: &str, byte: usize) -> Position {
    let prefix = &text[..byte];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    Position::new(line, prefix[line_start..].encode_utf16().count() as u32)
}

fn incremental_change(previous: &str, current: &str) -> Value {
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

impl Drop for Session {
    fn drop(&mut self) {
        self.reader.abort();
        self.writer.abort();
        self.stderr.abort();
        let _ = self.child.start_kill();
    }
}

fn overlay_language(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "rs" => "rust",
        _ => "",
    }
}
