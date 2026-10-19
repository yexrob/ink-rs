//! Ported from ink test/truncate-width.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn truncate_keeps_wide_characters_that_fit() {
    let output = render_cols(
        View::new()
            .width(4)
            .child(Text::new("你好世界").wrap(TextWrap::TruncateEnd)),
        80,
    );
    assert_eq!(output, "你…");
}

#[test]
fn truncate_preserves_the_adjacent_column() {
    let output = render_cols(
        View::new()
            .width(5)
            .child(Text::new("你好世界").wrap(TextWrap::TruncateEnd))
            .child(View::new().flex_shrink(0.0).child(Text::new("|"))),
        80,
    );
    assert_eq!(output, "你… |");
}

#[test]
fn truncate_end_keeps_wide_characters_that_fit() {
    let output = render_cols(
        View::new()
            .width(4)
            .child(Text::new("你好世界").wrap(TextWrap::TruncateEnd)),
        80,
    );
    assert_eq!(output, "你…");
}

#[test]
fn truncate_end_preserves_the_adjacent_column() {
    let output = render_cols(
        View::new()
            .width(5)
            .child(Text::new("你好世界").wrap(TextWrap::TruncateEnd))
            .child(View::new().flex_shrink(0.0).child(Text::new("|"))),
        80,
    );
    assert_eq!(output, "你… |");
}

#[test]
fn truncate_middle_keeps_wide_characters_that_fit() {
    let output = render_cols(
        View::new()
            .width(4)
            .child(Text::new("你好世界").wrap(TextWrap::TruncateMiddle)),
        80,
    );
    assert_eq!(output, "你…");
}

#[test]
fn truncate_middle_preserves_the_adjacent_column() {
    let output = render_cols(
        View::new()
            .width(5)
            .child(Text::new("你好世界").wrap(TextWrap::TruncateMiddle))
            .child(View::new().flex_shrink(0.0).child(Text::new("|"))),
        80,
    );
    assert_eq!(output, "你… |");
}

#[test]
fn truncate_start_keeps_wide_characters_that_fit() {
    let output = render_cols(
        View::new()
            .width(4)
            .child(Text::new("你好世界").wrap(TextWrap::TruncateStart)),
        80,
    );
    assert_eq!(output, "…界");
}

#[test]
fn truncate_start_preserves_the_adjacent_column() {
    let output = render_cols(
        View::new()
            .width(5)
            .child(Text::new("你好世界").wrap(TextWrap::TruncateStart))
            .child(View::new().flex_shrink(0.0).child(Text::new("|"))),
        80,
    );
    assert_eq!(output, "…界 |");
}

#[test]
fn text_that_fits_does_not_reserve_extra_width() {
    let output = render_cols(
        View::new()
            .width(10)
            .child(Text::new("你好").wrap(TextWrap::TruncateMiddle))
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "你好|");
}
