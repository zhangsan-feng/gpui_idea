use super::*;
use crate::event_bus::refresh_gui::GuiComponent;
impl TerminalContext {
    pub fn snapshot(&self, id: TerminalId) -> Arc<TerminalSnapshot> {
        self.snapshots
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }
    pub(crate) fn publish(&self, id: TerminalId, snapshot: TerminalSnapshot) {
        let retired = self.retired.read().unwrap();
        if retired.contains(&id) {
            return;
        }
        let mut snapshots = self.snapshots.write().unwrap();
        if snapshots
            .get(&id)
            .is_some_and(|current| current.generation > snapshot.generation)
        {
            return;
        }
        let status_changed = snapshots.get(&id).is_none_or(|previous| {
            previous.generation != snapshot.generation
                || previous.exited != snapshot.exited
                || previous.exit_code != snapshot.exit_code
                || previous.command != snapshot.command
                || previous.title != snapshot.title
        });
        snapshots.insert(id, Arc::new(snapshot));
        drop(snapshots);
        drop(retired);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[GuiComponent::Terminal(id)]);
        if status_changed {
            crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[
                GuiComponent::LeftSidebar,
                GuiComponent::WindowBar,
                GuiComponent::BottomPanel,
            ]);
        }
    }
    pub(crate) fn retire(&self, id: TerminalId) {
        let mut retired = self.retired.write().unwrap();
        retired.insert(id);
        self.snapshots.write().unwrap().remove(&id);
        drop(retired);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[
            GuiComponent::Terminal(id),
            GuiComponent::LeftSidebar,
            GuiComponent::WindowBar,
            GuiComponent::BottomPanel,
        ]);
    }
}
