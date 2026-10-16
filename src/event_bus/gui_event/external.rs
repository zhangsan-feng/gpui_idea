use super::*;
impl GuiEventBus {
    pub(crate) fn subscribe(
        &self,
        scope: gpui_kit::EntityId,
        target: GuiComponent,
    ) -> Subscription {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.mailboxes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert((scope, target), sender);
        Subscription {
            scope,
            target,
            receiver,
        }
    }
    pub(crate) fn send(&self, scope: gpui_kit::EntityId, event: GuiEvent) {
        let target = event.target();
        let mailboxes = self
            .mailboxes
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !mailboxes
            .get(&(scope, target))
            .is_some_and(|sender| sender.send(event).is_ok())
        {
            tracing::warn!(?scope, "GUI command has no active recipient");
        }
    }
}
