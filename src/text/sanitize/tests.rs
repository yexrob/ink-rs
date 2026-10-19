// Ported from ink test/sanitize-ansi.ts @ 26d2c3f.

use super::*;

fn strip_ansi(text: &str) -> String {
    super::super::ansi::tokenize(text)
        .into_iter()
        .filter_map(|token| match token {
            super::super::ansi::Token::Text(value) => Some(value),
            _ => None,
        })
        .collect()
}

#[test]
fn preserve_plain_text() {
    assert_eq!(sanitize_ansi("hello"), "hello");
}

#[test]
fn preserve_sgr_sequences() {
    let input = "A\u{1b}[1;4mtext\u{1b}[0mB";
    assert_eq!(sanitize_ansi(input), input);
}

#[test]
fn normalize_colon_color_parameters_38_5_196() {
    assert_eq!(
        sanitize_ansi("\u{1b}[38:5:196mcolor\u{1b}[0m"),
        "\u{1b}[38;5;196mcolor\u{1b}[0m"
    );
}

#[test]
fn normalize_colon_color_parameters_48_5_21() {
    assert_eq!(
        sanitize_ansi("\u{1b}[48:5:21mcolor\u{1b}[0m"),
        "\u{1b}[48;5;21mcolor\u{1b}[0m"
    );
}

#[test]
fn normalize_colon_color_parameters_38_2_empty_255_0_0() {
    assert_eq!(
        sanitize_ansi("\u{1b}[38:2::255:0:0mcolor\u{1b}[0m"),
        "\u{1b}[38;2;255;0;0mcolor\u{1b}[0m"
    );
}

#[test]
fn normalize_colon_color_parameters_38_2_0_255_0_0() {
    assert_eq!(
        sanitize_ansi("\u{1b}[38:2:0:255:0:0mcolor\u{1b}[0m"),
        "\u{1b}[38;2;255;0;0mcolor\u{1b}[0m"
    );
}

#[test]
fn normalize_colon_color_parameters_38_2_255_0_0() {
    assert_eq!(
        sanitize_ansi("\u{1b}[38:2:255:0:0mcolor\u{1b}[0m"),
        "\u{1b}[38;2;255;0;0mcolor\u{1b}[0m"
    );
}

#[test]
fn normalize_colon_color_parameters_1_38_5_196_48_2_empty_0_0_255() {
    assert_eq!(
        sanitize_ansi("\u{1b}[1;38:5:196;48:2::0:0:255mcolor\u{1b}[0m"),
        "\u{1b}[1;38;5;196;48;2;0;0;255mcolor\u{1b}[0m"
    );
}

#[test]
fn normalize_underline_style_4_1() {
    assert_eq!(
        sanitize_ansi("\u{1b}[4:1mtext\u{1b}[0m"),
        "\u{1b}[4mtext\u{1b}[0m"
    );
}

#[test]
fn normalize_underline_style_4_3() {
    assert_eq!(
        sanitize_ansi("\u{1b}[4:3mtext\u{1b}[0m"),
        "\u{1b}[4mtext\u{1b}[0m"
    );
}

#[test]
fn normalize_underline_style_4_0() {
    assert_eq!(
        sanitize_ansi("\u{1b}[4:0mtext\u{1b}[0m"),
        "\u{1b}[24mtext\u{1b}[0m"
    );
}

#[test]
fn normalize_underline_style_4() {
    assert_eq!(
        sanitize_ansi("\u{1b}[4:mtext\u{1b}[0m"),
        "\u{1b}[24mtext\u{1b}[0m"
    );
}

#[test]
fn normalize_underline_style_1_4_3_31() {
    assert_eq!(
        sanitize_ansi("\u{1b}[1;4:3;31mtext\u{1b}[0m"),
        "\u{1b}[1;4;31mtext\u{1b}[0m"
    );
}

#[test]
fn drop_unsupported_colon_parameters_58_5_1() {
    assert_eq!(
        sanitize_ansi("\u{1b}[58:5:1mtext\u{1b}[0m"),
        "text\u{1b}[0m"
    );
}

#[test]
fn drop_unsupported_colon_parameters_58_2_1_2_3() {
    assert_eq!(
        sanitize_ansi("\u{1b}[58:2::1:2:3mtext\u{1b}[0m"),
        "text\u{1b}[0m"
    );
}

#[test]
fn drop_unsupported_colon_parameters_38_2_1_255_0_0() {
    assert_eq!(
        sanitize_ansi("\u{1b}[38:2:1:255:0:0mtext\u{1b}[0m"),
        "text\u{1b}[0m"
    );
}

#[test]
fn drop_unsupported_colon_parameters_38_5_196_0() {
    assert_eq!(
        sanitize_ansi("\u{1b}[38:5:196:0mtext\u{1b}[0m"),
        "text\u{1b}[0m"
    );
}

#[test]
fn keep_the_remaining_parameters_when_dropping_colon_parameters() {
    assert_eq!(
        sanitize_ansi("\u{1b}[1;58:5:1;31mtext\u{1b}[0m"),
        "\u{1b}[1;31mtext\u{1b}[0m"
    );
}

