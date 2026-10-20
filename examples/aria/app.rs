// Ink examples/aria.

use ink_rs::{Element, FlexDirection, Scope, Styled, Text, View, component};

pub fn app() -> Element {
    component(checkbox, ())
}

fn checkbox(cx: &mut Scope, _: &()) -> Element {
    let checked = cx.use_state(|| false);
    cx.use_input(true, move |input, _| {
        if input == " " {
            checked.update(|value| *value = !*value);
        }
    });
    let instructions = "Press spacebar to toggle the checkbox. \
        This example is best experienced with a screen reader.";
    // M5 adds checkbox aria-role/state and aria-hidden on the last Text.
    View::new()
        .flex_direction(FlexDirection::Column)
        .child(Text::new(instructions))
        .child(View::new().margin_top(1).child(Text::new(if checked.get() {
            "[x]"
        } else {
            "[ ]"
        })))
        .child(
            View::new()
                .margin_top(1)
                .child(Text::new("This text is hidden from screen readers.")),
        )
        .into()
}
