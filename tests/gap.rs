//! Ported from ink test/gap.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn gap() {
    let output = render(
        View::new()
            .gap(1)
            .width(3)
            .flex_wrap(FlexWrap::Wrap)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&["A B", "", "C",]));
}

#[test]
fn column_gap() {
    let output = render(
        View::new()
            .gap(1)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A B");
}

#[test]
fn row_gap() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .gap(1)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "", "B",]));
}
