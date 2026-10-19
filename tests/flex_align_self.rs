//! Ported from ink test/flex-align-self.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn row_align_text_to_center() {
    let output = render(
        View::new().height(3).child(
            View::new()
                .align_self(AlignSelf::Center)
                .child(Text::new("Test")),
        ),
    );
    assert_eq!(output, lines(&["", "Test", "",]));
}

#[test]
fn row_align_multiple_text_nodes_to_center() {
    let output = render(
        View::new().height(3).child(
            View::new()
                .align_self(AlignSelf::Center)
                .child(Text::new("A"))
                .child(Text::new("B")),
        ),
    );
    assert_eq!(output, lines(&["", "AB", "",]));
}

#[test]
fn row_align_text_to_bottom() {
    let output = render(
        View::new().height(3).child(
            View::new()
                .align_self(AlignSelf::FlexEnd)
                .child(Text::new("Test")),
        ),
    );
    assert_eq!(output, lines(&["", "", "Test",]));
}

#[test]
fn row_align_multiple_text_nodes_to_bottom() {
    let output = render(
        View::new().height(3).child(
            View::new()
                .align_self(AlignSelf::FlexEnd)
                .child(Text::new("A"))
                .child(Text::new("B")),
        ),
    );
    assert_eq!(output, lines(&["", "", "AB",]));
}

#[test]
fn column_align_text_to_center() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .width(10)
            .child(
                View::new()
                    .align_self(AlignSelf::Center)
                    .child(Text::new("Test")),
            ),
    );
    assert_eq!(output, "   Test");
}

#[test]
fn column_align_text_to_right() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .width(10)
            .child(
                View::new()
                    .align_self(AlignSelf::FlexEnd)
                    .child(Text::new("Test")),
            ),
    );
    assert_eq!(output, "      Test");
}

#[test]
fn column_align_self_stretch() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .width(7)
            .child(
                View::new()
                    .align_self(AlignSelf::Stretch)
                    .border_style(BorderStyle::Single)
                    .child(Text::new("X")),
            ),
    );
    assert_eq!(output, lines(&["┌─────┐", "│X    │", "└─────┘",]));
}

#[test]
fn row_align_self_stretch() {
    let output = render(
        View::new().height(5).child(
            View::new()
                .align_self(AlignSelf::Stretch)
                .border_style(BorderStyle::Single)
                .child(Text::new("X")),
        ),
    );
    assert_eq!(output, lines(&["┌─┐", "│X│", "│ │", "│ │", "└─┘",]));
}

#[test]
fn row_align_self_baseline() {
    let output = render(
        View::new()
            .align_items(AlignItems::FlexEnd)
            .height(3)
            .child(Text::new("A").child(Newline::new()).child("B"))
            .child(
                View::new()
                    .align_self(AlignSelf::Baseline)
                    .child(Text::new("X")),
            ),
    );
    assert_eq!(output, lines(&["AX", "B", "",]));
}
