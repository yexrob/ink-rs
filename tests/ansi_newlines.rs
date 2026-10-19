//! Ported from ink test/ansi-newlines.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn raw_ansi_foreground_continues_across_explicit_newlines() {
    let output = render_cols(Text::new("\u{1b}[31mA\nB\u{1b}[39m"), 80);
    assert_ansi_eq(&output, &styled(&["{fg=red}A{/}", "{fg=red}B{/}"]));
}

#[test]
fn raw_ansi_foreground_survives_blank_lines_and_vertical_clipping() {
    let output = render_cols(
        View::new()
            .height(1)
            .overflow(Overflow::Hidden)
            .content_offset_y(2)
            .child(Text::new("\u{1b}[31mA\n\nB\u{1b}[39m")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{fg=red}B{/}"]));
}

#[test]
fn raw_ansi_foreground_stops_at_a_reset_on_a_later_line() {
    let output = render_cols(Text::new("\u{1b}[31mA\nB\u{1b}[39m\nC"), 80);
    assert_ansi_eq(&output, &styled(&["{fg=red}A{/}", "{fg=red}B{/}", "C"]));
}

#[test]
fn raw_ansi_background_continues_across_explicit_newlines() {
    let output = render_cols(Text::new("\u{1b}[44mA\nB\u{1b}[49m"), 80);
    assert_ansi_eq(&output, &styled(&["{bg=blue}A{/}", "{bg=blue}B{/}"]));
}

#[test]
fn raw_ansi_background_survives_blank_lines_and_vertical_clipping() {
    let output = render_cols(
        View::new()
            .height(1)
            .overflow(Overflow::Hidden)
            .content_offset_y(2)
            .child(Text::new("\u{1b}[44mA\n\nB\u{1b}[49m")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}B{/}"]));
}

#[test]
fn raw_ansi_background_stops_at_a_reset_on_a_later_line() {
    let output = render_cols(Text::new("\u{1b}[44mA\nB\u{1b}[49m\nC"), 80);
    assert_ansi_eq(&output, &styled(&["{bg=blue}A{/}", "{bg=blue}B{/}", "C"]));
}

#[test]
fn raw_ansi_bold_continues_across_explicit_newlines() {
    let output = render_cols(Text::new("\u{1b}[1mA\nB\u{1b}[22m"), 80);
    assert_ansi_eq(&output, &styled(&["{bold}A{/}", "{bold}B{/}"]));
}

#[test]
fn raw_ansi_bold_survives_blank_lines_and_vertical_clipping() {
    let output = render_cols(
        View::new()
            .height(1)
            .overflow(Overflow::Hidden)
            .content_offset_y(2)
            .child(Text::new("\u{1b}[1mA\n\nB\u{1b}[22m")),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bold}B{/}"]));
}

#[test]
fn raw_ansi_bold_stops_at_a_reset_on_a_later_line() {
    let output = render_cols(Text::new("\u{1b}[1mA\nB\u{1b}[22m\nC"), 80);
    assert_ansi_eq(&output, &styled(&["{bold}A{/}", "{bold}B{/}", "C"]));
}
