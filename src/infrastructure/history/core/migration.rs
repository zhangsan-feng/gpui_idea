use super::*;
use serde::Deserialize;
use std::io::Write as _;

#[derive(Deserialize)]
struct LegacyHistory {
    entries: Vec<OpenHistoryEntry>,
    session: Option<WorkspaceSession>,
    #[serde(default)]
    sessions: Vec<WorkspaceSession>,
}

pub(super) fn migrate(path: &Path, mut value: serde_json::Value) -> Result<HistoryIndex> {
    let migrate_options = |session: &mut serde_json::Value| {
        if let Some(object) = session.as_object_mut() {
            if !object.contains_key("run_automatic") {
                let command = object
                    .get("run_command")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .trim();
                let automatic = command.is_empty() || command == "cargo run";
                object.insert("run_automatic".into(), serde_json::Value::Bool(automatic));
            }
        }
    };
    if let Some(session) = value.get_mut("session") {
        migrate_options(session);
    }
    if let Some(sessions) = value
        .get_mut("sessions")
        .and_then(|value| value.as_array_mut())
    {
        for session in sessions {
            migrate_options(session);
        }
    }
    let legacy: LegacyHistory =
        serde_json::from_value(value).context("旧历史格式无效，原文件已保留")?;
    let last_project = legacy
        .session
        .as_ref()
        .map(|session| session.project_root.clone());
    let mut sessions: Vec<WorkspaceSession> = Vec::new();
    for session in legacy.session.into_iter().chain(legacy.sessions) {
        if let Some(previous) = sessions
            .iter_mut()
            .find(|saved| same_path(&saved.project_root, &session.project_root))
        {
            if session.revision > previous.revision {
                *previous = session;
            }
        } else {
            sessions.push(session);
        }
    }
    sessions.truncate(LIMIT_SESSIONS);
    let mut index = HistoryIndex {
        entries: legacy.entries,
        last_project,
        ..Default::default()
    };
    normalize_entries(&mut index.entries);
    // Keep the complete original file before committing the new index.
    let backup = path.with_extension("v1-backup.json");
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)
    {
        Ok(mut file) => {
            file.write_all(&fs::read(path)?)?;
            file.sync_all()?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).context("无法备份旧打开历史"),
    }
    for session in sessions {
        let reference = WorkspaceReference {
            project_root: session.project_root.clone(),
            file: storage::session_name(&session.project_root),
        };
        // A retry after an interrupted migration must preserve a newer session.
        if !storage::read_session(path, &reference)?
            .is_some_and(|saved| saved.revision > session.revision)
        {
            storage::write_session(path, &reference, &session)?;
        }
        index.workspaces.push(reference);
    }
    storage::write_index(path, &index)?;
    tracing::info!(projects = index.entries.len(), workspaces = index.workspaces.len(), backup = %backup.display(), "opening history migrated to project index and separate workspace states");
    Ok(index)
}
