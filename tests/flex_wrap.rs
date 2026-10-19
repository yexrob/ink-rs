//! Ported from ink test/flex-wrap.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn row_no_wrap() {
    let output = render(
        View::new()
            .width(2)
            .child(Text::new("A"))
            .child(Text::new("BC")),
    );
    assert_eq!(output, lines(&["BC", "",]));
}

#[test]
fn column_no_wrap() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .height(2)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&["B", "C",]));
}

#[test]
fn row_wrap_content() {
    let output = render(
        View::new()
            .width(2)
            .flex_wrap(FlexWrap::Wrap)
            .child(Text::new("A"))
            .child(Text::new("BC")),
    );
    assert_eq!(output, lines(&["A", "BC",]));
}

#[test]
fn column_wrap_content() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .height(2)
            .flex_wrap(FlexWrap::Wrap)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&["AC", "B",]));
}

#[test]
fn column_wrap_content_reverse() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .height(2)
            .width(3)
            .flex_wrap(FlexWrap::WrapReverse)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&[" CA", "  B",]));
}

#[test]
fn row_wrap_content_reverse() {
    let output = render(
        View::new()
            .height(3)
            .width(2)
            .flex_wrap(FlexWrap::WrapReverse)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&["", "C", "AB",]));
}