#[test]
fn preserve_empty_sgr_parameters() {
    let input = "A\u{1b}[mB\u{1b}[;1mC";
    assert_eq!(sanitize_ansi(input), input);
}

#[test]
fn defer_sgr_sequences_until_after_combining_marks() {
    assert_eq!(
        sanitize_ansi("e\u{1b}[31m\u{1b}[1m\u{301}X\u{1b}[39m"),
        "e\u{301}\u{1b}[31m\u{1b}[1mX\u{1b}[39m"
    );
}

#[test]
fn defer_a_trailing_sgr_sequence_past_a_combining_mark() {
    assert_eq!(sanitize_ansi("e\u{1b}[31m\u{301}"), "e\u{301}\u{1b}[31m");
}

#[test]
fn keep_deferred_sgr_sequences_before_a_following_hyperlink() {
    assert_eq!(
        sanitize_ansi("e\u{1b}[31m\u{301}\u{1b}]8;;https://example.com\u{1b}\\X"),
        "e\u{301}\u{1b}[31m\u{1b}]8;;https://example.com\u{1b}\\X"
    );
}

#[test]
fn strip_cursor_controls_before_checking_for_combining_marks() {
    assert_eq!(
        sanitize_ansi("e\u{1b}[31m\r\u{301}X"),
        "e\u{301}\u{1b}[31mX"
    );
}

