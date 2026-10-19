//! Ported from ink test/flex-direction.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn direction_row() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Row)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "AB");
}

#[test]
fn undefined_direction_uses_the_default_row_layout() {
    let output = render(View::new().child(Text::new("A")).child(Text::new("B")));
    assert_eq!(output, "AB");
}

#[test]
fn direction_row_reverse() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::RowReverse)
            .width(4)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "  BA");
}

#[test]
fn direction_column() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "B",]));
}

#[test]
fn direction_column_reverse() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::ColumnReverse)
            .height(4)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["", "", "B", "A",]));
}

#[test]
fn dont_squash_text_nodes_when_column_direction_is_applied() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "B",]));
}
