//! Ported from ink test/c1-rendering.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn c1_color_preserves_layout_and_styling_at_width_4() {
    let output = render_cols(
        View::new()
            .width(4)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{9b}31mred\u{9b}0m")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌──┐", "│{fg=red}re{/}│", "│{fg=red}d{/} │", "└──┘"]),
    );
}

#[test]
fn c1_color_preserves_layout_and_styling_at_width_8() {
    let output = render_cols(
        View::new()
            .width(8)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{9b}31mred\u{9b}0m")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌──────┐", "│{fg=red}red{/}   │", "└──────┘"]),
    );
}

#[test]
fn c1_hyperlink_preserves_layout_and_styling_at_width_4() {
    let output = render_cols(
        View::new()
            .width(4)
            .border_style(BorderStyle::Single)
            .child(Text::new(
                "\u{9d}8;;https://example.com\u{9c}link\u{9d}8;;\u{9c}",
            )),
        80,
    );
    assert_eq!(
        output,
        lines(&[
            "┌──┐",
            "│\u{1b}]8;;https://example.com\u{1b}\\li\u{1b}]8;;\u{1b}\\│",
            "│\u{1b}]8;;https://example.com\u{1b}\\\u{1b}]8;;https://example.com\u{7}nk\u{1b}]8;;\u{7}\u{1b}]8;;\u{1b}\\│",
            "└──┘",
        ])
    );
}

#[test]
fn c1_hyperlink_preserves_layout_and_styling_at_width_8() {
    let output = render_cols(
        View::new()
            .width(8)
            .border_style(BorderStyle::Single)
            .child(Text::new(
                "\u{9d}8;;https://example.com\u{9c}link\u{9d}8;;\u{9c}",
            )),
        80,
    );
    assert_eq!(
        output,
        lines(&[
            "┌──────┐",
            "│\u{1b}]8;;https://example.com\u{1b}\\link\u{1b}]8;;\u{1b}\\  │",
            "└──────┘",
        ])
    );
}

#[test]
fn c1_hyperlink_with_bel_terminators_preserves_layout_and_styling_at_width_4() {
    let output = render_cols(
        View::new()
            .width(4)
            .border_style(BorderStyle::Single)
            .child(Text::new(
                "\u{9d}8;;https://example.com\u{7}link\u{9d}8;;\u{7}",
            )),
        80,
    );
    assert_eq!(
        output,
        lines(&[
            "┌──┐",
            "│\u{1b}]8;;https://example.com\u{7}li\u{1b}]8;;\u{7}│",
            "│\u{1b}]8;;https://example.com\u{7}nk\u{1b}]8;;\u{7}│",
            "└──┘",
        ])
    );
}

#[test]
fn c1_hyperlink_with_bel_terminators_preserves_layout_and_styling_at_width_8() {
    let output = render_cols(
        View::new()
            .width(8)
            .border_style(BorderStyle::Single)
            .child(Text::new(
                "\u{9d}8;;https://example.com\u{7}link\u{9d}8;;\u{7}",
            )),
        80,
    );
    assert_eq!(
        output,
        lines(&[
            "┌──────┐",
            "│\u{1b}]8;;https://example.com\u{7}link\u{1b}]8;;\u{7}  │",
            "└──────┘",
        ])
    );
}

#[test]
fn c1_mixed_color_encodings_preserves_layout_and_styling_at_width_4() {
    let output = render_cols(
        View::new()
            .width(4)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[31mred\u{9b}0m")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌──┐", "│{fg=red}re{/}│", "│{fg=red}d{/} │", "└──┘"]),
    );
}

#[test]
fn c1_mixed_color_encodings_preserves_layout_and_styling_at_width_8() {
    let output = render_cols(
        View::new()
            .width(8)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[31mred\u{9b}0m")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌──────┐", "│{fg=red}red{/}   │", "└──────┘"]),
    );
}
