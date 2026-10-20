// Ink examples/terminal-resize.

use ink_rs::{Color, Element, FlexDirection, Scope, Text, View, component, rsx};

pub fn app() -> Element {
    component(terminal_resize, ())
}

fn terminal_resize(cx: &mut Scope, _: &()) -> Element {
    let size = cx.use_window_size();
    cx.use_input(true, |_, _| {});
    rsx! {
        View(flex_direction: FlexDirection::Column, padding: 1) {
            Text(bold: true, color: Color::Cyan) { "Terminal Size" }
            Text { {format!("Columns: {}", size.columns)} }
            Text { {format!("Rows: {}", size.rows)} }
            View(margin_top: 1) {
                Text(dim_color: true) {
                    "Resize your terminal to see the values update. Press Ctrl+C to exit."
                }
            }
        }
    }
    .into()
}
