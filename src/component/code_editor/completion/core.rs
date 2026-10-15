use super::CompletionDropdown;
use crate::application::{completion, lifecycle::run_application};
use gpui_kit::component::input::{EditorState, InputEvent};
use gpui_kit::{Context, Entity, Focusable as _, Window};
use lsp_types::CompletionResponse;
use std::time::Duration;

impl CompletionDropdown {
    pub(super) fn new(
        editor: &Entity<EditorState>,
        language: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let changes = cx.subscribe_in(editor, window, |this, _, event, window, cx| match event {
            InputEvent::Change => {
                let skip = std::mem::take(&mut this.skip_next_change);
                this.dismiss(cx);
                if !skip {
                    this.request(false, window, cx);
                }
            }
            InputEvent::Blur => this.dismiss(cx),
            _ => {}
        });
        let movement = cx.observe(editor, |this, editor, cx| {
            let editor = editor.read(cx);
            if editor.cursor() != this.offset
                || !editor.selected_range().is_empty()
                || !editor.is_editable()
            {
                this.dismiss(cx);
            }
        });
        Self {
            editor: editor.downgrade(),
            language,
            items: vec![],
            selected: 0,
            offset: editor.read(cx).cursor(),
            generation: 0,
            skip_next_change: false,
            scroll: Default::default(),
            request: None,
            pending: false,
            _subscriptions: vec![changes, movement],
        }
    }

    pub(super) fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        self.request = None;
        self.pending = false;
        if !self.items.is_empty() {
            self.items.clear();
            cx.notify();
        }
    }

    pub(super) fn request(&mut self, manual: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.dismiss(cx);
        let Some(editor) = self.editor.upgrade() else {
            return;
        };
        let state = editor.read(cx);
        if !state.is_editable()
            || !state.focus_handle(cx).is_focused(window)
            || !state.selected_range().is_empty()
        {
            return;
        }
        let offset = state.cursor();
        if !manual
            && !state
                .text()
                .chars_at(offset)
                .reversed()
                .next()
                .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
        {
            return;
        }
        let text = state.text().clone();
        self.offset = offset;
        let generation = self.generation;
        let language = self.language;
        let executor = cx.background_executor().clone();
        self.pending = true;
        self.request = Some(cx.spawn_in(window, async move |this, cx| {
            executor.timer(Duration::from_millis(100)).await;
            let result =
                run_application(async move { completion::complete(text, offset, language).await })
                    .await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.generation != generation {
                    return;
                }
                this.pending = false;
                let Some(editor) = this.editor.upgrade() else {
                    return;
                };
                let editor = editor.read(cx);
                if editor.cursor() != offset
                    || !editor.is_editable()
                    || !editor.focus_handle(cx).is_focused(window)
                    || !editor.selected_range().is_empty()
                {
                    return;
                }
                match result {
                    Ok(response) => {
                        this.items = match response {
                            CompletionResponse::Array(items) => items,
                            CompletionResponse::List(list) => list.items,
                        };
                        this.selected = 0;
                        this.scroll
                            .set_offset(gpui_kit::point(gpui_kit::px(0.), gpui_kit::px(0.)));
                        cx.notify();
                    }
                    Err(error) => {
                        tracing::warn!(%error, language, "code completion request failed")
                    }
                }
            });
        }));
    }

    pub(super) fn choose(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.items.get(index).cloned() else {
            return;
        };
        let Some(editor) = self.editor.upgrade() else {
            return;
        };
        let state = editor.read(cx);
        if state.cursor() != self.offset
            || !state.is_editable()
            || !state.selected_range().is_empty()
        {
            self.dismiss(cx);
            return;
        }
        self.skip_next_change = true;
        self.dismiss(cx);
        let offset = self.offset;
        editor.update(cx, |editor, cx| {
            editor.insert_completion(&item, offset..offset, window, cx)
        });
    }

    pub(super) fn move_selection(&mut self, down: bool, cx: &mut Context<Self>) -> bool {
        if self.items.is_empty() {
            return false;
        }
        self.selected = if down {
            (self.selected + 1) % self.items.len()
        } else {
            (self.selected + self.items.len() - 1) % self.items.len()
        };
        self.scroll.scroll_to_item(self.selected);
        cx.notify();
        true
    }
}
