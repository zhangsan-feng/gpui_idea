use super::{TerminalService, core};
use crate::{domain::terminal::TerminalId, infrastructure::INFRASTRUCTURE};
use anyhow::{Result, ensure};
impl TerminalService {
    pub(crate) async fn release(&self, id: TerminalId) -> Result<()> {
        ensure!(
            matches!(id, TerminalId::RunInstance(_)),
            "仅独立运行实例可以释放"
        );
        INFRASTRUCTURE.terminal.stop(id).await?;
        crate::data_context::DATA_CONTEXT.terminal.retire(id);
        INFRASTRUCTURE.terminal.release(id).await
    }
    pub(crate) async fn start(&self, id: TerminalId, command: Option<String>) -> Result<()> {
        if let Some(command) = &command {
            ensure!(
                !command.trim().is_empty() && command.len() <= 16_384,
                "运行命令不能为空或过长"
            );
        }
        core::start(id, command).await
    }
    pub(crate) async fn input(&self, id: TerminalId, bytes: Vec<u8>) -> Result<()> {
        if INFRASTRUCTURE.terminal.input(id, bytes)? {
            core::publish(id).await?;
        }
        Ok(())
    }
    pub(crate) async fn resize(
        &self,
        id: TerminalId,
        columns: usize,
        lines: usize,
        cell_width: u16,
        cell_height: u16,
    ) -> Result<()> {
        ensure!(
            (2..=1000).contains(&columns) && (1..=500).contains(&lines),
            "终端尺寸无效"
        );
        ensure!(cell_width > 0 && cell_height > 0, "终端字符尺寸无效");
        INFRASTRUCTURE
            .terminal
            .resize(id, columns, lines, cell_width, cell_height)?;
        core::publish(id).await
    }
    pub(crate) async fn scroll_to(&self, id: TerminalId, offset: usize) -> Result<()> {
        INFRASTRUCTURE.terminal.scroll_to(id, offset);
        core::publish(id).await
    }
    pub(crate) async fn scroll(&self, id: TerminalId, lines: i32) -> Result<()> {
        INFRASTRUCTURE.terminal.scroll(id, lines);
        core::publish(id).await
    }
    pub(crate) async fn stop(&self, id: TerminalId) -> Result<()> {
        INFRASTRUCTURE.terminal.stop(id).await?;
        core::publish(id).await
    }
    pub(crate) async fn clear(&self, id: TerminalId) -> Result<()> {
        INFRASTRUCTURE.terminal.clear(id);
        core::publish(id).await
    }
}
