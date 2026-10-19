use super::{TimerEvent, TimerEventBus, TimerRegistration};
use crate::{
    domain::document::DocumentSnapshot, event_bus::FileChanges, infrastructure::LocalChangeGuard,
};
use anyhow::{Context, Result, bail};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, atomic::Ordering},
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};

impl TimerEventBus {
    pub(crate) fn subscribe(&self) -> Result<mpsc::UnboundedReceiver<TimerEvent>> {
        self.receiver
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
            .context("定时事件已经被订阅")
    }

    fn register(&self, id: u64, event: TimerEvent) -> Result<TimerRegistration> {
        self.sender
            .send(event)
            .map_err(|_| anyhow::anyhow!("定时事件订阅已经关闭"))?;
        Ok(TimerRegistration {
            id,
            sender: self.sender.clone(),
        })
    }

    pub(crate) async fn delay(&self, name: &'static str, delay: Duration) -> Result<()> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (completion, receiver) = oneshot::channel();
        let _registration = self.register(
            id,
            TimerEvent::Delay {
                id,
                name,
                delay,
                completion,
            },
        )?;
        receiver.await.context("延迟事件被取消或定时器已经关闭")
    }

    pub(crate) fn watch_files(
        &self,
        root: PathBuf,
        pending: Arc<Mutex<FileChanges>>,
        period: Duration,
    ) -> Result<TimerRegistration> {
        if period.is_zero() {
            bail!("文件监听周期不能为零");
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.register(
            id,
            TimerEvent::WatchFiles {
                id,
                root,
                pending,
                period,
            },
        )
    }

    pub(crate) async fn save_file(&self, document: Arc<DocumentSnapshot>) -> Result<Arc<[u8]>> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (completion, receiver) = oneshot::channel();
        let _registration = self.register(
            id,
            TimerEvent::SaveFile {
                id,
                document,
                completion,
            },
        )?;
        receiver
            .await
            .context("文件保存事件被取消或定时器已经关闭")?
    }

    pub(crate) async fn exclude_gui_file_change(
        &self,
        root: PathBuf,
        paths: Vec<PathBuf>,
    ) -> Result<LocalChangeGuard> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (completion, receiver) = oneshot::channel();
        let _registration = self.register(
            id,
            TimerEvent::ExcludeGuiFileChange {
                id,
                root,
                paths,
                completion,
            },
        )?;
        receiver
            .await
            .context("文件修改排除事件被取消或定时器已经关闭")?
    }
}
