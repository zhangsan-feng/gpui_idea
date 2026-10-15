mod core;
mod external;

use crate::domain::document::DocumentSnapshot;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct DocumentContext {
    documents: Mutex<HashMap<PathBuf, Arc<DocumentSnapshot>>>,
}
impl DocumentContext {
    pub(super) fn new() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
        }
    }
}
