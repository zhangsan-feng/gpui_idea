use super::{Command, Scheduled, Work};
use std::collections::HashMap;
use tokio::{sync::mpsc, task::JoinSet, time::Instant};

pub(super) async fn run(mut commands: mpsc::UnboundedReceiver<Command>) {
    let mut scheduled = HashMap::<u64, Scheduled>::new();
    let mut running = JoinSet::new();
    let mut task_ids = HashMap::new();
    loop {
        let deadline = scheduled.values().filter_map(|job| job.deadline).min();
        tokio::select! {
            command = commands.recv() => {
                match command {
                    Some(Command::Add { id, name, delay, work }) => {
                        scheduled.insert(id, Scheduled {
                            name, deadline: Some(Instant::now() + delay), running: None, work,
                        });
                        tracing::debug!(id, name, delay_ms = delay.as_millis(), "timer task registered");
                    }
                    Some(Command::Cancel(id)) => {
                        if let Some(job) = scheduled.remove(&id) {
                            if let Some(task) = job.running { task.abort(); }
                            tracing::debug!(id, name = job.name, "timer task cancelled");
                        }
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
                let due: Vec<_> = scheduled.iter().filter_map(|(id, job)| {
                    job.deadline.filter(|deadline| *deadline <= now).map(|_| *id)
                }).collect();
                for id in due {
                    let Some(mut job) = scheduled.remove(&id) else { continue; };
                    match job.work {
                        Work::Delay(wake) => { let _ = wake.send(()); }
                        Work::Repeating { ref operation, .. } => {
                            let operation = operation.clone();
                            let handle = running.spawn(async move { operation().await });
                            task_ids.insert(handle.id(), id);
                            job.running = Some(handle);
                            job.deadline = None;
                            scheduled.insert(id, job);
                        }
                    }
                }
            }
            result = running.join_next_with_id(), if !running.is_empty() => {
                let Some(result) = result else { continue; };
                let (task_id, result) = match result {
                    Ok((id, result)) => (id, result),
                    Err(error) => (error.id(), Err(anyhow::anyhow!(error))),
                };
                let Some(id) = task_ids.remove(&task_id) else { continue; };
                let Some(job) = scheduled.get_mut(&id) else { continue; };
                if let Err(error) = result {
                    tracing::warn!(id, name = job.name, %error, "timer task failed");
                }
                if let Work::Repeating { period, .. } = &job.work {
                    // Fixed delay: never overlap a task or replay missed ticks in a burst.
                    job.deadline = Some(Instant::now() + *period);
                    job.running = None;
                }
            }
        }
    }
    running.abort_all();
    while running.join_next().await.is_some() {}
    tracing::info!("timer pipeline stopped");
}
