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
    ) -> Result<ContentSearchDataContextSnapshot> {
        let started = std::time::Instant::now();
        tracing::info!(request, query_len = query.len(), "content search started");
        if query.len() > 1024 {
            bail!("搜索文字过长");
        }
        let _permit = self.reads.acquire().await?;
        match INFRASTRUCTURE
            .filesystem
            .search_content(DATA_CONTEXT.project_root().to_owned(), query)
            .await
        {
            Ok(mut snapshot) => {
                snapshot.request = request;
                tracing::info!(
                    request,
                    matches = snapshot.matches.len(),
                    truncated = snapshot.truncated,
                    elapsed_ms = started.elapsed().as_millis(),
                    "content search completed"
                );
                DATA_CONTEXT.project.set_content_search(snapshot.clone());
                Ok(snapshot)
            }
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
