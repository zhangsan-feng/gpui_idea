mod external;
mod ui;

pub(crate) use external::open;
pub(crate) use ui::{content, frame};

use gpui_kit::{Pixels, Size};

/// Dimensions describe the content area, excluding the shared title bar.
pub(crate) struct ToolWindowOptions {
    title: &'static str,
    dimensions: Size<Pixels>,
    minimum: Size<Pixels>,
    resizable: bool,
}

impl ToolWindowOptions {
    pub(crate) fn new(title: &'static str, dimensions: Size<Pixels>) -> Self {
        Self {
            title,
            dimensions,
            minimum: dimensions,
            resizable: false,
        }
    }

    pub(crate) fn resizable(mut self, minimum: Size<Pixels>) -> Self {
        self.minimum = minimum;
        self.resizable = true;
        self
    }
}
