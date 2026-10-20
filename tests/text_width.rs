//! Ported from ink test/text-width.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

fn strip_ansi(input: &str) -> String {
    let mut chars = input.chars().peekable();
    let mut output = String::new();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            output.push(c);
            continue;
        }
        match chars.next() {
            Some('[') => {
                for c in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            Some(']') => {
                while let Some(c) = chars.next() {
                    if c == '\u{7}' {
                        break;
                    }
                    if c == '\u{1b}' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    output
}

// These width assertions use ASCII, Hiragana, CJK, Katakana, and an ellipsis.
fn fixture_width(text: &str) -> usize {
    strip_ansi(text)
        .chars()
        .map(|c| match c {
            '\u{3040}'..='\u{30ff}' | '\u{4e00}'..='\u{9fff}' => 2,
            _ => 1,
        })
        .sum()
}

#[test]
fn zero_width_spaces_do_not_overwrite_box_borders() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Single)
            .width(7)
            .child(Text::new("he\u{200b}llo")),
    );
    assert_eq!(
        output.replace('\u{200b}', ""),
        lines(&["┌─────┐", "│hello│", "└─────┘",])
    );
}

#[test]
fn zero_width_overlay_does_not_erase_wide_characters() {
    let output = render(
        View::new()
            .width(4)
            .height(1)
            .child(Text::new("你好"))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(1)
                    .child(Text::new("\u{200b}")),
            ),
    );
    assert_eq!(output.replace('\u{200b}', ""), "你好");
}

#[test]
fn combining_marks_and_joined_emoji_retain_their_graphemes() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Single)
            .width(5)
            .child(Text::new("e\u{301}👩\u{200d}💻")),
    );
    assert_eq!(output, lines(&["┌───┐", "│é👩‍💻│", "└───┘",]));
}

#[test]
fn crlf_line_endings_do_not_overwrite_box_borders() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Single)
            .width(7)
            .child(Text::new("hello\r\nworld")),
    );
    assert_eq!(
        output,
        lines(&["┌─────┐", "│hello│", "│world│", "└─────┘",])
    );
}

#[test]
fn wide_characters_do_not_add_extra_space_inside_fixed_width_box() {
    let output = render(
        View::new().flex_direction(FlexDirection::Column).children([
            View::new()
                .child(View::new().width(2).child(Text::new("\u{1f354}")))
                .child(Text::new("|")),
            View::new()
                .child(View::new().width(2).child(Text::new("\u{23f3}")))
                .child(Text::new("|")),
        ]),
    );
    let rows: Vec<_> = output.split('\n').collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], "🍔|");
    assert_eq!(rows[1], "⏳|");
}

#[test]
fn cjk_characters_occupy_correct_width_in_fixed_width_box() {
    let output = render(
        View::new()
            .child(View::new().width(4).child(Text::new("\u{4f60}\u{597d}")))
            .child(Text::new("|")),
    );
    assert_eq!(output, "你好|");
}

#[test]
fn mixed_ascii_and_wide_characters_align_correctly() {
    let output = render(
        View::new().flex_direction(FlexDirection::Column).children([
            View::new()
                .child(View::new().width(6).child(Text::new("ab\u{1f354}cd")))
                .child(Text::new("|")),
            View::new()
                .child(View::new().width(6).child(Text::new("abcdef")))
                .child(Text::new("|")),
        ]),
    );
    let rows: Vec<_> = output.split('\n').collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], "ab🍔cd|");
    assert_eq!(rows[1], "abcdef|");
}

#[test]
fn ansi_styled_text_does_not_affect_layout_width() {
    let output = render(
        View::new()
            .child(
                View::new()
                    .width(5)
                    .child(Text::new("hello").color(Color::Red)),
            )
            .child(Text::new("|")),
    );
    assert_eq!(strip_ansi(&output), "hello|");
}

#[test]
fn empty_text_does_not_affect_sibling_layout() {
    let output = render(View::new().child(Text::default()).child(Text::new("hello")));
    assert_eq!(output, "hello");
}

#[test]
fn truncate_cjk_text_at_end() {
    let output = render(
        View::new()
            .width(20)
            .child(Text::new("あいうえおかきくけこ|end").wrap(TextWrap::TruncateEnd)),
    );
    assert!(fixture_width(&output) <= 20);
}

#[test]
fn truncate_cjk_text_in_the_middle() {
    let output = render(
        View::new()
            .width(20)
            .child(Text::new("あいうえおかきくけこ|end").wrap(TextWrap::TruncateMiddle)),
    );
    assert!(fixture_width(&output) <= 20);
}

#[test]
fn truncate_cjk_text_at_start() {
    let output = render(
        View::new()
            .width(20)
            .child(Text::new("あいうえおかきくけこ|end").wrap(TextWrap::TruncateStart)),
    );
    assert!(fixture_width(&output) <= 20);
}

#[test]
fn truncate_cjk_text_does_not_exceed_box_width() {
    let output = render(
        View::new()
            .child(
                View::new()
                    .width(20)
                    .child(Text::new("あいうえおかきくけこ|end").wrap(TextWrap::TruncateEnd)),
            )
            .child(Text::new("|")),
    );
    let rows: Vec<_> = output.split('\n').collect();
    assert_eq!(rows.len(), 1);
    assert!(strip_ansi(rows[0]).ends_with('|'));
}

#[test]
fn overlay_on_2nd_cell_of_cjk_character_clears_the_full_character() {
    let output = render_cols(
        View::new()
            .width(20)
            .height(1)
            .child(Text::new("あいうえおかきくけこ"))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(9)
                    .child(Text::new("XYZ")),
            ),
        20,
    );
    let first = output.split('\n').next().unwrap();
    assert_eq!(fixture_width(first), 20);
    assert_eq!(strip_ansi(first), "あいうえ XYZきくけこ");
}

#[test]
fn overlay_on_1st_cell_of_cjk_character_clears_trailing_placeholder() {
    let output = render_cols(
        View::new()
            .width(20)
            .height(1)
            .child(Text::new("あいうえおかきくけこ"))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(10)
                    .child(Text::new("X")),
            ),
        20,
    );
    let first = output.split('\n').next().unwrap();
    assert_eq!(fixture_width(first), 20);
    assert_eq!(strip_ansi(first), "あいうえおX きくけこ");
}

#[test]
fn cjk_overlay_on_2nd_cell_of_cjk_clears_both_sides() {
    let output = render_cols(
        View::new()
            .width(20)
            .height(1)
            .child(Text::new("あいうえおかきくけこ"))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(5)
                    .child(Text::new("\u{6f22}\u{5b57}\u{30c6}\u{30b9}\u{30c8}")),
            ),
        20,
    );
    let first = output.split('\n').next().unwrap();
    assert_eq!(fixture_width(first), 20);
    assert_eq!(strip_ansi(first), "あい 漢字テスト けこ");
}

#[test]
fn clipped_empty_write_does_not_corrupt_existing_wide_characters() {
    let output = render_cols(
        View::new()
            .width(4)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .child(Text::new("あい"))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(-1)
                    .width(1)
                    .child(Text::new("Z")),
            ),
        4,
    );
    assert_eq!(strip_ansi(&output), "あい");
}

#[test]
fn wide_characters_crossing_the_left_output_edge_preserve_visible_columns() {
    let output = render(
        View::new()
            .width(4)
            .height(1)
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(-1)
                    .child(Text::new("好AB")),
            )
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(3)
                    .child(Text::new("!")),
            ),
    );
    assert_eq!(output, " AB!");
}
