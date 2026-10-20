// Ink examples/use-focus-with-id.

use ink_rs::{
    Color, Element, FlexDirection, FocusOptions, Options, Scope, Text, View, component, render, rsx,
};

fn item(cx: &mut Scope, id: &&'static str) -> Element {
    let focused = cx.use_focus(FocusOptions {
        id: Some((*id).into()),
        ..FocusOptions::default()
    });
    rsx! {
        Text {
            "Press {id} to focus "
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

fn focus(cx: &mut Scope, _: &()) -> Element {
    let manager = cx.use_focus_manager();
    cx.use_input(true, move |input, _| {
        if ["1", "2", "3"].contains(&input) {
            manager.focus(input);
        }
    });
    let instructions =
        "Press Tab to focus next element, Shift+Tab to focus previous element, Esc to reset focus.";
    rsx! {
        View(flex_direction: FlexDirection::Column, padding: 1) {
            View(margin_bottom: 1) {
                Text { {instructions} }
            }
            for id in ["1", "2", "3"] {
                {component(item, id)}
            }
        }
    }
    .into()
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(focus, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
