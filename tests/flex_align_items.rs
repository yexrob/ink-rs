//! Ported from ink test/flex-align-items.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

#[test]
fn row_align_text_to_center() {
    let output = render(
        View::new()
            .align_items(AlignItems::Center)
            .height(3)
            .child(Text::new("Test")),
    );
    assert_eq!(output, lines(&["", "Test", "",]));
}

#[test]
fn row_align_multiple_text_nodes_to_center() {
    let output = render(
        View::new()
            .align_items(AlignItems::Center)
            .height(3)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["", "AB", "",]));
}

#[test]
fn row_align_text_to_bottom() {
    let output = render(
        View::new()
            .align_items(AlignItems::FlexEnd)
            .height(3)
            .child(Text::new("Test")),
    );
    assert_eq!(output, lines(&["", "", "Test",]));
}

#[test]
fn row_align_multiple_text_nodes_to_bottom() {
    let output = render(
        View::new()
            .align_items(AlignItems::FlexEnd)
            .height(3)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["", "", "AB",]));
}

#[test]
fn column_align_text_to_center() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::Center)
            .width(10)
            .child(Text::new("Test")),
    );
    assert_eq!(output, "   Test");
}

#[test]
fn column_align_text_to_right() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexEnd)
            .width(10)
            .child(Text::new("Test")),
    );
    assert_eq!(output, "      Test");
}

#[test]
fn row_align_items_stretch() {
    let output = render(
        View::new()
            .align_items(AlignItems::Stretch)
            .height(5)
            .child(
                View::new()
                    .border_style(BorderStyle::Single)
                    .child(Text::new("X")),
            ),
    );
    assert_eq!(output, lines(&["┌─┐", "│X│", "│ │", "│ │", "└─┘",]));
}

#[test]
fn row_default_align_items_stretches_children() {
    let output = render(
        View::new().height(5).child(
            View::new()
                .border_style(BorderStyle::Single)
                .child(Text::new("X")),
        ),
    );
    assert_eq!(output, lines(&["┌─┐", "│X│", "│ │", "│ │", "└─┘",]));
}

#[test]
fn row_align_text_to_baseline() {
    let output = render(
        View::new()
            .align_items(AlignItems::Baseline)
            .height(3)
            .child(Text::new("A").child(Newline::new()).child("B"))
            .child(Text::new("X")),
    );
    assert_eq!(output, lines(&["A", "BX", "",]));
}
