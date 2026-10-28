mod core;
mod external;
mod keywords;
mod insertion;
mod semantic;
mod snippet;

pub(crate) use external::{complete, prepare_insertion};
