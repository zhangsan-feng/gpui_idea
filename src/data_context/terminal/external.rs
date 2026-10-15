use super::*;
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
        snapshots.insert(id, Arc::new(snapshot));
        drop(snapshots);
        drop(retired);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
    pub(crate) fn retire(&self, id: TerminalId) {
        let mut retired = self.retired.write().unwrap();
        retired.insert(id);
        self.snapshots.write().unwrap().remove(&id);
        drop(retired);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
}
