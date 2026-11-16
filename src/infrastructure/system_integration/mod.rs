#[cfg(windows)]
mod association_registry;
mod core;
#[cfg(windows)]
mod default_associations;
mod external;
#[cfg(windows)]
mod file_associations;
#[cfg(windows)]
mod registry;

pub(crate) struct SystemIntegration;
