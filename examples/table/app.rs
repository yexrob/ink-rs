// Ink examples/table: fixed rows replace faker, without a data dependency.

use ink_rs::{Element, FlexDirection, Percent, Styled, Text, View};

fn row(id: &str, name: &str, email: &str) -> View {
    View::new().children(
        [(10.0, id), (50.0, name), (40.0, email)].map(|(width, text)| {
            View::new()
                .width(Percent(width))
                .child(Text::new(text.to_owned()))
        }),
    )
}

pub fn app() -> Element {
    let names = [
        "ada", "grace", "linus", "margaret", "alan", "barbara", "donald", "frances", "ken", "radia",
    ];
    View::new()
        .flex_direction(FlexDirection::Column)
        .width(Percent(100.0))
        .max_width(80)
        .child(row("ID", "Name", "Email"))
        .children(
            names
                .into_iter()
                .enumerate()
                .map(|(i, name)| row(&i.to_string(), name, &format!("{name}@example.com")).key(i)),
        )
        .into()
}
