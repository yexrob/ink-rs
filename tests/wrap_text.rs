//! Ported from ink test/wrap-text.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn changing_text_wrapping_recalculates_the_container_height() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        View::new()
            .width(7)
            .border_style(BorderStyle::Single)
            .child(Text::new("abcdefghij").wrap(if *value {
                TextWrap::TruncateEnd
            } else {
                TextWrap::Wrap
            }))
            .into()
    }
    let mut instance = testing::render(component(example, false));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&["┌─────┐", "│abcde│", "│fghij│", "└─────┘"]),
    );
    instance.rerender(component(example, true));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&["┌─────┐", "│abcd…│", "└─────┘"]),
    );
    instance.rerender(component(example, false));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&["┌─────┐", "│abcde│", "│fghij│", "└─────┘"]),
    );
}

#[test]
fn truncated_multi_line_text_keeps_its_lines_in_the_layout() {
    let output = render(
        View::new()
            .width(8)
            .border_style(BorderStyle::Single)
            .child(Text::new("hello world\nfoo").wrap(TextWrap::TruncateEnd)),
    );
    assert_eq!(
        output,
        lines(&["┌──────┐", "│hello…│", "│foo   │", "└──────┘"])
    );
}

#[test]
fn truncated_multi_line_text_keeps_a_c1_sgr_color_that_spans_a_newline() {
    let output = render(
        View::new()
            .width(4)
            .child(Text::new("\u{9b}31mabcdef\nuvwxyz\u{9b}39m").wrap(TextWrap::TruncateEnd)),
    );
    assert_ansi_eq(&output, &styled(&["{fg=red}abc…{/}", "{fg=red}uvw…{/}"]));
}

#[test]
fn truncated_multi_line_text_keeps_a_c1_osc_hyperlink_that_spans_a_newline() {
    let output = render(
        View::new().width(4).child(
            Text::new("\u{9d}8;;https://example.com\u{9c}abcdef\nuvwxyz\u{9d}8;;\u{9c}")
                .wrap(TextWrap::TruncateEnd),
        ),
    );
    assert_eq!(
        output,
        lines(&[
            "\u{1b}]8;;https://example.com\u{1b}\\abc\u{1b}]8;;\u{1b}\\…",
            "\u{1b}]8;;https://example.com\u{1b}\\uvw\u{1b}]8;;\u{1b}\\…",
        ])
    );
}

#[test]
fn truncated_multi_line_text_keeps_a_colon_256_color_that_spans_a_newline() {
    let output =
        render(View::new().width(4).child(
            Text::new("\u{1b}[38:5:196mabcdef\nuvwxyz\u{1b}[39m").wrap(TextWrap::TruncateEnd),
        ));
    assert_ansi_eq(&output, &styled(&["{fg=196}abc…{/}", "{fg=196}uvw…{/}"]));
}

#[test]
fn truncated_multi_line_text_keeps_a_colon_truecolor_that_spans_a_newline() {
    let output = render(View::new().width(4).child(
        Text::new("\u{1b}[38:2::255:0:0mabcdef\nuvwxyz\u{1b}[39m").wrap(TextWrap::TruncateEnd),
    ));
    assert_ansi_eq(
        &output,
        &styled(&["{fg=#ff0000}abc…{/}", "{fg=#ff0000}uvw…{/}"]),
    );
}

#[test]
fn does_not_reuse_a_cached_result_from_a_wider_box() {
    render(
        View::new()
            .width(15)
            .child(Text::new("一二三四五六七八九十")),
    );
    let output = render(
        View::new()
            .width(5)
            .child(Text::new("一二三四五六七八九十1")),
    );
    assert_eq!(output, lines(&["一二", "三四", "五六", "七八", "九十1"]));
}
