// Ported from ink test/wrap-text.tsx @ 26d2c3f.

use super::*;

fn wrap_text(text: &str, max_width: usize, wrap: TextWrap) -> String {
    let lines = crate::text::styled::parse(text);
    super::wrap_text(&lines, max_width, wrap)
        .iter()
        .map(|line| crate::text::styled::serialize(line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn wraps_text() {
    assert_eq!(
        wrap_text("hello world", 5, TextWrap::Wrap),
        "hello\n \nworld"
    );
}

#[test]
fn truncates_text_at_the_end() {
    assert_eq!(
        wrap_text("hello world", 5, TextWrap::TruncateEnd),
        "hell\u{2026}"
    );
}

#[test]
fn truncates_each_line_of_multi_line_text_separately() {
    assert_eq!(
        wrap_text("hello world\nfoo", 6, TextWrap::TruncateEnd),
        "hello\u{2026}\nfoo"
    );
    assert_eq!(
        wrap_text("foo\nhello world", 6, TextWrap::TruncateEnd),
        "foo\nhello\u{2026}"
    );
    assert_eq!(
        wrap_text("ab\ncdefgh\nij", 4, TextWrap::TruncateEnd),
        "ab\ncde\u{2026}\nij"
    );
    assert_eq!(
        wrap_text("hello world\nfoo", 6, TextWrap::TruncateMiddle),
        "hel\u{2026}ld\nfoo"
    );
    assert_eq!(
        wrap_text("hello world\nfoo", 6, TextWrap::TruncateStart),
        "\u{2026}world\nfoo"
    );
}

#[test]
fn truncated_multi_line_text_keeps_styles_that_span_a_newline() {
    assert_eq!(
        wrap_text(
            "\u{1b}[31mabcdef\nuvwxyz\u{1b}[39m",
            4,
            TextWrap::TruncateEnd
        ),
        "\u{1b}[31mabc\u{2026}\u{1b}[39m\n\u{1b}[31muvw\u{2026}\u{1b}[39m"
    );
    let link = "\u{1b}]8;;https://example.com\u{7}";
    let link_end = "\u{1b}]8;;\u{7}";
    assert_eq!(
        wrap_text(
            &format!("{link}abcdef\nuvwxyz{link_end}"),
            4,
            TextWrap::TruncateEnd
        ),
        format!("{link}abc{link_end}\u{2026}\n{link}uvw{link_end}\u{2026}")
    );
}

#[test]
fn keeps_text_at_its_natural_width_when_there_is_no_room_to_wrap() {
    assert_eq!(wrap_text("hello", 0, TextWrap::Wrap), "hello");
    assert_eq!(wrap_text("hello", 0, TextWrap::Hard), "hello");
}

#[test]
fn leaves_truncation_at_zero_columns_alone() {
    assert_eq!(wrap_text("hello", 0, TextWrap::TruncateEnd), "");
}

#[test]
fn uses_separate_cache_entries_for_different_widths() {
    assert_eq!(
        wrap_text("hello world", 5, TextWrap::TruncateEnd),
        "hell\u{2026}"
    );
    assert_eq!(
        wrap_text("hello world", 8, TextWrap::TruncateEnd),
        "hello w\u{2026}"
    );
}

#[test]
fn uses_separate_cache_entries_for_texts_that_end_with_digits() {
    assert_eq!(wrap_text("ab", 12, TextWrap::Wrap), "ab");
    assert_eq!(wrap_text("ab1", 2, TextWrap::Wrap), "ab\n1");
}
