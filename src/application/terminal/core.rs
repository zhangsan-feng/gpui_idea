use crate::{
    data_context::DATA_CONTEXT, domain::terminal::TerminalId, infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;

pub(super) async fn start(id: TerminalId, command: Option<String>) -> Result<()> {
    let root = DATA_CONTEXT.require_project_root()?.to_owned();
    if let Some((generation, mut events)) = INFRASTRUCTURE.terminal.start(id, root, command).await?
    {
        publish(id).await?;
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                let mut batch = vec![event];
                if let Err(error) = INFRASTRUCTURE
                    .timer
                    .delay(
                        "terminal-output-batch",
                        std::time::Duration::from_millis(16),
                    )
                    .await
                {
                    tracing::debug!(?id, %error, "terminal output timer stopped");
                    break;
                }
                while let Ok(event) = events.try_recv() {
                    batch.push(event);
                }
                match INFRASTRUCTURE
                    .terminal
                    .update(id, Some(generation), batch)
                    .await
                {
                    Ok(snapshot) => {
                        let exited = snapshot.exited;
                        DATA_CONTEXT.terminal.publish(id, snapshot);
                        if exited {
                            break;
                        }
                    }
                    Err(error) => {
                        tracing::debug!(?id, %error, "terminal refresh stopped");
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}
pub(super) async fn publish(id: TerminalId) -> Result<()> {
    let snapshot = INFRASTRUCTURE.terminal.update(id, None, Vec::new()).await?;
    DATA_CONTEXT.terminal.publish(id, snapshot);
    Ok(())
}
