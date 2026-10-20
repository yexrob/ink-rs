// Ink examples/select-input.

use ink_rs::{
    Color, Element, FlexDirection, Options, Scope, Styled, Text, View, component, render,
};

fn select(cx: &mut Scope, _: &()) -> Element {
    let items = ["Red", "Green", "Blue", "Yellow", "Magenta", "Cyan"];
    let selected = cx.use_state(|| 0_usize);
    cx.use_input(true, move |_, key| {
        selected.update(|index| {
            if key.up_arrow {
                *index = (*index + items.len() - 1) % items.len();
            }
            if key.down_arrow {
                *index = (*index + 1) % items.len();
            }
        });
    });
    // M5 adds aria list/listitem props and the screen-reader numbered-input branch.
    View::new()
        .flex_direction(FlexDirection::Column)
        .child(Text::new("Select a color:"))
        .children(items.into_iter().enumerate().map(|(i, item)| {
            let active = selected.get() == i;
            Text::new(format!("{} {item}", if active { ">" } else { " " }))
                .color(active.then_some(Color::Blue))
                .key(item)
        }))
        .into()
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(select, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
