use super::{Scheduled, ScheduledEvent, TimerEvent, operations};
use crate::{event_bus::EVENT_BUS, infrastructure::INFRASTRUCTURE};
use std::collections::HashMap;
use tokio::{
    sync::mpsc,
    task::{AbortHandle, JoinSet},
    time::Instant,
};

pub(super) async fn run(mut events: mpsc::UnboundedReceiver<TimerEvent>) {
    let mut scheduled = HashMap::<u64, Scheduled>::new();
    let mut running = JoinSet::new();
    let mut running_tasks = HashMap::<u64, AbortHandle>::new();
    let mut task_ids = HashMap::new();
    loop {
        let deadline = scheduled.values().map(|job| job.deadline).min();
        tokio::select! {
            event = events.recv() => {
                match event {
                    Some(TimerEvent::Delay { id, name, delay, completion }) => {
                        scheduled.insert(id, Scheduled {
                            deadline: Instant::now() + delay,
                            event: ScheduledEvent::Delay { name, completion },
                        });
                        tracing::debug!(id, name, delay_ms = delay.as_millis(), "timer received Delay event");
                    }
                    Some(TimerEvent::WatchFiles { id, root, pending, period }) => {
                        tracing::debug!(id, root = %root.display(), "timer received WatchFiles event");
                        scheduled.insert(id, Scheduled {
                            deadline: Instant::now() + period,
                            event: ScheduledEvent::WatchFiles { root, pending, period },
                        });
                    }
                    Some(TimerEvent::SaveFile { id, document, completion }) => {
                        let handle = running.spawn(operations::save_file(document, completion));
                        task_ids.insert(handle.id(), id);
                        running_tasks.insert(id, handle);
                    }
                    Some(TimerEvent::ExcludeGuiFileChange { id, root, paths, completion }) => {
                        let handle = running.spawn(operations::exclude_gui_file_change(root, paths, completion));
                        task_ids.insert(handle.id(), id);
                        running_tasks.insert(id, handle);
                    }
                    Some(TimerEvent::Cancel(id)) => {
                        scheduled.remove(&id);
                        if let Some(task) = running_tasks.remove(&id) { task.abort(); }
                        tracing::debug!(id, "timer received Cancel event");
                    }
                    None => break,
                }
            }
            _ = async {
                match deadline {
                    Some(deadline) => tokio::time::sleep_until(deadline).await,
                    None => std::future::pending::<()>().await,
                }
            } => {
                let now = Instant::now();
                let due: Vec<_> = scheduled.iter()
                    .filter_map(|(id, job)| (job.deadline <= now).then_some(*id)).collect();
                for id in due {
                    let Some(job) = scheduled.remove(&id) else { continue; };
                    match job.event {
                        ScheduledEvent::Delay { name, completion } => {
                            let _ = completion.send(());
                            tracing::debug!(id, name, "timer delay completed");
                        }
                        ScheduledEvent::WatchFiles { root, pending, period } => {
                            if let Some(changes) = INFRASTRUCTURE.filesystem.take_watch_changes(&pending) {
                                EVENT_BUS.refresh_data_context(root.clone(), changes);
                            }
                            scheduled.insert(id, Scheduled {
                                deadline: Instant::now() + period,
                                event: ScheduledEvent::WatchFiles { root, pending, period },
                            });
                        }
                    }
                }
            }
            result = running.join_next_with_id(), if !running.is_empty() => {
                let Some(result) = result else { continue; };
                let task_id = match result {
                    Ok((id, ())) => id,
                    Err(error) => {
                        if !error.is_cancelled() { tracing::warn!(%error, "timer event handler failed"); }
                        error.id()
                    }
                };
                if let Some(id) = task_ids.remove(&task_id) { running_tasks.remove(&id); }
            }
        }
    }
    running.abort_all();
    while running.join_next().await.is_some() {}
    tracing::info!("timer event subscriber stopped");
}
