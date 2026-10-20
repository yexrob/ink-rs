// Ink examples/aria.

use ink_rs::{Element, FlexDirection, Scope, Text, View, component, rsx};

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
    // The deferred screen-reader mode would add checkbox aria-role/state and
    // aria-hidden on the last Text.
    rsx! {
        View(flex_direction: FlexDirection::Column) {
            Text { {instructions} }
            View(margin_top: 1) {
                Text {
                    if checked.get() {
                        "[x]"
                    } else {
                        "[ ]"
                    }
                }
            }
            View(margin_top: 1) {
                Text { "This text is hidden from screen readers." }
            }
        }
    }
    .into()
}
