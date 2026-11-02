mod core;
mod external;

use crate::domain::language_server::LanguageServerDataContextSnapshot;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub(crate) struct LanguageServerContext {
    snapshot: Mutex<Arc<LanguageServerDataContextSnapshot>>,
}
