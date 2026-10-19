//! Ported from ink test/display.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn display_flex() {
    let output = render(View::new().display(Display::Flex).child(Text::new("X")));
    assert_eq!(output, "X");
}

#[test]
fn display_none() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .display(Display::None)
                    .child(Text::new("Kitty!")),
            )
            .child(Text::new("Doggo")),
    );
    assert_eq!(output, "Doggo");
}
