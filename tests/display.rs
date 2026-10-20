//! Ported from ink test/display.tsx @ 26d2c3f.

mod common;

use common::{render, *};
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

#[test]
fn removing_display_none_restores_the_default_layout() {
    fn example(_: &mut Scope, value: &Option<Display>) -> Element {
        let mut child = View::new();
        if let Some(display) = value {
            child = child.display(*display);
        }
        View::new()
            .child(child.child(Text::new("Cat")))
            .child(Text::new("Dog"))
            .into()
    }
    let mut instance = testing::render(component(example, Some(Display::None)));
    assert_ansi_eq(instance.last_frame(), "Dog");
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), "CatDog");
}

#[test]
fn removing_display_flex_restores_the_default_layout() {
    fn example(_: &mut Scope, value: &Option<Display>) -> Element {
        let mut child = View::new();
        if let Some(display) = value {
            child = child.display(*display);
        }
        View::new()
            .child(child.child(Text::new("Cat")))
            .child(Text::new("Dog"))
            .into()
    }
    let mut instance = testing::render(component(example, Some(Display::Flex)));
    assert_ansi_eq(instance.last_frame(), "CatDog");
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), "CatDog");
}
