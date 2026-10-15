mod external;
use crate::domain::terminal::{TerminalId, TerminalSnapshot};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

pub struct TerminalContext {
    snapshots: RwLock<HashMap<TerminalId, Arc<TerminalSnapshot>>>,
    retired: RwLock<HashSet<TerminalId>>,
}
impl TerminalContext {
    pub(crate) fn new() -> Self {
        Self {
            snapshots: RwLock::new(HashMap::new()),
            retired: RwLock::new(HashSet::new()),
        }
    }
}
