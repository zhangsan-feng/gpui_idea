use super::{ProxyStore, core};
use crate::domain::settings::ProxySettings;
use anyhow::Result;

impl ProxyStore {
    pub(crate) fn validate(&self, settings: &ProxySettings) -> Result<()> {
        core::validate(settings)
    }

    pub(crate) fn set_lsp(&self, settings: ProxySettings) {
        self.set_lsp_core(settings);
    }

    pub(crate) fn lsp_http_client(
        &self,
        builder: reqwest::blocking::ClientBuilder,
    ) -> Result<reqwest::blocking::ClientBuilder> {
        self.lsp_http_client_core(builder)
    }

    pub(crate) fn configure_lsp_command(&self, command: &mut std::process::Command) {
        self.configure_lsp_command_core(command);
    }
}
