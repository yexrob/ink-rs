// Ink examples/use-focus.

use ink_rs::{
    Color, Element, FlexDirection, FocusOptions, Options, Scope, Styled, Text, View, component,
    render,
};

fn item(cx: &mut Scope, label: &&'static str) -> Element {
    let focused = cx.use_focus(FocusOptions::default());
    Text::new(format!("{label} "))
        .child(Text::new(if focused { "(focused)" } else { "" }).color(Color::Green))
        .into()
}

fn main() -> Result<(), ink_rs::Error> {
    let instructions =
        "Press Tab to focus next element, Shift+Tab to focus previous element, Esc to reset focus.";
    let app = View::new()
        .flex_direction(FlexDirection::Column)
        .padding(1)
        .child(View::new().margin_bottom(1).child(Text::new(instructions)))
        .children(["First", "Second", "Third"].map(|label| component(item, label)));
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
