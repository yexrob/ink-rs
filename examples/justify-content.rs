// Ink examples/justify-content.

use ink_rs::{FlexDirection, JustifyContent, Options, Styled, Text, View, render};

fn main() -> Result<(), ink_rs::Error> {
    let modes = [
        ("flex-start", JustifyContent::FlexStart),
        ("flex-end", JustifyContent::FlexEnd),
        ("center", JustifyContent::Center),
        ("space-around", JustifyContent::SpaceAround),
        ("space-between", JustifyContent::SpaceBetween),
        ("space-evenly", JustifyContent::SpaceEvenly),
    ];
    let app = View::new()
        .flex_direction(FlexDirection::Column)
        .children(modes.map(|(name, mode)| {
            View::new()
                .child(Text::new("["))
                .child(
                    View::new()
                        .justify_content(mode)
                        .width(20)
                        .height(1)
                        .child(Text::new("X"))
                        .child(Text::new("Y")),
                )
                .child(Text::new(format!("] {name}")))
        }));
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
