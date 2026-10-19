// Ported from ink test/measure-text.tsx @ 26d2c3f.

use super::*;

fn measure_text(text: &str) -> TextSize {
    super::measure_text(&crate::text::styled::parse(text))
}

#[test]
fn measure_single_word() {
    assert_eq!(
        measure_text("constructor"),
        TextSize {
            width: 11,
            height: 1
        }
    );
}

#[test]
fn measure_empty_string() {
    assert_eq!(
        measure_text(""),
        TextSize {
            width: 0,
            height: 0
        }
    );
}

#[test]
fn measure_multiline_text() {
    assert_eq!(
        measure_text("hello\nworld"),
        TextSize {
            width: 5,
            height: 2
        }
    );
}

#[test]
fn measure_multiline_text_with_varying_line_lengths() {
    assert_eq!(
        measure_text("a\nfoo\nhi"),
        TextSize {
            width: 3,
            height: 3
        }
    );
}

#[test]
fn measure_text_with_trailing_newline() {
    assert_eq!(
        measure_text("hello\n"),
        TextSize {
            width: 5,
            height: 2
        }
    );
}

#[test]
fn measure_text_with_only_newlines() {
    assert_eq!(
        measure_text("\n\n"),
        TextSize {
            width: 0,
            height: 3
        }
    );
}

#[test]
fn measure_text_with_ansi_escape_sequences() {
    assert_eq!(
        measure_text("\u{1b}[31mred\u{1b}[0m"),
        TextSize {
            width: 3,
            height: 1
        }
    );
}

#[test]
fn measure_text_with_256_color_ansi() {
    assert_eq!(
        measure_text("\u{1b}[38;5;196mred\u{1b}[0m"),
        TextSize {
            width: 3,
            height: 1
        }
    );
}

#[test]
fn measure_text_with_wide_characters() {
    assert_eq!(
        measure_text("\u{4f60}\u{597d}"),
        TextSize {
            width: 4,
            height: 1
        }
    );
}

#[test]
fn measure_text_with_emoji() {
    assert_eq!(
        measure_text("\u{1f354}"),
        TextSize {
            width: 2,
            height: 1
        }
    );
}

#[test]
fn measure_multiline_with_wide_characters() {
    assert_eq!(
        measure_text("\u{1f354}\u{1f35f}\nabc"),
        TextSize {
            width: 4,
            height: 2
        }
    );
}
