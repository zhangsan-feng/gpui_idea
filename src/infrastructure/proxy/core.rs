use crate::domain::settings::ProxySettings;
use anyhow::{Result, ensure};

pub(super) fn validate(settings: &ProxySettings) -> Result<()> {
    if !settings.enabled {
        return Ok(());
    }
    // Do not include the supplied address in errors: it may contain credentials.
    let url = url::Url::parse(settings.url.trim())
        .map_err(|_| anyhow::anyhow!("代理地址无效，请输入完整地址，例如 http://127.0.0.1:7890"))?;
    ensure!(
        matches!(url.scheme(), "http" | "https") && url.host_str().is_some(),
        "代理地址必须使用 HTTP 或 HTTPS 协议，并包含主机地址"
    );
    ensure!(
        matches!(url.path(), "" | "/") && url.query().is_none() && url.fragment().is_none(),
        "代理地址不能包含路径、查询参数或片段"
    );
    reqwest::Proxy::all(url.as_str())
        .map_err(|_| anyhow::anyhow!("无法配置代理，请检查代理地址"))?;
    Ok(())
}

impl super::ProxyStore {
    // Application validates before persistence, then publishes the saved value.
    pub(super) fn set_lsp_core(&self, mut settings: ProxySettings) {
        settings.url = settings.url.trim().to_owned();
        tracing::info!(
            enabled = settings.enabled,
            "LSP download proxy configuration updated"
        );
        *self.lsp.write().unwrap_or_else(|error| error.into_inner()) = settings;
    }

    pub(super) fn lsp_http_client_core(
        &self,
        builder: reqwest::blocking::ClientBuilder,
    ) -> Result<reqwest::blocking::ClientBuilder> {
        let settings = self.lsp.read().unwrap_or_else(|error| error.into_inner());
        if !settings.enabled {
            return Ok(builder);
        }
        let proxy = reqwest::Proxy::all(&settings.url)
            .map_err(|_| anyhow::anyhow!("无法配置 LSP 下载代理"))?;
        tracing::debug!("Applying configured proxy to LSP HTTP request");
        Ok(builder.no_proxy().proxy(proxy))
    }

    pub(super) fn configure_lsp_command_core(&self, command: &mut std::process::Command) {
        let settings = self.lsp.read().unwrap_or_else(|error| error.into_inner());
        if !settings.enabled {
            return;
        }
        // Only the installation child inherits these overrides, never the app,
        // terminal, or running language server. Ignore inherited bypass rules.
        for key in [
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
            "npm_config_proxy",
            "npm_config_https_proxy",
        ] {
            command.env(key, &settings.url);
        }
        for key in ["NO_PROXY", "no_proxy", "npm_config_noproxy"] {
            command.env(key, "");
        }
        tracing::debug!("Applying configured proxy to LSP installation command");
    }
}
