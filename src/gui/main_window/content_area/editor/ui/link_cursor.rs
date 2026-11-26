use crate::application::{
    editor::{LinkScope, web_link_spans},
    lifecycle::run_application,
};
use crate::component::code_editor::glyph_offset_at_point;
use gpui_kit::{
    App, CursorStyle, DispatchPhase, Entity, Hitbox, Modifiers, MouseMoveEvent, Task, Window,
    component::input::EditorState,
};
use ropey::{LineType, Rope, extra::esoterica::ropes_are_instances};
use std::{
    cell::{Cell, RefCell},
    ops::Range,
    rc::Rc,
};

struct LinkLookup {
    buffer: Rope,
    scope: LinkScope,
    spans: Rc<RefCell<Vec<Range<usize>>>>,
    _task: Task<()>,
}

#[derive(Default)]
pub(super) struct LinkCursor {
    lookup: RefCell<Option<LinkLookup>>,
}

impl LinkCursor {
    fn pointing(
        &self,
        editor: &Entity<EditorState>,
        hitbox: &Hitbox,
        is_markdown: bool,
        modifiers: Modifiers,
        window: &Window,
        cx: &App,
    ) -> bool {
        if !modifiers.control
            || modifiers.alt
            || modifiers.shift
            || !window.is_window_active()
            || cx.has_active_drag()
            || !hitbox.is_hovered(window)
        {
            return false;
        }
        let editor = editor.read(cx);
        let Some(offset) = glyph_offset_at_point(editor, window.mouse_position()) else {
            return false;
        };
        let buffer = editor.text();
        // Markdown reference destinations may be defined elsewhere in the document.
        let scope = if is_markdown {
            LinkScope::Markdown
        } else {
            LinkScope::Line(buffer.byte_to_line_idx(offset, LineType::LF))
        };
        let mut lookup = self.lookup.borrow_mut();
        if let Some(lookup) = lookup.as_ref()
            && lookup.scope == scope
            && ropes_are_instances(buffer, &lookup.buffer)
        {
            let spans = lookup.spans.borrow();
            let next = spans.partition_point(|span| span.end <= offset);
            return spans.get(next).is_some_and(|span| span.contains(&offset));
        }

        let spans = Rc::new(RefCell::new(Vec::new()));
        let pending_spans = spans.clone();
        let pending_buffer = buffer.clone();
        let task = window.spawn(cx, async move |cx| {
            match run_application(web_link_spans(pending_buffer, scope)).await {
                Ok(found) if !found.is_empty() => {
                    pending_spans.replace(found);
                    let _ = cx.update(|window, _| window.refresh());
                }
                Ok(_) => {}
                Err(error) => tracing::debug!(%error, "Editor link cursor lookup failed"),
            }
        });
        *lookup = Some(LinkLookup {
            buffer: buffer.clone(),
            scope,
            spans,
            _task: task,
        });
        false
    }

    pub(super) fn install(
        self: &Rc<Self>,
        editor: &Entity<EditorState>,
        hitbox: &Hitbox,
        is_markdown: bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        let active = self.pointing(editor, hitbox, is_markdown, window.modifiers(), window, cx);
        if active {
            window.set_cursor_style(CursorStyle::PointingHand, hitbox);
        }
        let active = Rc::new(Cell::new(active));
        let cursor = self.clone();
        let state = editor.clone();
        let region = hitbox.clone();
        let moved = active.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture {
                let next =
                    cursor.pointing(&state, &region, is_markdown, event.modifiers, window, cx);
                if moved.replace(next) != next {
                    window.refresh();
                }
            }
        });
        let cursor = self.clone();
        let state = editor.clone();
        let region = hitbox.clone();
        window.on_modifiers_changed(move |event, window, cx| {
            let next = cursor.pointing(&state, &region, is_markdown, event.modifiers, window, cx);
            if active.replace(next) != next {
                window.refresh();
            }
        });
    }
}
