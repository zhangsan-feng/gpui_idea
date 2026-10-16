use super::*;
impl RefreshGui {
    pub(crate) fn subscribe(&self, component: GuiComponent) -> watch::Receiver<u64> {
        self.channels
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entry(component)
            .or_insert_with(|| watch::channel(0).0)
            .subscribe()
    }

    pub(crate) fn notify(&self, components: &[GuiComponent]) {
        let channels = self
            .channels
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for component in components {
            if let Some(channel) = channels.get(component) {
                channel.send_modify(|revision| *revision = revision.wrapping_add(1));
            }
        }
        tracing::debug!(?components, "component redraw requested");
    }
}
