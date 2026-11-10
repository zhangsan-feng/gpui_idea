use super::SettingsContext;
use crate::domain::{
    language_server::ManagedServer,
    settings::{ContextMenuStatus, GeneralSettings, SettingsDataContextSnapshot},
};
use crate::event_bus::refresh_gui::GuiComponent;
use std::sync::Arc;

impl SettingsContext {
    pub(crate) fn publish_fonts(&self, fonts: Vec<crate::domain::settings::FontAsset>) {
        self.replace(
            |next| next.fonts = fonts.into(),
            &[GuiComponent::SettingsWindow],
        );
    }
    pub(crate) fn snapshot(&self) -> Arc<SettingsDataContextSnapshot> {
        self.snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn publish_context_menu(&self, status: Option<ContextMenuStatus>) {
        self.replace(
            |next| next.context_menu = status,
            &[GuiComponent::SettingsWindow],
        );
    }

    pub(crate) fn publish_servers(&self, servers: Vec<ManagedServer>) {
        self.replace(
            |next| next.servers = servers.into(),
            &[GuiComponent::SettingsWindow],
        );
    }

    pub(crate) fn publish_general(&self, value: GeneralSettings) {
        self.replace(
            |next| next.general = Arc::new(value),
            &[
                GuiComponent::SettingsWindow,
                GuiComponent::WindowBar,
                GuiComponent::BottomBar,
                GuiComponent::Editor,
                GuiComponent::FileTree,
                GuiComponent::BottomPanel,
            ],
        );
    }
}
