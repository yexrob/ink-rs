use crate::{ColorLevel, Element, host, layout, paint};

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
/// [`Static`]: crate::Static
pub fn render_to_string(element: impl Into<Element>, options: RenderOptions) -> String {
    let (mut tree, root) = host::build(element.into(), options.columns, options.color_level);
    layout::compute(&mut tree, root);
    paint::paint(&tree, root, options.color_level).serialize()
}
