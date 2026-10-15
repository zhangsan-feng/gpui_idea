use super::{CompletionDropdown, CompletionRegistry, ShowCodeCompletions};
use gpui_kit::component::input::{EditorState, Enter, Escape, MoveDown, MoveUp};
use gpui_kit::{
    App, AppContext as _, Div, Entity, InteractiveElement as _, IntoElement, KeyBinding,
    ParentElement as _, Styled as _, Window, div,
};

pub(in crate::component::code_editor) fn install_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "ctrl-space",
        ShowCodeCompletions,
        Some("Input && mode == editor"),
    )]);
}

pub(in crate::component::code_editor) fn attach(
    editor: &Entity<EditorState>,
    language: &'static str,
    window: &mut Window,
    cx: &mut App,
) {
    if !cx.has_global::<CompletionRegistry>() {
        cx.set_global(CompletionRegistry::default());
    }
    let dead: Vec<_> = cx
        .global::<CompletionRegistry>()
        .0
        .iter()
        .filter(|(_, menu)| menu.read(cx).editor.upgrade().is_none())
        .map(|(id, _)| *id)
        .collect();
    for id in dead {
        cx.global_mut::<CompletionRegistry>().0.remove(&id);
    }
    let menu = cx.new(|cx| CompletionDropdown::new(editor, language, window, cx));
    cx.global_mut::<CompletionRegistry>()
        .0
        .insert(editor.entity_id(), menu);
}

pub(in crate::component::code_editor) fn dismiss(editor: &Entity<EditorState>, cx: &mut App) {
    let menu = cx
        .try_global::<CompletionRegistry>()
        .and_then(|registry| registry.0.get(&editor.entity_id()))
        .cloned();
    if let Some(menu) = menu {
        menu.update(cx, |menu, cx| menu.dismiss(cx));
    }
}

pub(in crate::component::code_editor) fn wrap(
    editor: &Entity<EditorState>,
    content: impl IntoElement,
    cx: &App,
) -> Div {
    let Some(menu) = cx
        .try_global::<CompletionRegistry>()
        .and_then(|registry| registry.0.get(&editor.entity_id()))
        .cloned()
    else {
        return div().size_full().child(content);
    };
    let enter = menu.clone();
    let up = menu.clone();
    let down = menu.clone();
    let escape = menu.clone();
    let show = menu.clone();
    div()
        .size_full()
        .min_w_0()
        .min_h_0()
        .capture_action(move |action: &Enter, window, cx| {
            if action.shift || action.secondary {
                return;
            }
            let handled = enter.update(cx, |menu, cx| {
                if menu.items.is_empty() {
                    return false;
                }
                menu.choose(menu.selected, window, cx);
                true
            });
            if handled {
                cx.stop_propagation();
            }
        })
        .capture_action(move |_: &MoveUp, _, cx| {
            if up.update(cx, |menu, cx| menu.move_selection(false, cx)) {
                cx.stop_propagation();
            }
        })
        .capture_action(move |_: &MoveDown, _, cx| {
            if down.update(cx, |menu, cx| menu.move_selection(true, cx)) {
                cx.stop_propagation();
            }
        })
        .capture_action(move |_: &Escape, _, cx| {
            let handled = escape.update(cx, |menu, cx| {
                let open = !menu.items.is_empty() || menu.pending;
                menu.dismiss(cx);
                open
            });
            if handled {
                cx.stop_propagation();
            }
        })
        .capture_action(move |_: &ShowCodeCompletions, window, cx| {
            show.update(cx, |menu, cx| menu.request(true, window, cx));
            cx.stop_propagation();
        })
        .child(content)
        .child(menu)
}
