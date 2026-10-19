//! Ported from ink test/text-controls.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn text_removes_cursor_control_r() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\rb")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_removes_cursor_control_b() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{8}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_removes_cursor_control_u000b() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{b}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_removes_cursor_control_f() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{c}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_removes_nul_inside_a_bordered_box() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{0}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_with_nul_keeps_the_sibling_in_the_next_column() {
    let output = render_cols(
        View::new()
            .child(View::new().width(10).child(Text::new("a\u{0}b")))
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "ab        |");
}

#[test]
fn text_removes_soh_inside_a_bordered_box() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{1}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_with_soh_keeps_the_sibling_in_the_next_column() {
    let output = render_cols(
        View::new()
            .child(View::new().width(10).child(Text::new("a\u{1}b")))
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "ab        |");
}

#[test]
fn text_removes_bel_inside_a_bordered_box() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{7}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_with_bel_keeps_the_sibling_in_the_next_column() {
    let output = render_cols(
        View::new()
            .child(View::new().width(10).child(Text::new("a\u{7}b")))
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "ab        |");
}

#[test]
fn text_removes_sub_inside_a_bordered_box() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{1a}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_with_sub_keeps_the_sibling_in_the_next_column() {
    let output = render_cols(
        View::new()
            .child(View::new().width(10).child(Text::new("a\u{1a}b")))
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "ab        |");
}

#[test]
fn text_removes_us_inside_a_bordered_box() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{1f}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_with_us_keeps_the_sibling_in_the_next_column() {
    let output = render_cols(
        View::new()
            .child(View::new().width(10).child(Text::new("a\u{1f}b")))
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "ab        |");
}

#[test]
fn text_removes_del_inside_a_bordered_box() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(6)
            .child(Text::new("a\u{7f}b")),
        80,
    );
    assert_eq!(output, lines(&["┌────┐", "│ab  │", "└────┘"]));
}

#[test]
fn text_with_del_keeps_the_sibling_in_the_next_column() {
    let output = render_cols(
        View::new()
            .child(View::new().width(10).child(Text::new("a\u{7f}b")))
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "ab        |");
}
