use super::LanguageServerContext;
use crate::domain::language_server::{
    LanguageServerDataContextSnapshot, LanguageServerInfo, LanguageServerStatus,
};
use std::sync::Arc;

impl LanguageServerContext {
    pub(crate) fn snapshot(&self) -> Arc<LanguageServerDataContextSnapshot> {
        self.snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn publish_servers(&self, servers: Vec<LanguageServerInfo>) {
        self.replace(|next| next.servers = servers.into());
    }

    pub(crate) fn publish_status(&self, status: LanguageServerStatus) {
        self.replace(|next| next.status = status);
    }
}
