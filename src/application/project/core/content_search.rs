use super::super::ProjectService;
use crate::{
    data_context::DATA_CONTEXT, domain::project::ContentSearchSnapshot,
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Result, bail};

impl ProjectService {
    pub(in crate::application::project) async fn search_content_core(
        &self,
        query: String,
        request: u64,
    ) -> Result<()> {
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
                DATA_CONTEXT.project.set_content_search(snapshot);
                Ok(())
            }
            Err(error) => {
                DATA_CONTEXT
                    .project
                    .set_content_search(ContentSearchSnapshot {
                        request,
                        error: Some(format!("{error:#}")),
                        ..Default::default()
                    });
                Err(error)
            }
        }
    }
}
