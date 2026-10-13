mod core;
mod external;

pub struct ProjectService {
    reads: tokio::sync::Semaphore,
}
impl ProjectService {
    pub(super) fn new() -> Self {
        Self {
            reads: tokio::sync::Semaphore::new(2),
        }
    }
}
