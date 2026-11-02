mod core;
mod external;

pub(super) use external::attach;

use gpui_kit::component::input::{EditorState, RangeDecorationCollection, Rope};
use gpui_kit::{Entity, EntityId, Global, Subscription, Task, WeakEntity};
use std::{collections::HashMap, ops::Range, path::PathBuf};

#[derive(Default)]
struct HighlightRegistry(HashMap<EntityId, Entity<Highlights>>);
impl Global for HighlightRegistry {}

#[derive(PartialEq)]
struct Snapshot {
    text: Rope,
    selection: Range<usize>,
    cursor: usize,
    focused: bool,
    dark: bool,
}

struct Highlights {
    editor: WeakEntity<EditorState>,
    path: PathBuf,
    decorations: RangeDecorationCollection,
    snapshot: Option<Snapshot>,
    generation: u64,
    cancellation: Option<tokio::sync::watch::Sender<bool>>,
    pending: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}
