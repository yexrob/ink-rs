use std::time::Instant;

use crate::{ColorLevel, Element, runtime::Runtime};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderOptions {
    /// Width of the virtual terminal.
    pub columns: u16,
    pub color_level: ColorLevel,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            columns: 80,
            color_level: ColorLevel::TrueColor,
        }
    }
}

/// Renders an element tree to a string, without a terminal. [`Static`]
/// output comes first, followed by the rest of the tree. Lines carry no
/// trailing whitespace except styled cells, and the result has no trailing
/// newline.
///
/// Layout effects settle before paint. Passive effects run after the output
/// is captured, so their state updates do not change the returned string.
/// Unmount runs effect cleanups, including when rendering unwinds.
///
/// [`Static`]: crate::Static
pub fn render_to_string(element: impl Into<Element>, options: RenderOptions) -> String {
    let mut runtime = Runtime::new(element.into(), options);
    let mut output = String::new();
    runtime.render_if_due(Instant::now(), |frame| output = frame.serialize());
    output
}