#[test]
fn preserve_osc_hyperlinks() {
    let output = sanitize_ansi("\u{1b}]8;;https://example.com\u{1b}\\link\u{1b}]8;;\u{1b}\\");
    assert!(output.contains("\u{1b}]8;;https://example.com"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn preserve_osc_hyperlinks_terminated_by_c1_st() {
    let output = sanitize_ansi("\u{1b}]8;;https://example.com\u{9c}link\u{1b}]8;;\u{9c}");
    assert!(output.contains("\u{1b}]8;;https://example.com\u{9c}"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn preserve_c1_osc_hyperlinks_terminated_by_c1_st() {
    let input = "\u{9d}8;;https://example.com\u{9c}link\u{9d}8;;\u{9c}";
    let output = sanitize_ansi(input);
    assert!(output.contains("\u{9d}8;;https://example.com\u{9c}"));
    assert_eq!(output, input);
}

#[test]
fn preserve_c1_osc_hyperlinks_terminated_by_esc_st() {
    let input = "\u{9d}8;;https://example.com\u{1b}\\link\u{9d}8;;\u{1b}\\";
    let output = sanitize_ansi(input);
    assert!(output.contains("\u{9d}8;;https://example.com\u{1b}\\"));
    assert_eq!(output, input);
}

#[test]
fn preserve_c1_osc_hyperlinks_terminated_by_bel() {
    let input = "\u{9d}8;;https://example.com\u{7}link\u{9d}8;;\u{7}";
    let output = sanitize_ansi(input);
    assert!(output.contains("\u{9d}8;;https://example.com\u{7}"));
    assert_eq!(output, input);
}

#[test]
fn strip_non_sgr_csi_sequences_as_complete_units() {
    let output = sanitize_ansi("A\u{1b}[>4;2mB\u{1b}[2 qC");
    assert!(!output.contains("4;2m"));
    assert!(!output.contains(" q"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn strip_c1_non_sgr_csi_sequences_as_complete_units() {
    let output = sanitize_ansi("A\u{9b}>4;2mB\u{9b}2 qC");
    assert!(!output.contains("4;2m"));
    assert!(!output.contains(" q"));
    assert_eq!(strip_ansi(&output), "ABC");
}

#[test]
fn normalize_c1_sgr_csi_sequences_to_esc_form() {
    assert_eq!(
        sanitize_ansi("A\u{9b}31mgreen\u{9b}0mB"),
        "A\u{1b}[31mgreen\u{1b}[0mB"
    );
}

#[test]
fn strip_private_parameter_m_sequences_that_are_not_sgr() {
    let output = sanitize_ansi("A\u{1b}[>4;2mB");
    assert!(!output.contains("\u{1b}[>4;2m"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_tmux_dcs_passthrough_wrappers_with_escaped_st_payload_terminators() {
    let wrapped_hyperlink_start =
        "\u{1b}Ptmux;\u{1b}\u{1b}]8;;https://example.com\u{1b}\u{1b}\\\u{1b}\\";
    let wrapped_hyperlink_end = "\u{1b}Ptmux;\u{1b}\u{1b}]8;;\u{1b}\u{1b}\\\u{1b}\\";
    let output = sanitize_ansi(&format!(
        "{wrapped_hyperlink_start}link{wrapped_hyperlink_end}"
    ));
    assert!(!output.contains("tmux;"));
    assert!(!output.contains("\u{1b}P"));
    assert_eq!(strip_ansi(&output), "link");
}

#[test]
fn strip_incomplete_dcs_passthrough_sequences_to_avoid_payload_leaks() {
    let output = sanitize_ansi("A\u{1b}Ptmux;\u{1b}link");
    assert!(!output.contains("tmux;"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_dcs_control_strings_with_bel_in_payload_until_st_terminator() {
    let output = sanitize_ansi("A\u{1b}Ppayload\u{7}still-payload\u{1b}\\B");
    assert!(!output.contains("payload"));
    assert!(!output.contains("still-payload"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_esc_sos_control_strings_as_complete_units() {
    let output = sanitize_ansi("A\u{1b}Xpayload\u{1b}\\B");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_esc_sos_control_strings_with_c1_st_terminator() {
    let output = sanitize_ansi("A\u{1b}Xpayload\u{9c}B");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_c1_sos_control_strings_as_complete_units_with_c1_st_terminator() {
    let output = sanitize_ansi("A\u{98}payload\u{9c}B");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_c1_sos_control_strings_as_complete_units_with_esc_st_terminator() {
    let output = sanitize_ansi("A\u{98}payload\u{1b}\\B");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_esc_sos_with_bel_terminator_as_malformed_control_string() {
    let output = sanitize_ansi("A\u{1b}Xpayload\u{7}B");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_c1_sos_with_bel_terminator_as_malformed_control_string() {
    let output = sanitize_ansi("A\u{98}payload\u{7}B");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_incomplete_esc_sos_control_strings_to_avoid_payload_leaks() {
    let output = sanitize_ansi("A\u{1b}Xpayload");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_incomplete_c1_sos_control_strings_to_avoid_payload_leaks() {
    let output = sanitize_ansi("A\u{98}payload");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_sos_with_escaped_esc_in_payload_until_final_st_terminator() {
    let output = sanitize_ansi("A\u{1b}Xfoo\u{1b}\u{1b}\\bar\u{1b}\\B");
    assert!(!output.contains("foo"));
    assert!(!output.contains("bar"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn preserve_sgr_around_stripped_sos_control_strings() {
    let output = sanitize_ansi("A\u{1b}[31mR\u{1b}[0m\u{1b}Xpayload\u{1b}\\B");
    assert!(output.contains("\u{1b}[31m"));
    assert!(output.contains("\u{1b}[0m"));
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "ARB");
}

#[test]
fn strip_esc_st_sequences() {
    let output = sanitize_ansi("A\u{1b}\\B");
    assert!(!output.contains("\u{1b}\\"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_malformed_esc_control_sequences_with_intermediates_and_non_final_bytes() {
    let output = sanitize_ansi("A\u{1b}#\u{7}payload");
    assert!(!output.contains("payload"));
    assert_eq!(strip_ansi(&output), "A");
}

#[test]
fn strip_incomplete_csi_after_preserving_prior_sgr_content() {
    let output = sanitize_ansi("A\u{1b}[31mB\u{1b}[");
    assert!(output.contains("\u{1b}[31m"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_standalone_st_bytes() {
    let output = sanitize_ansi("A\u{9c}B");
    assert!(!output.contains("\u{9c}"));
    assert_eq!(strip_ansi(&output), "AB");
}

#[test]
fn strip_standalone_c1_control_characters() {
    let output = sanitize_ansi("A\u{85}B\u{8e}C");
    assert!(!output.contains("\u{85}"));
    assert!(!output.contains("\u{8e}"));
    assert_eq!(strip_ansi(&output), "ABC");
}

// Ported from ink test/text-controls.tsx @ 26d2c3f.

#[test]
fn sanitizeansi_strips_nul() {
    assert_eq!(sanitize_ansi("a\u{0}b"), "ab");
}

#[test]
fn sanitizeansi_strips_soh() {
    assert_eq!(sanitize_ansi("a\u{1}b"), "ab");
}

#[test]
fn sanitizeansi_strips_bel() {
    assert_eq!(sanitize_ansi("a\u{7}b"), "ab");
}

#[test]
fn sanitizeansi_strips_sub() {
    assert_eq!(sanitize_ansi("a\u{1a}b"), "ab");
}

#[test]
fn sanitizeansi_strips_us() {
    assert_eq!(sanitize_ansi("a\u{1f}b"), "ab");
}

#[test]
fn sanitizeansi_strips_del() {
    assert_eq!(sanitize_ansi("a\u{7f}b"), "ab");
}

#[test]
fn sanitizing_cursor_controls_preserves_tabs_newlines_and_styles() {
    assert_eq!(
        sanitize_ansi("\u{1b}[31ma\rb\u{8}c\u{b}d\u{c}e\t\nf\u{1b}[0m"),
        "\u{1b}[31mabcde\t\nf\u{1b}[0m"
    );
}

#[test]
fn cursor_controls_inside_osc_payloads_remain_untouched() {
    let hyperlink = "\u{1b}]8;;https://example.com/\rpath\u{7}";
    assert_eq!(
        sanitize_ansi(&format!("{hyperlink}a\rb\u{1b}]8;;\u{7}")),
        format!("{hyperlink}ab\u{1b}]8;;\u{7}")
    );
}
