use super::{LanguageServers, SessionSlot, core, launch};
use crate::domain::language_server::LanguageServerStatus;
use anyhow::{Context, Result, bail};
use lsp_types::{Location, Position};
use std::sync::atomic::Ordering;
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::{Mutex, watch};

impl LanguageServers {
    pub(crate) fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            status: watch::channel(LanguageServerStatus::Stopped).0,
            stopping: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn subscribe_status(&self) -> watch::Receiver<LanguageServerStatus> {
        self.status.subscribe()
    }

    pub(crate) async fn start(&self, root: PathBuf) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        let cell = sessions
            .entry((root.clone(), "rust"))
            .or_insert_with(|| {
                Arc::new(SessionSlot {
                    session: Mutex::new(None),
                    generation: watch::channel(0).0,
                })
            })
            .clone();
        drop(sessions);
        let mut session = cell.session.lock().await;
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        if session.as_mut().is_some_and(|session| session.alive()) {
            return Ok(());
        }
        if let Some(mut dead) = session.take() {
            dead.stop().await;
        }
        *session = Some(self.start_session(&root, "rust").await?);
        Ok(())
    }

    pub(crate) async fn warm_document(
        &self,
        root: PathBuf,
        path: PathBuf,
        text: String,
    ) -> Result<()> {
        self.start(root.clone()).await?;
        let cell = self
            .sessions
            .lock()
            .await
            .get(&(root, "rust"))
            .cloned()
            .context("语言服务会话不可用")?;
        let mut changes = cell.generation.subscribe();
        let mut session = tokio::select! {
            session = cell.session.lock() => session,
            _ = changes.changed() => return Ok(()),
        };
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        let operation = session
            .as_mut()
            .context("语言服务会话不可用")?
            .warm_document(path, text);
        let result = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(180), operation) => Some(result),
            _ = changes.changed() => None,
        };
        let result = match result {
            Some(Ok(result)) => result,
            interrupted => {
                if let Some(session) = session.as_mut() {
                    session.cancel_current_request().await;
                    session.finish_warmup();
                }
                if interrupted.is_none() {
                    tracing::debug!("LSP document warmup yielded to navigation or shutdown");
                    return Ok(());
                }
                Err(anyhow::anyhow!("语言服务文档预热超时"))
            }
        };
        if let Err(error) = &result {
            if error.is::<core::SessionFault>() {
                if let Some(mut failed) = session.take() {
                    failed.stop().await;
                }
                self.status
                    .send_replace(LanguageServerStatus::Error(format!("{error:#}")));
            } else {
                self.status
                    .send_replace(LanguageServerStatus::Warning(format!(
                        "文档预热失败：{error:#}"
                    )));
            }
        }
        result
    }

    async fn start_session(&self, root: &std::path::Path, family: &str) -> Result<core::Session> {
        self.status.send_replace(LanguageServerStatus::Starting);
        match core::Session::start(root, family, self.status.clone()).await {
            Ok(session) => Ok(session),
            Err(error) => {
                tracing::warn!(%error, "Language server startup failed");
                self.status
                    .send_replace(LanguageServerStatus::Error(format!("{error:#}")));
                Err(error)
            }
        }
    }

    pub(crate) async fn definitions(
        &self,
        root: PathBuf,
        path: PathBuf,
        text: String,
        position: Position,
        language: &'static str,
        overlays: Vec<(PathBuf, String)>,
        mut cancelled: watch::Receiver<bool>,
    ) -> Result<Vec<Location>> {
        if *cancelled.borrow() || cancelled.has_changed().is_err() {
            bail!("定义跳转已被取消");
        }
        let family = launch::family(language)?;
        let mut sessions = self.sessions.lock().await;
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        let cell = sessions
            .entry((root.clone(), family))
            .or_insert_with(|| {
                Arc::new(SessionSlot {
                    session: Mutex::new(None),
                    generation: watch::channel(0).0,
                })
            })
            .clone();
        // The map lock can yield: an older call may have been cancelled while
        // a newer one entered. It must not advance the newer call's generation.
        if *cancelled.borrow() || cancelled.has_changed().is_err() {
            bail!("定义跳转已被取消");
        }
        let mut changes = cell.generation.subscribe();
        let mut generation = 0;
        cell.generation.send_modify(|value| {
            *value = value.wrapping_add(1);
            generation = *value;
        });
        if *changes.borrow_and_update() != generation {
            bail!("定义跳转已被较新的请求取消");
        }
        drop(sessions);
        let mut session = tokio::select! {
            session = cell.session.lock() => session,
            _ = changes.changed() => bail!("定义跳转已被较新的请求取消"),
            _ = cancelled.changed() => bail!("定义跳转已被取消"),
        };
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        if *changes.borrow() != generation
            || *cancelled.borrow()
            || cancelled.has_changed().is_err()
        {
            bail!("定义跳转已被较新的请求取消");
        }
        if session.as_mut().is_some_and(|s| !s.alive()) {
            if let Some(mut dead) = session.take() {
                dead.stop().await;
            }
        }
        if session.is_none() {
            *session = Some(self.start_session(&root, family).await?);
        }
        if *changes.borrow() != generation
            || *cancelled.borrow()
            || cancelled.has_changed().is_err()
        {
            bail!("定义跳转已被较新的请求取消");
        }
        let operation = session
            .as_mut()
            .context("语言服务会话不可用")?
            .definitions(&path, text, position, language, overlays);
        let result = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(if family == "rust" { 180 } else { 60 }), operation) => Some(result),
            _ = changes.changed() => None,
            _ = cancelled.changed() => None,
        };
        let Some(result) = result else {
            if let Some(session) = session.as_mut() {
                session.cancel_current_request().await;
            }
            bail!("定义跳转已被较新的请求取消");
        };
        let result = match result {
            Ok(result) => result,
            Err(_) => {
                let indexing = session.as_ref().is_some_and(|session| session.indexing());
                if let Some(session) = session.as_mut() {
                    session.cancel_current_request().await;
                }
                if indexing {
                    bail!("语言服务仍在索引；已保留后台会话，请稍后重试");
                }
                if let Some(mut failed) = session.take() {
                    failed.stop().await;
                }
                self.status.send_replace(LanguageServerStatus::Error(
                    "语言服务文档同步或响应等待超时；下次请求将重新建立会话".into(),
                ));
                bail!("语言服务文档同步或响应等待超时；下次请求将重新建立会话");
            }
        };
        if let Err(error) = &result {
            let reset = error.is::<core::SessionFault>()
                || session.as_mut().is_some_and(|session| !session.alive());
            tracing::warn!(language, path = %path.display(), %error, reset, "LSP definition failed");
            if reset {
                if let Some(mut failed) = session.take() {
                    failed.stop().await;
                }
                self.status
                    .send_replace(LanguageServerStatus::Error(format!("{error:#}")));
            }
        }
        result
    }

    pub(crate) async fn shutdown(&self) {
        self.stopping.store(true, Ordering::Release);
        let sessions = std::mem::take(&mut *self.sessions.lock().await);
        for (_, cell) in sessions {
            cell.generation
                .send_modify(|value| *value = value.wrapping_add(1));
            if let Some(mut session) = cell.session.lock().await.take() {
                session.stop().await;
            }
        }
        self.status.send_replace(LanguageServerStatus::Stopped);
    }
}
