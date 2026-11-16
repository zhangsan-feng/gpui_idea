mod bridge;
mod core;
mod external;
mod memory;

pub use external::initialize;
pub(crate) use external::is_output_file;
pub(crate) use memory::log_memory;

static OUTPUT_FILE: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
