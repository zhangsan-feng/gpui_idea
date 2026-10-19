mod core;
mod external;

use crate::domain::document::DocumentSnapshot;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, atomic::AtomicBool},
};

pub struct DocumentContext {
    documents: Mutex<HashMap<PathBuf, Arc<DocumentSnapshot>>>,
    autosave_paused: AtomicBool,
}
impl DocumentContext {
    pub(super) fn new() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
            autosave_paused: AtomicBool::new(false),
        }
    }
}
