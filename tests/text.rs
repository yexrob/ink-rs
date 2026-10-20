//! Ported from ink test/text.tsx @ 26d2c3f.

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

fn render_text(text: &str) -> String {
    render(View::new().child(Text::new(text)))
}

#[test]
fn text_with_undefined_children() {
    let output = render(Text::default());
    assert_eq!(output, "");
}

#[test]
fn text_with_null_children() {
    let output = render(Text::new(Element::from(None::<Text>)));
    assert_eq!(output, "");
}

#[test]
fn tabs_are_measured_and_rendered_without_overwriting_adjacent_text() {
    let output = render(
        View::new()
            .width(20)
            .child(Text::new("A").child(Text::new("\tB")))
            .child(Text::new("!")),
    );
    assert_eq!(output, "A       B!");
}

#[test]
fn expanded_tabs_participate_in_text_wrapping() {
    let output = render(
        View::new()
            .width(5)
            .flex_direction(FlexDirection::Column)
            .child(Text::new("A\tB"))
            .child(Text::new("!")),
    );
    assert_eq!(output, lines(&["A", "   B", "!",]));
}

#[test]
fn tab_stops_account_for_styled_wide_characters() {
    let output = render_truecolor(
        View::new()
            .width(20)
            .child(Text::new(Text::new("\u{754c}").color(Color::Red)).child("\tB"))
            .child(Text::new("!")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{fg=red}界{/}      B!"]));
}

#[test]
fn text_with_standard_color() {
    let output = render(Text::new("Test").color(Color::Green));
    assert_ansi_eq(&output, &styled(&["{fg=green}Test{/}"]));
}

#[test]
fn text_with_dim_bold() {
    let output = render_truecolor(Text::new("Test").dim_color(true).bold(true), 100);
    assert_eq!(strip_ansi(&output), "Test");
    assert_ne!(output, "Test");
}

#[test]
fn text_with_dimmed_color() {
    let output = render(Text::new("Test").color(Color::Green).dim_color(true));
    assert_ansi_eq(&output, &styled(&["{fg=green,dim}Test{/}"]));
}

#[test]
fn text_with_hex_color() {
    let output = render_truecolor(Text::new("Test").color(Color::Rgb(255, 136, 0)), 100);
    assert_ansi_eq(&output, &styled(&["{fg=#ff8800}Test{/}"]));
}

#[test]
fn text_with_rgb_color() {
    let output = render_truecolor(Text::new("Test").color(Color::Rgb(255, 136, 0)), 100);
    assert_ansi_eq(&output, &styled(&["{fg=#ff8800}Test{/}"]));
}

#[test]
fn text_with_ansi256_color() {
    let output = render_truecolor(Text::new("Test").color(Color::Ansi256(194)), 100);
    assert_ansi_eq(&output, &styled(&["{fg=194}Test{/}"]));
}

#[test]
fn text_with_standard_background_color() {
    let output = render(Text::new("Test").background_color(Color::Green));
    assert_ansi_eq(&output, &styled(&["{bg=green}Test{/}"]));
}

#[test]
fn text_with_hex_background_color() {
    let output = render_truecolor(
        Text::new("Test").background_color(Color::Rgb(255, 136, 0)),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=#ff8800}Test{/}"]));
}

#[test]
fn text_with_rgb_background_color() {
    let output = render_truecolor(
        Text::new("Test").background_color(Color::Rgb(255, 136, 0)),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=#ff8800}Test{/}"]));
}

#[test]
fn text_with_ansi256_background_color() {
    let output = render_truecolor(Text::new("Test").background_color(Color::Ansi256(194)), 100);
    assert_ansi_eq(&output, &styled(&["{bg=194}Test{/}"]));
}

#[test]
fn text_with_inversion() {
    let output = render(Text::new("Test").inverse(true));
    assert_ansi_eq(&output, &styled(&["{inverse}Test{/}"]));
}

#[test]
fn text_with_empty_to_nonempty_sibling_does_not_wrap() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        View::new()
            .child(Text::new(if *value { "x" } else { "" }).child("hello"))
            .into()
    }
    let mut instance = testing::render(component(example, false));
    assert_ansi_eq(instance.last_frame(), "hello");
    instance.rerender(component(example, true));
    assert_ansi_eq(instance.last_frame(), "xhello");
}

#[test]
fn remeasure_text_when_text_is_changed() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        View::new()
            .child(Text::new(if *value { "abcx" } else { "abc" }))
            .into()
    }
    let mut instance = testing::render(component(example, false));
    assert_ansi_eq(instance.last_frame(), "abc");
    instance.rerender(component(example, true));
    assert_ansi_eq(instance.last_frame(), "abcx");
}

