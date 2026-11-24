use super::SettingsContext;
use crate::domain::{
    language_server::ManagedServer,
    settings::{ContextMenuStatus, GeneralSettings, SettingsDataContextSnapshot},
};
use crate::event_bus::refresh_gui::GuiComponent;
use std::sync::{Arc, atomic::Ordering};

impl SettingsContext {
    pub(crate) fn editor_font_size(&self) -> u16 {
        self.pending_editor_font_size()
            .unwrap_or_else(|| self.snapshot().general.appearance.editor_size)
    }

    pub(crate) fn pending_editor_font_size(&self) -> Option<u16> {
        let size = self.pending_editor_font_size.load(Ordering::Acquire);
        (size != 0).then_some(size)
    }

    pub(crate) fn stage_editor_font_size(&self, size: u16) {
        self.pending_editor_font_size.store(size, Ordering::Release);
    }

    pub(crate) fn clear_pending_editor_font_size(&self, size: u16) {
        let _ = self.pending_editor_font_size.compare_exchange(
            size,
            0,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    pub(crate) fn publish_editor_font_size(&self, size: u16) {
        self.replace(
            |next| Arc::make_mut(&mut next.general).appearance.editor_size = size,
            &[GuiComponent::SettingsWindow],
        );
    }

    pub(crate) fn publish_file_associations(
        &self,
        status: Option<crate::domain::settings::FileAssociationStatus>,
    ) {
        self.replace(
            |next| next.file_associations = status,
            &[GuiComponent::SettingsWindow],
        );
    }

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
                GuiComponent::BottomSidebar,
                GuiComponent::Editor,
                GuiComponent::FileTree,
                GuiComponent::BottomPanel,
            ],
        );
    }
}
