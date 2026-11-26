use crate::application::{
    editor::clipboard_text_with_normalized_breaks, lifecycle::run_application,
};
use gpui_kit::{
    Div, Entity, Focusable as _, InteractiveElement as _,
    component::input::{EditorState, Paste},
};
use std::cell::Cell;

thread_local! {
    static FORWARDING_NATIVE_PASTE: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn capture(wrapper: Div, editor: &Entity<EditorState>) -> Div {
    let editor = editor.clone();
    wrapper.capture_action(move |_: &Paste, window, cx| {
        let state = editor.read(cx);
        if FORWARDING_NATIVE_PASTE.get() || !state.is_editable() {
            return;
        }

        let text_before = state.text().clone();
        let selection = state.selected_range();
        let cursor = state.cursor();
        cx.stop_propagation();
        editor.update(cx, |_, cx| {
            cx.spawn_in(window, async move |editor, cx| {
                let normalized =
                    match run_application(clipboard_text_with_normalized_breaks()).await {
                        Ok(normalized) => normalized,
                        Err(error) => {
                            tracing::debug!(%error, "clipboard line ending inspection failed");
                            None
                        }
                    };
                let _ = editor.update_in(cx, |state, window, cx| {
                    let focus = state.focus_handle(cx);
                    if !state.is_editable()
                        || !focus.is_focused(window)
                        || state.cursor() != cursor
                        || state.selected_range() != selection
                        || state.text() != &text_before
                    {
                        return;
                    }
                    if let Some(text) = normalized {
                        state.replace(text, window, cx);
                    } else {
                        // Dispatch outside the update so GPUI can borrow the editor.
                        window.defer(cx, move |window, cx| {
                            FORWARDING_NATIVE_PASTE.set(true);
                            focus.dispatch_action(&Paste, window, cx);
                            FORWARDING_NATIVE_PASTE.set(false);
                        });
                    }
                });
            })
            .detach();
        });
    })
}
