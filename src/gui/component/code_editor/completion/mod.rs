mod core;
mod external;
mod popup;
mod ui;

use gpui_kit::component::input::EditorState;
use gpui_kit::{Entity, EntityId, Global, ScrollHandle, Subscription, Task, WeakEntity};
use lsp_types::CompletionItem;
use std::collections::HashMap;

gpui_kit::actions!(code_completion, [ShowCodeCompletions]);

#[derive(Default)]
struct CompletionRegistry(HashMap<EntityId, Entity<CompletionDropdown>>);
impl Global for CompletionRegistry {}

pub(crate) struct CompletionDropdown {
    editor: WeakEntity<EditorState>,
    language: &'static str,
    path: std::path::PathBuf,
    cancellation: Option<tokio::sync::watch::Sender<bool>>,
    items: Vec<CompletionItem>,
    selected: usize,
    offset: usize,
    generation: u64,
    skip_next_change: bool,
    scroll: ScrollHandle,
    request: Option<Task<()>>,
    pending: bool,
    _subscriptions: Vec<Subscription>,
}

pub(crate) use external::{attach, dismiss, install_keys, wrap};

impl gpui_kit::Render for CompletionDropdown {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        use gpui_kit::{Focusable as _, IntoElement as _};
        if self.items.is_empty()
            || !self.editor.upgrade().is_some_and(|editor| {
                editor.read(cx).focus_handle(cx).is_focused(window) && editor.read(cx).is_editable()
            })
        {
            return gpui_kit::Empty.into_any_element();
        }
        gpui_kit::deferred(popup::CompletionPopup::new(
            cx.entity(),
            self.editor.clone(),
        ))
        .into_any_element()
    }
}