#[test]
fn remeasure_text_when_text_nodes_are_changed() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        View::new()
            .child(Text::new("abc").child(Element::from(value.then(|| Text::new("x")))))
            .into()
    }
    let mut instance = testing::render(component(example, false));
    assert_ansi_eq(instance.last_frame(), "abc");
    instance.rerender(component(example, true));
    assert_ansi_eq(instance.last_frame(), "abcx");
}

#[test]
fn text_with_content_constructor_wraps_correctly() {
    let output = render(Text::new("constructor"));
    assert_eq!(output, "constructor");
}

#[test]
fn strip_ansi_cursor_movement_sequences_from_text() {
    let output =
        render_text("\u{1b}[1A\u{1b}[2KStarting client ... \u{1b}[32mdone\u{1b}[0m\u{1b}[1B");
    assert!(!output.contains("\u{1b}[1A"));
    assert!(!output.contains("\u{1b}[2K"));
    assert!(!output.contains("\u{1b}[1B"));
    assert_eq!(strip_ansi(&output), "Starting client ... done");
}

#[test]
fn strip_ansi_cursor_position_and_erase_sequences_from_text() {
    let output = render_text("Hello\u{1b}[5;10HWorld\u{1b}[2J!");
    assert!(!output.contains("\u{1b}[5;10H"));
    assert!(!output.contains("\u{1b}[2J"));
    assert_eq!(strip_ansi(&output), "HelloWorld!");
}

#[test]
fn preserve_sgr_color_sequences_in_text() {
    let output = render_text("\u{1b}[32mgreen\u{1b}[0m normal");
    assert_ansi_eq(&output, &styled(&["{fg=green}green{/} normal"]));
    assert_eq!(strip_ansi(&output), "green normal");
}

