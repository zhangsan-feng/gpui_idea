use super::super::inventory;
use super::*;

impl Session {
    pub(in crate::infrastructure::language_server) async fn start(
        root: &Path,
        family: &str,
        status: watch::Sender<LanguageServerStatus>,
        inventory: watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
        stopping: Arc<AtomicBool>,
    ) -> Result<Self> {
        // PATH lookup and launcher inspection perform filesystem IO off the GUI thread.
        let root_path = root.to_owned();
        let family_name = family.to_owned();
        let mut registration = inventory::Registration::new(inventory.clone(), root, family, None);
        let server_id = registration.id;
        let preparation = super::super::managed::InstallContext::new(
            status.clone(),
            stopping,
            inventory.clone(),
            server_id,
        );
        let prepared = tokio::task::spawn_blocking(move || {
            launch::spawn(&root_path, &family_name, &preparation)
        })
        .await
        .context("语言服务准备任务失败")
        .and_then(|result| result);
        let (mut child, _installed_version, version_program) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                registration.failed(&error);
                return Err(error);
            }
        };
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
        let (status_sender, readiness) = watch::channel(Readiness {
            quiescent: (family != "rust").then_some(true),
            progress: 0,
        });
        let reader_input = input.clone();
        let reader_status = status.clone();
        let warming = Arc::new(AtomicBool::new(false));
        let reader_warming = warming.clone();
        let root_uri = uri(root)?;
        let workspace_folders = json!([{"uri": root_uri, "name": root.file_name().unwrap_or_default().to_string_lossy()}]);
        let reader_workspace_folders = workspace_folders.clone();
        let reader = tokio::spawn(async move {
            let _registration = registration;
            let mut stdout = BufReader::new(stdout);
            let mut progress = HashSet::new();
            let mut health_status = None;
            loop {
                let result = read_message(&mut stdout).await;
                match result {
                    Ok(message) if message.get("method").is_some() => {
                        if message.get("id").is_some() {
                            if let Err(error) = answer_server_request(
                                &reader_input,
                                &message,
                                &reader_workspace_folders,
                            )
                            .await
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
                            tracing::debug!(
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
                                tracing::debug!(token, kind, progress = %message["params"]["value"], active = progress.len(), "Language server indexing progress");
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
            completion_supported: false,
            definition_supported: false,
            documents: HashMap::new(),
            readiness,
            rust: family == "rust",
            diagnosed: false,
            status,
            warming,
            warmed: HashSet::new(),
        };
        let initialization_options = configuration::initialization_options(family);
        let initialized = session.request("initialize", json!({
            "processId": std::process::id(), "clientInfo": {"name":"gpui-idea"},
            "rootUri": root_uri, "workspaceFolders": workspace_folders,
            "capabilities": {
                "general":{"positionEncodings":["utf-16"]},
                "workspace":{"configuration":true,"workspaceFolders":true},
                "textDocument":{
                    "completion":{"completionItem":{"snippetSupport":matches!(family, "html" | "css" | "json"),"insertReplaceSupport":true}},
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
        let supports = |name: &str| {
            capabilities["capabilities"]
                .get(name)
                .is_some_and(|value| value == &Value::Bool(true) || value.is_object())
        };
        session.completion_supported = supports("completionProvider");
        session.definition_supported = supports("definitionProvider");
        tracing::debug!(
            family,
            completion = session.completion_supported,
            definition = session.definition_supported,
            sync_kind = session.sync_kind,
            "LSP capabilities negotiated"
        );
        session.notify("initialized", json!({})).await?;
        if family != "rust" {
            session
                .notify(
                    "workspace/didChangeConfiguration",
                    json!({"settings": configuration::settings()}),
                )
                .await?;
        }
        if family != "rust" {
            session.status.send_replace(LanguageServerStatus::Ready);
        }
        inventory::initialized(&inventory, server_id, &capabilities);
        if capabilities
            .pointer("/serverInfo/version")
            .and_then(Value::as_str)
            .is_none_or(|version| version.trim().is_empty())
        {
            if let Some(program) = version_program {
                let argument = if family == "go" {
                    "version"
                } else {
                    "--version"
                };
                tokio::spawn(async move {
                    if let Some(version) = launch::native_version(program, argument).await {
                        inventory.send_modify(|servers| {
                            if let Some(server) =
                                servers.iter_mut().find(|server| server.id == server_id)
                            {
                                server.version = Some(version);
                            }
                        });
                    }
                });
            }
        }
        Ok(session)
    }
}
