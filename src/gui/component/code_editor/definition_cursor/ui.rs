use super::core;
use gpui_kit::{
    App, CursorStyle, DispatchPhase, Entity, Hitbox, Modifiers, MouseMoveEvent, Window,
    component::input::EditorState,
};
use std::{cell::Cell, rc::Rc};

fn pointing(
    editor: &Entity<EditorState>,
    hitbox: &Hitbox,
    modifiers: Modifiers,
    window: &Window,
    cx: &App,
) -> bool {
    modifiers.control
        && !modifiers.alt
        && !modifiers.shift
        && window.is_window_active()
        && !cx.has_active_drag()
        && hitbox.is_hovered(window)
        && core::over_identifier(editor.read(cx), window.mouse_position())
}

// Install during canvas paint so our cursor overrides the editor's I-beam
// only over a Ctrl-hovered Rust name. The next frame restores its normal cursor.
pub(crate) fn install(
    editor: &Entity<EditorState>,
    hitbox: &Hitbox,
    window: &mut Window,
    cx: &mut App,
) {
    let active = pointing(editor, hitbox, window.modifiers(), window, cx);
    if active {
        window.set_cursor_style(CursorStyle::PointingHand, hitbox);
    }
    let active = Rc::new(Cell::new(active));
    let moving = active.clone();
    let state = editor.clone();
    let region = hitbox.clone();
    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
        if phase != DispatchPhase::Capture {
            return;
        }
        let next = pointing(&state, &region, event.modifiers, window, cx);
        if moving.replace(next) != next {
            window.refresh();
        }
    });
    let state = editor.clone();
    let region = hitbox.clone();
    window.on_modifiers_changed(move |event, window, cx| {
        let next = pointing(&state, &region, event.modifiers, window, cx);
        if active.replace(next) != next {
            window.refresh();
        }
    });
}
