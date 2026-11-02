use super::{HighlightRegistry, Highlights};
use gpui_kit::{App, AppContext as _, Entity, Window, component::input::EditorState};
use std::path::PathBuf;

pub(in crate::gui::component::code_editor) fn attach(
    editor: &Entity<EditorState>,
    path: PathBuf,
    window: &mut Window,
    cx: &mut App,
) {
    if !cx.has_global::<HighlightRegistry>() {
        cx.set_global(HighlightRegistry::default());
    }
    let decorations = editor.update(cx, |state, cx| {
        state.create_range_decorations_collection(vec![], cx)
    });
    let highlights = cx.new(|cx| {
        let observer = cx.observe_in(editor, window, |this: &mut Highlights, _, window, cx| {
            this.refresh(window, cx);
        });
        let events = cx.subscribe_in(editor, window, |this: &mut Highlights, _, _, window, cx| {
            this.refresh(window, cx);
        });
        Highlights {
            editor: editor.downgrade(),
            path,
            decorations,
            snapshot: None,
            generation: 0,
            cancellation: None,
            pending: None,
            _subscriptions: vec![observer, events],
        }
    });
    highlights.update(cx, |this, cx| this.refresh(window, cx));
    let id = editor.entity_id();
    cx.global_mut::<HighlightRegistry>()
        .0
        .insert(id, highlights);
    cx.observe_release(editor, move |_, cx| {
        cx.global_mut::<HighlightRegistry>().0.remove(&id);
    })
    .detach();
}
