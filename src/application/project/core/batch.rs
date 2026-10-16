use super::super::ProjectService;
use super::file_tree::ensure_idle;
use crate::{
    data_context::DATA_CONTEXT,
    domain::file_tree::{
        FileTreeBatchOutcome, FileTreeOperation, FileTreeOutcome, selection_roots,
    },
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Result, ensure};
use std::path::{Component, PathBuf};

impl ProjectService {
    // Preflight the entire selection before the first mutation. A runtime IO
    // failure stops the batch and reports completed items so GUI paths stay valid.
    pub(in crate::application::project) async fn operate_entries_core(
        &self,
        sources: Vec<PathBuf>,
        target: Option<PathBuf>,
    ) -> Result<FileTreeBatchOutcome> {
        let _operations = self.file_operations.lock().await;
        let _refresh = crate::application::APPLICATION
            .file_watch
            .lock_refresh()
            .await;
        ensure!(!sources.is_empty(), "请先选择文件或目录");
        for source in &sources {
            ensure!(
                !source.as_os_str().is_empty()
                    && source
                        .components()
                        .all(|part| matches!(part, Component::Normal(_))),
                "只能操作项目内的文件或目录"
            );
        }
        let sources = selection_roots(sources);
        for source in &sources {
            ensure_idle(source)?;
        }
        INFRASTRUCTURE
            .filesystem
            .validate_entry_batch(
                DATA_CONTEXT.project_root().to_owned(),
                sources.clone(),
                target.clone(),
            )
            .await?;
        tracing::info!(
            count = sources.len(),
            ?target,
            "project batch operation requested"
        );
        let mut outcome = FileTreeBatchOutcome {
            completed: Vec::new(),
            failure: None,
        };
        for source in sources {
            let result = if let Some(target) = &target {
                self.move_unlocked(source.clone(), target.clone())
                    .await
                    .map(|destination| FileTreeOutcome::Renamed {
                        source: source.clone(),
                        destination,
                    })
            } else {
                self.file_tree_operation_unlocked(FileTreeOperation::Delete {
                    source: source.clone(),
                })
                .await
            };
            match result {
                Ok(item) => outcome.completed.push(item),
                Err(error) => {
                    tracing::warn!(path = %source.display(), %error, completed = outcome.completed.len(), "project batch operation stopped");
                    outcome.failure = Some(format!("{}：{error:#}", source.display()));
                    break;
                }
            }
        }
        Ok(outcome)
    }
}
