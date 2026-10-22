mod bridge;
mod core;
mod external;
mod memory;

pub use external::initialize;
pub(crate) use memory::log_memory;
