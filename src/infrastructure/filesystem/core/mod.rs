mod blocks;
mod directory;
mod file;
mod lines;
pub(super) mod path;

pub(super) use blocks::{modify_blocks, write_changed_block};
pub(super) use directory::{create_directory, remove_directory};
pub(super) use file::{
    create_file, metadata, read_file, remove_file, rename, set_readonly, write_file,
};
pub(super) use lines::modify_lines;

mod task;
pub(super) use task::run;
