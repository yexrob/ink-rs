// Ink examples/use-focus.

use ink_rs::{
    Color, Element, FlexDirection, FocusOptions, Options, Scope, Text, View, component, render, rsx,
};

fn item(cx: &mut Scope, label: &&'static str) -> Element {
    let focused = cx.use_focus(FocusOptions::default());
    rsx! {
        Text {
            "{label} "
            Text(color: Color::Green) {
                if focused {
                    "(focused)"
                } else {
                    ""
                }
            }
        }
    }
    .into()
}

fn main() -> Result<(), ink_rs::Error> {
    let instructions =
        "Press Tab to focus next element, Shift+Tab to focus previous element, Esc to reset focus.";
    let app = rsx! {
        View(flex_direction: FlexDirection::Column, padding: 1) {
            View(margin_bottom: 1) {
                Text { {instructions} }
            }
            for label in ["First", "Second", "Third"] {
                {component(item, label)}
            }
        }
    };
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
