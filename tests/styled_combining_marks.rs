//! Ported from ink test/styled-combining-marks.tsx @ 26d2c3f.

mod common;

use common::*;
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

#[test]
fn combining_mark_survives_a_nested_text_style_boundary_acute() {
    let output = render_truecolor(
        View::new()
            .child(Text::new("e").child(Text::new("\u{301}").color(Color::Red)))
            .child(Text::new("X")),
        80,
    );
    assert_eq!(strip_ansi(&output), "éX");
}

#[test]
fn combining_mark_survives_a_nested_text_style_boundary_diaeresis() {
    let output = render_truecolor(
        View::new()
            .child(Text::new("e").child(Text::new("\u{308}").color(Color::Red)))
            .child(Text::new("X")),
        80,
    );
    assert_eq!(strip_ansi(&output), "ëX");
}

#[test]
fn following_text_retains_its_color_after_a_styled_combining_mark() {
    let output = render_truecolor(
        Text::new("e")
            .child(Text::new("\u{301}X").color(Color::Red))
            .child("Y"),
        80,
    );
    assert_ansi_eq(&output, &styled(&["é{fg=red}X{/}Y"]));
}

#[test]
fn a_combining_mark_retains_the_style_of_its_base_character() {
    let output = render_truecolor(
        Text::new(Text::new("e").color(Color::Blue)).child(Text::new("\u{301}X").color(Color::Red)),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{fg=blue}é{fg=red}X{/}"]));
}

#[test]
fn multiple_styled_marks_attach_to_one_base_cell() {
    let output = render_truecolor(
        View::new()
            .child(
                Text::new("a")
                    .child(Text::new("\u{301}").bold(true))
                    .child(Text::new("\u{308}").color(Color::Red)),
            )
            .child(Text::new("X")),
        80,
    );
    assert_eq!(strip_ansi(&output), "á̈X");
}

#[test]
fn newlines_remain_between_a_base_and_a_following_combining_mark() {
    let output = render_truecolor(
        Text::new("e\n").child(Text::new("\u{301}X").color(Color::Red)),
        80,
    );
    assert!(strip_ansi(&output).starts_with("e\n"));
}

#[test]
fn wrapping_keeps_a_styled_accent_on_the_preceding_base_character() {
    let output = render_truecolor(
        View::new()
            .width(1)
            .child(Text::new("e").child(Text::new("\u{301}X").color(Color::Red))),
        80,
    );
    let normalized = strip_ansi(&output).replace("e\u{301}", "\u{e9}");
    assert_eq!(normalized, lines(&["é", "X",]));
}

#[test]
fn a_styled_variation_selector_keeps_emoji_presentation_and_width() {
    let output = render_truecolor(
        View::new()
            .child(Text::new("\u{2764}").child(Text::new("\u{fe0f}").color(Color::Red)))
            .child(Text::new("X")),
        80,
    );
    assert_eq!(strip_ansi(&output), "❤️X");
}

#[test]
fn a_styled_mark_without_a_base_character_keeps_its_position_and_style() {
    let output = render_truecolor(
        Text::new(Text::new("\u{301}X").color(Color::Red)).child("Y"),
        80,
    );
    assert_ansi_eq(&output, &styled(&["́{fg=red}X{/}Y"]));
}
