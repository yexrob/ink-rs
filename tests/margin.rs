//! Ported from ink test/margin.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn margin() {
    let output = render(View::new().margin(2).child(Text::new("X")));
    assert_eq!(output, lines(&["", "", "  X", "", "",]));
}

#[test]
fn margin_x() {
    let output = render(
        View::new()
            .child(View::new().margin_x(2).child(Text::new("X")))
            .child(Text::new("Y")),
    );
    assert_eq!(output, "  X  Y");
}

#[test]
fn removing_margin_left_restores_margin_x_on_rerender() {
    fn example(_: &mut Scope, value: &Option<i16>) -> Element {
        let mut child = View::new().margin_x(2);
        if let Some(left) = value {
            child = child.margin_left(*left);
        }
        View::new()
            .child(child.child(Text::new("X")))
            .child(Text::new("Y"))
            .into()
    }
    let mut instance = testing::render(component(example, Some(1)));
    assert_ansi_eq(instance.last_frame(), " X  Y");
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), "  X  Y");
    instance.rerender(component(example, Some(0)));
    assert_ansi_eq(instance.last_frame(), "X  Y");
}

#[test]
fn margin_y() {
    let output = render(View::new().margin_y(2).child(Text::new("X")));
    assert_eq!(output, lines(&["", "", "X", "", "",]));
}

#[test]
fn margin_top() {
    let output = render(View::new().margin_top(2).child(Text::new("X")));
    assert_eq!(output, lines(&["", "", "X",]));
}

#[test]
fn margin_bottom() {
    let output = render(View::new().margin_bottom(2).child(Text::new("X")));
    assert_eq!(output, lines(&["X", "", "",]));
}

#[test]
fn margin_left() {
    let output = render(View::new().margin_left(2).child(Text::new("X")));
    assert_eq!(output, "  X");
}

#[test]
fn margin_right() {
    let output = render(
        View::new()
            .child(View::new().margin_right(2).child(Text::new("X")))
            .child(Text::new("Y")),
    );
    assert_eq!(output, "X  Y");
}

#[test]
fn nested_margin() {
    let output = render(
        View::new()
            .margin(2)
            .child(View::new().margin(2).child(Text::new("X"))),
    );
    assert_eq!(output, lines(&["", "", "", "", "    X", "", "", "", "",]));
}

#[test]
fn margin_with_multiline_string() {
    let output = render(View::new().margin(2).child(Text::new("A\nB")));
    assert_eq!(output, lines(&["", "", "  A", "  B", "", "",]));
}

#[test]
fn apply_margin_to_text_with_newlines() {
    let output = render(
        View::new()
            .margin(1)
            .child(Text::new("Hello").child("\n").child("World")),
    );
    assert_eq!(output, lines(&["", " Hello", " World", "",]));
}

#[test]
fn apply_margin_to_wrapped_text() {
    let output = render(
        View::new()
            .margin(1)
            .width(6)
            .child(Text::new("Hello World")),
    );
    assert_eq!(output, lines(&["", " Hello", " World", "",]));
}

#[test]
fn negative_margin_top_with_multiline_string() {
    let output = render(
        View::new().flex_direction(FlexDirection::Column).child(
            View::new()
                .margin_top(-1)
                .flex_direction(FlexDirection::Column)
                .child(Text::new("Line 1\nLine 2\nLine 3")),
        ),
    );
    assert_eq!(output, lines(&["Line 2", "Line 3",]));
}
