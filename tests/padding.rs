//! Ported from ink test/padding.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn padding() {
    let output = render(View::new().padding(2).child(Text::new("X")));
    assert_eq!(output, lines(&["", "", "  X", "", "",]));
}

#[test]
fn padding_x() {
    let output = render(
        View::new()
            .child(View::new().padding_x(2).child(Text::new("X")))
            .child(Text::new("Y")),
    );
    assert_eq!(output, "  X  Y");
}

#[test]
fn removing_padding_left_restores_padding_x_on_rerender() {
    fn example(_: &mut Scope, value: &Option<u16>) -> Element {
        let mut child = View::new().padding_x(2);
        if let Some(left) = value {
            child = child.padding_left(*left);
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
fn removing_padding_x_restores_padding_on_rerender() {
    fn example(_: &mut Scope, value: &Option<u16>) -> Element {
        let mut view = View::new().padding(2);
        if let Some(x) = value {
            view = view.padding_x(*x);
        }
        view.child(Text::new("X")).into()
    }
    let mut instance = testing::render(component(example, Some(1)));
    assert_ansi_eq(instance.last_frame(), &lines(&["", "", " X", "", ""]));
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), &lines(&["", "", "  X", "", ""]));
}

#[test]
fn padding_y() {
    let output = render(View::new().padding_y(2).child(Text::new("X")));
    assert_eq!(output, lines(&["", "", "X", "", "",]));
}

#[test]
fn padding_top() {
    let output = render(View::new().padding_top(2).child(Text::new("X")));
    assert_eq!(output, lines(&["", "", "X",]));
}

#[test]
fn padding_bottom() {
    let output = render(View::new().padding_bottom(2).child(Text::new("X")));
    assert_eq!(output, lines(&["X", "", "",]));
}

#[test]
fn padding_left() {
    let output = render(View::new().padding_left(2).child(Text::new("X")));
    assert_eq!(output, "  X");
}

#[test]
fn padding_right() {
    let output = render(
        View::new()
            .child(View::new().padding_right(2).child(Text::new("X")))
            .child(Text::new("Y")),
    );
    assert_eq!(output, "X  Y");
}

#[test]
fn nested_padding() {
    let output = render(
        View::new()
            .padding(2)
            .child(View::new().padding(2).child(Text::new("X"))),
    );
    assert_eq!(output, lines(&["", "", "", "", "    X", "", "", "", "",]));
}

#[test]
fn padding_with_multiline_string() {
    let output = render(View::new().padding(2).child(Text::new("A\nB")));
    assert_eq!(output, lines(&["", "", "  A", "  B", "", "",]));
}

#[test]
fn apply_padding_to_text_with_newlines() {
    let output = render(
        View::new()
            .padding(1)
            .child(Text::new("Hello").child("\n").child("World")),
    );
    assert_eq!(output, lines(&["", " Hello", " World", "",]));
}

#[test]
fn apply_padding_to_wrapped_text() {
    let output = render(
        View::new()
            .padding(1)
            .width(5)
            .child(Text::new("Hello World")),
    );
    assert_eq!(output, lines(&["", " Hel", " lo", " Wor", " ld", "",]));
}

#[test]
fn text_wrapping_respects_padding_x_with_flex_grow() {
    let output = render(
        View::new()
            .width(40)
            .border_style(BorderStyle::Round)
            .child(View::new().padding_x(2).child(
                View::new().margin_left(2).child(Text::new("•")).child(
                    View::new().flex_grow(1.0).margin_left(1).child(Text::new(
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit",
                    )),
                ),
            )),
    );
    for line in output.split('\n') {
        let length = line.encode_utf16().count();
        assert!(
            length <= 40,
            "Line {line:?} exceeds container width of 40 (got {length})"
        );
    }
}
