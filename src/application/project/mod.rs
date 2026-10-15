mod core;
mod external;

pub struct ProjectService {
    file_operations: tokio::sync::Mutex<()>,
    reads: tokio::sync::Semaphore,
    run_initialized: std::sync::atomic::AtomicBool,
    run_updates: std::sync::Mutex<()>,
    run_executions: tokio::sync::Mutex<()>,
    run_sessions_updates: std::sync::Mutex<()>,
    run_instance_sequence: std::sync::atomic::AtomicU64,
    run_configuration_sequence: std::sync::atomic::AtomicU64,
}
impl ProjectService {
    pub(super) fn new() -> Self {
        Self {
            file_operations: tokio::sync::Mutex::new(()),
            reads: tokio::sync::Semaphore::new(2),
            run_initialized: std::sync::atomic::AtomicBool::new(false),
            run_updates: std::sync::Mutex::new(()),
            run_executions: tokio::sync::Mutex::new(()),
            run_sessions_updates: std::sync::Mutex::new(()),
            run_instance_sequence: std::sync::atomic::AtomicU64::new(1),
            run_configuration_sequence: std::sync::atomic::AtomicU64::new(2),
        }
    }
}
