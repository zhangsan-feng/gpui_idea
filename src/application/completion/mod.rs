mod core;
mod external;
mod insertion;
mod keywords;
mod semantic;
mod snippet;

pub(crate) use external::{complete, prepare_insertion};
