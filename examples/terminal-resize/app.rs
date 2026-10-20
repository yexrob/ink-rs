// Ink examples/terminal-resize.

use ink_rs::{Color, Element, FlexDirection, Scope, Styled, Text, View, component};

pub fn app() -> Element {
    component(terminal_resize, ())
}

fn terminal_resize(cx: &mut Scope, _: &()) -> Element {
    let size = cx.use_window_size();
    cx.use_input(true, |_, _| {});
    View::new()
        .flex_direction(FlexDirection::Column)
        .padding(1)
        .child(Text::new("Terminal Size").bold(true).color(Color::Cyan))
        .child(Text::new(format!("Columns: {}", size.columns)))
        .child(Text::new(format!("Rows: {}", size.rows)))
        .child(
            View::new().margin_top(1).child(
                Text::new("Resize your terminal to see the values update. Press Ctrl+C to exit.")
                    .dim_color(true),
            ),
        )
        .into()
}
