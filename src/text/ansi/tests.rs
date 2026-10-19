// Ported from ink test/ansi-tokenizer.ts @ 26d2c3f.

use super::*;

#[test]
fn tokenize_plain_text() {
    assert_eq!(tokenize("hello"), vec![Token::Text("hello")]);
}

#[test]
fn tokenize_esc_csi_sgr_sequence() {
    assert_eq!(
        tokenize("A\u{1b}[31mB"),
        vec![
            Token::Text("A"),
            Token::Csi {
                value: "\u{1b}[31m",
                params: "31",
                intermediates: "",
                final_char: 'm'
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_c1_csi_sequence() {
    let tokens = tokenize("A\u{9b}2 qB");
    assert_eq!(
        tokens[1],
        Token::Csi {
            value: "\u{9b}2 q",
            params: "2",
            intermediates: " ",
            final_char: 'q'
        }
    );
}

#[test]
fn tokenize_osc_control_string_with_st_terminator() {
    assert_eq!(
        tokenize("A\u{1b}]8;;https://example.com\u{1b}\\B"),
        vec![
            Token::Text("A"),
            Token::ControlString {
                kind: ControlStringKind::Osc,
                value: "\u{1b}]8;;https://example.com\u{1b}\\"
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_tmux_dcs_passthrough_as_one_control_string_token() {
    let tokens =
        tokenize("A\u{1b}Ptmux;\u{1b}\u{1b}]8;;https://example.com\u{1b}\u{1b}\\\u{1b}\\B");
    let [
        Token::Text(_),
        Token::ControlString {
            kind: ControlStringKind::Dcs,
            value,
        },
        Token::Text(_),
    ] = tokens.as_slice()
    else {
        panic!("unexpected tokens: {tokens:?}");
    };
    assert!(value.starts_with("\u{1b}Ptmux;"));
    assert!(value.ends_with("\u{1b}\\"));
}

#[test]
fn tokenize_incomplete_csi_as_invalid_and_stop() {
    assert_eq!(
        tokenize("A\u{1b}["),
        vec![Token::Text("A"), Token::Invalid("\u{1b}[")]
    );
}

#[test]
fn tokenize_incomplete_esc_intermediate_sequence_as_invalid_and_stop() {
    assert_eq!(
        tokenize("A\u{1b}#"),
        vec![Token::Text("A"), Token::Invalid("\u{1b}#")]
    );
}

#[test]
fn ignore_lone_esc_before_non_final_byte() {
    assert_eq!(
        tokenize("A\u{1b}\u{7}B"),
        vec![Token::Text("A"), Token::Text("\u{7}B")]
    );
}

#[test]
fn tokenize_esc_st_sequence_as_esc_token() {
    assert_eq!(
        tokenize("A\u{1b}\\B"),
        vec![
            Token::Text("A"),
            Token::Esc {
                value: "\u{1b}\\",
                intermediates: "",
                final_char: '\\'
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_c1_osc_with_c1_st_terminator() {
    assert_eq!(
        tokenize("A\u{9d}8;;https://example.com\u{9c}B"),
        vec![
            Token::Text("A"),
            Token::ControlString {
                kind: ControlStringKind::Osc,
                value: "\u{9d}8;;https://example.com\u{9c}"
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_c1_osc_with_esc_st_terminator() {
    assert_eq!(
        tokenize("A\u{9d}8;;https://example.com\u{1b}\\B"),
        vec![
            Token::Text("A"),
            Token::ControlString {
                kind: ControlStringKind::Osc,
                value: "\u{9d}8;;https://example.com\u{1b}\\"
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_c1_sgr_csi_sequence() {
    assert_eq!(
        tokenize("A\u{9b}31mB"),
        vec![
            Token::Text("A"),
            Token::Csi {
                value: "\u{9b}31m",
                params: "31",
                intermediates: "",
                final_char: 'm'
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_incomplete_c1_csi_as_invalid_and_stop() {
    assert_eq!(
        tokenize("A\u{9b}31"),
        vec![Token::Text("A"), Token::Invalid("\u{9b}31")]
    );
}

#[test]
fn tokenize_incomplete_c1_osc_as_invalid_and_stop() {
    assert_eq!(
        tokenize("A\u{9d}8;;https://example.com"),
        vec![
            Token::Text("A"),
            Token::Invalid("\u{9d}8;;https://example.com")
        ]
    );
}

#[test]
fn tokenize_dcs_with_bel_in_payload_until_st_terminator() {
    let tokens = tokenize("A\u{1b}Ppayload\u{7}still-payload\u{1b}\\B");
    let [
        Token::Text(_),
        Token::ControlString {
            kind: ControlStringKind::Dcs,
            value,
        },
        Token::Text(_),
    ] = tokens.as_slice()
    else {
        panic!("unexpected tokens: {tokens:?}");
    };
    assert!(value.contains("\u{7}"));
    assert!(value.ends_with("\u{1b}\\"));
}

#[test]
fn tokenize_c1_osc_control_string_with_bel_terminator() {
    assert_eq!(
        tokenize("A\u{9d}8;;https://example.com\u{7}B"),
        vec![
            Token::Text("A"),
            Token::ControlString {
                kind: ControlStringKind::Osc,
                value: "\u{9d}8;;https://example.com\u{7}"
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_esc_sos_control_string_with_st_terminator() {
    assert_eq!(
        tokenize("A\u{1b}Xpayload\u{1b}\\B"),
        vec![
            Token::Text("A"),
            Token::ControlString {
                kind: ControlStringKind::Sos,
                value: "\u{1b}Xpayload\u{1b}\\"
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_esc_sos_control_string_with_c1_st_terminator() {
    assert_eq!(
        tokenize("A\u{1b}Xpayload\u{9c}B"),
        vec![
            Token::Text("A"),
            Token::ControlString {
                kind: ControlStringKind::Sos,
                value: "\u{1b}Xpayload\u{9c}"
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_c1_sos_control_string_with_c1_st_terminator() {
    assert_eq!(
        tokenize("A\u{98}payload\u{9c}B"),
        vec![
            Token::Text("A"),
            Token::ControlString {
                kind: ControlStringKind::Sos,
                value: "\u{98}payload\u{9c}"
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_c1_sos_control_string_with_esc_st_terminator() {
    assert_eq!(
        tokenize("A\u{98}payload\u{1b}\\B"),
        vec![
            Token::Text("A"),
            Token::ControlString {
                kind: ControlStringKind::Sos,
                value: "\u{98}payload\u{1b}\\"
            },
            Token::Text("B")
        ]
    );
}

#[test]
fn tokenize_esc_sos_with_bel_terminator_as_invalid_and_stop() {
    assert_eq!(
        tokenize("A\u{1b}Xpayload\u{7}B"),
        vec![Token::Text("A"), Token::Invalid("\u{1b}Xpayload\u{7}B")]
    );
}

#[test]
fn tokenize_c1_sos_with_bel_terminator_as_invalid_and_stop() {
    assert_eq!(
        tokenize("A\u{98}payload\u{7}B"),
        vec![Token::Text("A"), Token::Invalid("\u{98}payload\u{7}B")]
    );
}

#[test]
fn tokenize_incomplete_c1_sos_as_invalid_and_stop() {
    assert_eq!(
        tokenize("A\u{98}payload"),
        vec![Token::Text("A"), Token::Invalid("\u{98}payload")]
    );
}

#[test]
fn tokenize_incomplete_esc_sos_as_invalid_and_stop() {
    assert_eq!(
        tokenize("A\u{1b}Xpayload"),
        vec![Token::Text("A"), Token::Invalid("\u{1b}Xpayload")]
    );
}

#[test]
fn tokenize_sos_with_escaped_esc_in_payload_until_final_st_terminator() {
    let tokens = tokenize("A\u{1b}Xfoo\u{1b}\u{1b}\\bar\u{1b}\\B");
    let [
        Token::Text(_),
        Token::ControlString {
            kind: ControlStringKind::Sos,
            value,
        },
        Token::Text(_),
    ] = tokens.as_slice()
    else {
        panic!("unexpected tokens: {tokens:?}");
    };
    assert!(value.contains("\u{1b}\u{1b}\\"));
    assert!(value.ends_with("\u{1b}\\"));
}

#[test]
fn tokenize_standalone_c1_controls_as_c1_tokens() {
    assert_eq!(
        tokenize("A\u{85}B\u{8e}C"),
        vec![
            Token::Text("A"),
            Token::C1("\u{85}"),
            Token::Text("B"),
            Token::C1("\u{8e}"),
            Token::Text("C")
        ]
    );
}
