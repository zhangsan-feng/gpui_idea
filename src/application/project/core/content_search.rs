use super::super::ProjectService;
use crate::{
    data_context::DATA_CONTEXT, domain::project::ContentSearchDataContextSnapshot,
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Result, bail};

impl ProjectService {
    pub(in crate::application::project) async fn search_content_core(
        &self,
        query: String,
        request: u64,
        mut cancel: tokio::sync::watch::Receiver<()>,
    ) -> Result<Option<ContentSearchDataContextSnapshot>> {
        let started = std::time::Instant::now();
        tracing::debug!(request, query_len = query.len(), "content search started");
        if query.len() > 1024 {
            bail!("搜索文字过长");
        }
        let _permit = tokio::select! {
            biased;
            _ = cancel.changed() => {
                tracing::debug!(request, "content search cancelled before scanning");
                return Ok(None);
            }
            permit = self.reads.acquire() => permit?,
        };
        let result = INFRASTRUCTURE
            .filesystem
            .search_content(
                DATA_CONTEXT.project_root().to_owned(),
                query,
                cancel.clone(),
            )
            .await;
        if cancel.has_changed().is_err() {
            tracing::debug!(
                request,
                elapsed_ms = started.elapsed().as_millis(),
                "content search cancelled"
            );
            return Ok(None);
        }
        match result {
            Ok(Some(mut snapshot)) => {
                snapshot.request = request;
                tracing::debug!(
                    request,
                    matches = snapshot.matches.len(),
                    truncated = snapshot.truncated,
                    elapsed_ms = started.elapsed().as_millis(),
                    "content search completed"
                );
                DATA_CONTEXT.project.set_content_search(snapshot.clone());
                Ok(Some(snapshot))
            }
            Ok(None) => Ok(None),
            Err(error) => {
                DATA_CONTEXT
                    .project
                    .set_content_search(ContentSearchDataContextSnapshot {
                        request,
                        error: Some(format!("{error:#}")),
                        ..Default::default()
                    });
                Err(error)
            }
        }
    }
}
