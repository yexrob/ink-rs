// Ink examples/use-focus-with-id.

use ink_rs::{
    Color, Element, FlexDirection, FocusOptions, Options, Scope, Styled, Text, View, component,
    render,
};

fn item(cx: &mut Scope, id: &&'static str) -> Element {
    let focused = cx.use_focus(FocusOptions {
        id: Some((*id).into()),
        ..FocusOptions::default()
    });
    Text::new(format!("Press {id} to focus "))
        .child(Text::new(if focused { "(focused)" } else { "" }).color(Color::Green))
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
    View::new()
        .flex_direction(FlexDirection::Column)
        .padding(1)
        .child(View::new().margin_bottom(1).child(Text::new(instructions)))
        .children(["1", "2", "3"].map(|id| component(item, id)))
        .into()
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(focus, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
