//! Ported from ink test/absolute-truncation.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn truncate_renders_text_in_an_absolute_box_without_an_explicit_width() {
    let output = render_cols(
        View::new().height(2).child(
            View::new()
                .position(Position::Absolute)
                .child(Text::new("hello").wrap(TextWrap::TruncateEnd)),
        ),
        80,
    );
    assert_eq!(output, lines(&["hello", "",]));
}

#[test]
fn truncate_start_renders_text_in_an_absolute_box_without_an_explicit_width() {
    let output = render_cols(
        View::new().height(2).child(
            View::new()
                .position(Position::Absolute)
                .child(Text::new("hello").wrap(TextWrap::TruncateStart)),
        ),
        80,
    );
    assert_eq!(output, lines(&["hello", "",]));
}

#[test]
fn truncate_middle_renders_text_in_an_absolute_box_without_an_explicit_width() {
    let output = render_cols(
        View::new().height(2).child(
            View::new()
                .position(Position::Absolute)
                .child(Text::new("hello").wrap(TextWrap::TruncateMiddle)),
        ),
        80,
    );
    assert_eq!(output, lines(&["hello", "",]));
}

#[test]
fn truncate_end_renders_text_in_an_absolute_box_without_an_explicit_width() {
    let output = render_cols(
        View::new().height(2).child(
            View::new()
                .position(Position::Absolute)
                .child(Text::new("hello").wrap(TextWrap::TruncateEnd)),
        ),
        80,
    );
    assert_eq!(output, lines(&["hello", "",]));
}
