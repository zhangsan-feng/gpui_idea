mod common;
mod interpreted;
mod javascript;
mod native;
mod rust;

mod core;
pub(in crate::infrastructure::filesystem) use core::resolve;