#[test]
fn preserve_osc_hyperlink_sequences_in_text() {
    let output = render_text("\u{1b}]8;;https://example.com\u{7}link\u{1b}]8;;\u{7}");
    assert!(output.contains("\u{1b}]8;;"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn preserve_osc_hyperlink_sequences_with_st_terminator_in_text() {
    let output = render_text("\u{1b}]8;;https://example.com\u{1b}\\link\u{1b}]8;;\u{1b}\\");
    assert!(output.contains("\u{1b}]8;;"));
    assert!(output.contains("\u{1b}\\"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn preserve_c1_osc_sequences_in_text() {
    let output = render_text("\u{9d}8;;https://example.com\u{7}link\u{9d}8;;\u{7}");
    assert!(output.contains("\u{1b}]8;;https://example.com"));
    assert!(output.contains("\u{1b}]8;;\u{7}"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn preserve_c1_osc_hyperlink_sequences_with_st_terminator_in_text() {
    let output = render_text("\u{9d}8;;https://example.com\u{1b}\\link\u{9d}8;;\u{1b}\\");
    assert!(output.contains("\u{1b}]8;;https://example.com"));
    assert!(output.contains("\u{1b}\\"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn preserve_colors_encoded_with_colon_parameters() {
    let output = render_text("A\u{1b}[38:2::255:100:0mcolor\u{1b}[0mB");
    assert_ansi_eq(&output, &styled(&["A{fg=#ff6400}color{/}B"]));
    assert_eq!(strip_ansi(&output), "AcolorB");
}

#[test]
fn strip_complete_non_sgr_csi_sequences_without_leaking_parameters() {
    let output = render_text("A\u{1b}[>4;2mB\u{1b}[2 qC");
    assert!(!output.contains("4;2m"));
    assert!(!output.contains(" q"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn strip_complete_c1_non_sgr_csi_sequences_without_leaking_parameters() {
    let output = render_text("A\u{9b}>4;2mB\u{9b}2 qC");
    assert!(!output.contains("4;2m"));
    assert!(!output.contains(" q"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn strip_complete_esc_control_sequences_with_intermediates() {
    let output = render_text("A\u{1b}#8B\u{1b}cC");
    assert!(!output.contains("\u{1b}#8"));
    assert!(!output.contains("\u{1b}c"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn strip_tmux_dcs_passthrough_wrappers_without_leaking_payload() {
    let output = render_text(
        "\u{1b}Ptmux;\u{1b}\u{1b}]8;;https://example.com\u{7}\u{1b}\\link\u{1b}Ptmux;\u{1b}\u{1b}]8;;\u{7}\u{1b}\\",
    );
    assert!(!output.contains("tmux;"));
    assert!(!output.contains("\u{1b}P"));
    assert!(!output.contains("\u{1b}\\"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn strip_tmux_dcs_passthrough_wrappers_with_st_terminated_osc_payload() {
    let output = render_text(
        "\u{1b}Ptmux;\u{1b}\u{1b}]8;;https://example.com\u{1b}\u{1b}\\\u{1b}\\link\u{1b}Ptmux;\u{1b}\u{1b}]8;;\u{1b}\u{1b}\\\u{1b}\\",
    );
    assert!(!output.contains("tmux;"));
    assert!(!output.contains("\u{1b}\\"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn strip_c1_dcs_control_strings_as_complete_units() {
    let output = render_text("A\u{90}payload\u{1b}\\B\u{90}payload\u{9c}C");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn strip_pm_and_apc_control_strings_as_complete_units() {
    let output = render_text("A\u{1b}^pm-payload\u{1b}\\B\u{1b}_apc-payload\u{1b}\\C");
    assert!(!output.contains("pm-payload"));
    assert!(!output.contains("apc-payload"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn strip_c1_pm_and_apc_control_strings_as_complete_units() {
    let output = render_text("A\u{9e}pm-payload\u{9c}B\u{9f}apc-payload\u{9c}C");
    assert!(!output.contains("pm-payload"));
    assert!(!output.contains("apc-payload"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn strip_esc_sos_control_strings_as_complete_units() {
    let output = render_text("A\u{1b}Xpayload\u{1b}\\B");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_c1_sos_control_strings_as_complete_units() {
    let output = render_text("A\u{98}payload\u{1b}\\B\u{98}payload\u{9c}C");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn strip_malformed_sos_control_strings_to_avoid_payload_leaks() {
    let output = render_text("A\u{1b}Xpayload\u{7}B\u{98}payload");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn preserve_sgr_sequences_around_stripped_sos_control_strings() {
    let output = render_text("A\u{1b}[32mgreen\u{1b}[0m\u{1b}Xpayload\u{1b}\\B");
    assert!(!output.contains("payload"));
    assert_ansi_eq(&output, &styled(&["A{fg=green}green{/}B"]));
    assert_eq!(strip_ansi(&output), "AgreenB");
}

#[test]
fn strip_tmux_dcs_passthrough_containing_bel_until_the_final_st_terminator() {
    let output = render_text("A\u{1b}Ptmux;\u{1b}\u{1b}]0;title\u{7}\u{1b}\\B");
    assert!(!output.contains("tmux;"));
    assert!(!output.contains("title"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_incomplete_dcs_passthrough_sequences_to_avoid_payload_leaks() {
    let output = render_text("\u{1b}Ptmux;\u{1b}link");
    assert!(!output.contains("tmux;"));
    assert_eq!(strip_ansi(&output), "");
}

#[test]
fn strip_incomplete_c1_dcs_control_strings_to_avoid_payload_leaks() {
    let output = render_text("A\u{90}payload");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_incomplete_osc_control_strings_to_avoid_payload_leaks() {
    let output = render_text("A\u{1b}]8;;https://example.comlink");
    assert!(!output.contains("https://example.com"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_incomplete_c1_osc_control_strings_to_avoid_payload_leaks() {
    let output = render_text("A\u{9d}8;;https://example.comlink");
    assert!(!output.contains("https://example.com"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_incomplete_esc_control_sequences_with_intermediates_to_avoid_payload_leaks() {
    let output = render_text("A\u{1b}#");
    assert!(!output.contains("\u{1b}#"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_malformed_esc_control_sequences_with_intermediates_and_non_final_bytes() {
    let output = render_text("A\u{1b}#\u{7}payload");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_standalone_st_bytes_from_text_output() {
    let output = render_text("A\u{9c}B");
    assert!(!output.contains("\u{9c}"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_standalone_c1_control_characters_from_text_output() {
    let output = render_text("A\u{85}B\u{8e}C");
    assert!(!output.contains("\u{85}"));
    assert!(!output.contains("\u{8e}"));
    assert_eq!(strip_ansi(&output), "ABC");
}
