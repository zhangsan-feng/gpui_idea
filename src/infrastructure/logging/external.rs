pub use super::core::initialize;

pub(crate) fn is_output_file(path: &std::path::Path) -> bool {
    super::OUTPUT_FILE
        .get()
        .is_some_and(|output| output == path)
}
