use super::LanguageServers;
use crate::domain::language_server::{ManagedServer, ServerManagementAction};
use anyhow::Result;

impl LanguageServers {
    pub(crate) async fn manage(&self, action: ServerManagementAction) -> Result<()> {
        // Freeze session creation/restarts while checking usage and removing a version.
        let sessions = if matches!(&action, ServerManagementAction::Delete { .. }) {
            Some(self.sessions.lock().await)
        } else {
            None
        };
        let cells = sessions
            .as_ref()
            .map(|sessions| {
                sessions
                    .iter()
                    .filter_map(|((_, family), cell)| {
                        let ServerManagementAction::Delete { key, .. } = &action else {
                            return None;
                        };
                        super::languages::get(family)
                            .ok()
                            .filter(|language| language.key == key)
                            .map(|_| cell.clone())
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut session_guards = Vec::new();
        for cell in &cells {
            session_guards.push(cell.session.lock().await);
        }
        let inventory = self.inventory.clone();
        tokio::task::spawn_blocking(move || {
            use super::managed::management;
            match action {
                ServerManagementAction::Refresh => {}
                ServerManagementAction::Delete { key, directory } => {
                    management::remove_version(&key, &directory, &inventory)?;
                }
                ServerManagementAction::CleanHistory => management::clean_history()?,
                ServerManagementAction::Select { key, directory } => {
                    management::select(&key, directory)?
                }
                ServerManagementAction::Update(key) => {
                    tracing::info!(key, "Checking managed LSP release from settings");
                    management::install_latest(&key, &management::install_context(None)?)?;
                }
            }
            Ok(())
        })
        .await?
    }

    pub(crate) async fn managed_servers(&self) -> Result<Vec<ManagedServer>> {
        let inventory = self.inventory.clone();
        tokio::task::spawn_blocking(move || {
            let mut servers = super::managed::management::snapshot()?;
            let running = inventory.borrow().clone();
            for server in &mut servers {
                for version in &mut server.versions {
                    version.in_use = super::managed::management::version_in_use(
                        &super::managed::core::root().join(&version.directory),
                        &running,
                    );
                }
            }
            Ok(servers)
        })
        .await?
    }
}
