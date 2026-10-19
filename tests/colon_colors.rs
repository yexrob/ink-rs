//! Ported from ink test/colon-colors.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn colon_color_38_5_196_preserves_layout_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .width(6)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[38:5:196mRed\u{1b}[0m"))
            .child(Text::new("X")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["┌────┐", "│{fg=196}Red{/}X│", "└────┘"]));
}

#[test]
fn colon_color_48_5_21_preserves_layout_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .width(6)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[48:5:21mRed\u{1b}[0m"))
            .child(Text::new("X")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["┌────┐", "│{bg=21}Red{/}X│", "└────┘"]));
}

#[test]
fn colon_color_38_2_255_0_0_preserves_layout_beside_a_sibling_empty_colorspace() {
    let output = render_cols(
        View::new()
            .width(6)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[38:2::255:0:0mRed\u{1b}[0m"))
            .child(Text::new("X")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌────┐", "│{fg=#ff0000}Red{/}X│", "└────┘"]),
    );
}

#[test]
fn colon_color_48_2_0_0_255_preserves_layout_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .width(6)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[48:2::0:0:255mRed\u{1b}[0m"))
            .child(Text::new("X")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌────┐", "│{bg=#0000ff}Red{/}X│", "└────┘"]),
    );
}

#[test]
fn colon_color_38_2_255_0_0_preserves_layout_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .width(6)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[38:2:255:0:0mRed\u{1b}[0m"))
            .child(Text::new("X")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌────┐", "│{fg=#ff0000}Red{/}X│", "└────┘"]),
    );
}

#[test]
fn colon_color_38_2_0_255_0_0_preserves_layout_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .width(6)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[38:2:0:255:0:0mRed\u{1b}[0m"))
            .child(Text::new("X")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌────┐", "│{fg=#ff0000}Red{/}X│", "└────┘"]),
    );
}

#[test]
fn colon_color_1_38_5_196_48_2_0_0_255_preserves_layout_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .width(6)
            .border_style(BorderStyle::Single)
            .child(Text::new("\u{1b}[1;38:5:196;48:2::0:0:255mRed\u{1b}[0m"))
            .child(Text::new("X")),
        80,
    );
    assert_ansi_eq(
        &output,
        &styled(&["┌────┐", "│{fg=196,bg=#0000ff,bold}Red{/}X│", "└────┘"]),
    );
}

#[test]
fn colon_colors_wrap_and_reset_like_semicolon_colors() {
    let output = render_cols(
        View::new()
            .width(2)
            .child(Text::new("\u{1b}[38:5:196mABCD\u{1b}[0mE")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{fg=196}AB{/}", "{fg=196}CD{/}", "E"]));
}

#[test]
fn color_normalization_leaves_colon_containing_hyperlink_targets_intact() {
    let output = render_cols(
        Text::new(
            "\u{1b}[38:5:196m\u{1b}]8;;https://example.com/38:5:196\u{1b}\\Link\u{1b}]8;;\u{1b}\\\u{1b}[0m",
        ),
        80,
    );
    assert!(output.contains("https://example.com/38:5:196"));
    assert_ansi_eq(
        &output,
        &styled(&[
            "\u{1b}]8;;https://example.com/38:5:196\u{1b}\\{fg=196}Link\u{1b}]8;;\u{1b}\\{/}",
        ]),
    );
}

#[test]
fn underline_style_4_3_renders_as_plain_underline_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .child(
                View::new()
                    .width(6)
                    .child(Text::new("\u{1b}[4:3mab\u{1b}[0m")),
            )
            .child(Text::new("|")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{underline}ab{/}    |"]));
}

#[test]
fn underline_style_reset_4_0_turns_the_underline_off() {
    let output = render_cols(
        View::new()
            .child(
                View::new()
                    .width(6)
                    .child(Text::new("\u{1b}[4:3ma\u{1b}[4:0mb")),
            )
            .child(Text::new("|")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{underline}a{/}b    |"]));
}

#[test]
fn underline_color_58_5_1_is_dropped_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .child(
                View::new()
                    .width(6)
                    .child(Text::new("\u{1b}[58:5:1mab\u{1b}[59m")),
            )
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "ab    |");
}

#[test]
fn underline_color_58_5_1_is_dropped_inside_a_clipped_box() {
    let output = render_cols(
        View::new()
            .width(1)
            .overflow(Overflow::Hidden)
            .child(Text::new("\u{1b}[58:5:1mab\u{1b}[59m")),
        80,
    );
    assert_eq!(output, lines(&["a", "b",]));
}

#[test]
fn underline_color_58_2_1_2_3_is_dropped_beside_a_sibling() {
    let output = render_cols(
        View::new()
            .child(
                View::new()
                    .width(6)
                    .child(Text::new("\u{1b}[58:2::1:2:3mab\u{1b}[59m")),
            )
            .child(Text::new("|")),
        80,
    );
    assert_eq!(output, "ab    |");
}

#[test]
fn underline_color_58_2_1_2_3_is_dropped_inside_a_clipped_box() {
    let output = render_cols(
        View::new()
            .width(1)
            .overflow(Overflow::Hidden)
            .child(Text::new("\u{1b}[58:2::1:2:3mab\u{1b}[59m")),
        80,
    );
    assert_eq!(output, lines(&["a", "b",]));
}

#[test]
fn underline_style_4_3_is_clipped_like_plain_underline() {
    let output = render_cols(
        View::new()
            .width(1)
            .overflow(Overflow::Hidden)
            .child(Text::new("\u{1b}[4:3mab\u{1b}[4:0m")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{underline}a{/}", "{underline}b{/}"]));
}
