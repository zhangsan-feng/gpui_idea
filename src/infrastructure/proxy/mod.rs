mod core;
mod external;

use crate::domain::settings::ProxySettings;

pub(crate) struct ProxyStore {
    lsp: std::sync::RwLock<ProxySettings>,
}

impl ProxyStore {
    pub(super) fn new() -> Self {
        Self {
            lsp: Default::default(),
        }
    }
}
