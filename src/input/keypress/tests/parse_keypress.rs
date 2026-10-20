// Ported from ink test/parse-keypress.ts @ 26d2c3f.
// PK-01..03 exercise the byte overload; App decodes UTF-8 before this layer.

use super::*;

#[test]
fn kitty_functional_keys_preserve_modifiers_without_an_explicit_event_type() {
    for (sequence, name) in [
        ("\x1b[1;249A", "up"),
        ("\x1b[3;249~", "delete"),
        ("\x1b[13;249~", "f3"),
        ("\x1b[1;249P", "f1"),
    ] {
        let key = parse_keypress(sequence);
        assert_eq!(key.name, name);
        assert!(key.super_key);
        assert!(key.hyper);
        assert!(key.meta);
        assert!(key.caps_lock);
        assert!(key.num_lock);
        assert!(!key.ctrl);
        assert!(!key.shift);
        assert_eq!(key.event_type, Some(KeyEventType::Press));
        assert!(!key.is_printable);
    }
    let key = parse_keypress("\x1b[1;9A");
    assert!(key.super_key);
    assert!(!key.meta);
}

#[test]
fn kitty_modifiers_and_event_types_agree_across_key_encodings() {
    for bit in 0..8 {
        for (event_code, event_type) in [
            ("", KeyEventType::Press),
            (":1", KeyEventType::Press),
            (":2", KeyEventType::Repeat),
            (":3", KeyEventType::Release),
        ] {
            let modifiers = format!("{}{event_code}", (1 << bit) + 1);
            for sequence in [
                format!("\x1b[97;{modifiers}u"),
                format!("\x1b[1;{modifiers}A"),
                format!("\x1b[3;{modifiers}~"),
            ] {
                let key = parse_keypress(&sequence);
                let flags = [
                    key.shift,
                    key.meta,
                    key.ctrl,
                    key.super_key,
                    key.hyper,
                    key.meta,
                    key.caps_lock,
                    key.num_lock,
                ];
                assert!(flags[bit], "Sequence: {sequence}");
                assert_eq!(key.event_type, Some(event_type), "Sequence: {sequence}");
            }
        }
    }
}

#[test]
fn kitty_keypad_begin_supports_both_functional_encodings() {
    for sequence in ["\x1b[1E", "\x1b[57427~", "\x1b[1;1:2E", "\x1b[57427;1:2~"] {
        assert_eq!(parse_keypress(sequence).name, "clear");
    }
}

#[test]
fn kitty_text_only_events_have_no_key_identity() {
    let key = parse_keypress("\x1b[0;;229:128169u");
    assert_eq!(key.name, "");
    assert_eq!(key.text.as_deref(), Some("\u{e5}\u{1f4a9}"));
    assert!(key.is_printable);
}

#[test]
fn kitty_unknown_functional_and_control_codes_do_not_invent_printable_characters() {
    for codepoint in [0, 1, 3, 31, 128, 159, 57_344, 63_743] {
        let key = parse_keypress(&format!("\x1b[{codepoint}u"));
        assert_eq!(key.name, "");
        assert!(!key.is_printable);
        assert_eq!(key.text, None);
    }
    let key = parse_keypress("\x1b[63744u");
    assert_eq!(key.text.as_deref(), Some("\u{f900}"));
    assert!(key.is_printable);
}

#[test]
fn legacy_ctrl_space_preserves_the_space_key_and_modifiers() {
    for sequence in ["\0", "\x1b\0"] {
        let key = parse_keypress(sequence);
        assert_eq!(key.name, "space");
        assert!(key.ctrl);
        assert_eq!(key.meta, sequence.len() == 2);
        assert!(!key.shift);
        assert_eq!(key.sequence, sequence);
    }
}

#[test]
fn kitty_keypad_navigation_keys_use_standard_key_names() {
    for (codepoint, name) in [
        (57_417, "left"),
        (57_418, "right"),
        (57_419, "up"),
        (57_420, "down"),
        (57_421, "pageup"),
        (57_422, "pagedown"),
        (57_423, "home"),
        (57_424, "end"),
        (57_425, "insert"),
        (57_426, "delete"),
    ] {
        let sequence = format!("\x1b[{codepoint};5:2u");
        let key = parse_keypress(&sequence);
        assert_eq!(key.name, name);
        assert!(key.ctrl);
        assert_eq!(key.event_type, Some(KeyEventType::Repeat));
        assert!(key.is_kitty_protocol);
        assert!(!key.is_printable);
        assert_eq!(key.text, None);
        assert_eq!(key.sequence, sequence);
    }
}

#[test]
fn legacy_ctrl_punctuation_shortcuts_preserve_the_key_and_modifiers() {
    for (character, name) in [("\x1c", "\\"), ("\x1d", "]"), ("\x1e", "^"), ("\x1f", "_")] {
        for prefix in ["", "\x1b"] {
            let sequence = format!("{prefix}{character}");
            let key = parse_keypress(&sequence);
            assert_eq!(key.name, name);
            assert!(key.ctrl);
            assert_eq!(key.meta, !prefix.is_empty());
            assert!(!key.shift);
            assert_eq!(key.sequence, sequence);
        }
    }
}

#[test]
fn kitty_alternate_key_codes_preserve_the_primary_key_event() {
    for sequence in [
        "\x1b[97:65;6:2;65u",
        "\x1b[97:65:113;6:2;65u",
        "\x1b[97::113;6:2;65u",
    ] {
        let key = parse_keypress(sequence);
        assert_eq!(key.name, "a");
        assert_eq!(key.text.as_deref(), Some("A"));
        assert!(key.ctrl);
        assert!(key.shift);
        assert_eq!(key.event_type, Some(KeyEventType::Repeat));
        assert!(key.is_kitty_protocol);
        assert_eq!(key.sequence, sequence);
    }
    let key = parse_keypress("\x1b[1092::97;5u");
    assert_eq!(key.name, "\u{444}");
    assert_eq!(key.text.as_deref(), Some("\u{444}"));
    assert!(key.ctrl);
    assert!(!key.shift);
    assert_eq!(key.event_type, Some(KeyEventType::Press));
    assert!(key.is_kitty_protocol);
}

#[test]
fn kitty_keypad_associated_text_is_printable_input() {
    for (sequence, name, text) in [
        ("\x1b[57399;129;48u", "kp0", "0"),
        ("\x1b[57413;1;43u", "kpadd", "+"),
    ] {
        let key = parse_keypress(sequence);
        assert_eq!(key.name, name);
        assert_eq!(key.text.as_deref(), Some(text));
        assert!(key.is_printable);
        assert!(key.is_kitty_protocol);
    }
}

#[test]
fn kitty_keypad_enter_produces_a_return_key_and_carriage_return_text() {
    for (sequence, shift, event_type) in [
        ("\x1b[57414u", false, KeyEventType::Press),
        ("\x1b[57414;2:2u", true, KeyEventType::Repeat),
    ] {
        let key = parse_keypress(sequence);
        assert_eq!(key.name, "return");
        assert_eq!(key.text.as_deref(), Some("\r"));
        assert!(key.is_printable);
        assert!(key.is_kitty_protocol);
        assert_eq!(key.sequence, sequence);
        assert_eq!(key.shift, shift);
        assert_eq!(key.event_type, Some(event_type));
    }
}

#[test]
fn application_keypad_enter_maps_to_return_with_carriage_return_sequence() {
    for (sequence, meta) in [("\x1bOM", false), ("\x1b\x1bOM", true)] {
        let key = parse_keypress(sequence);
        assert_eq!(key.name, "return");
        assert_eq!(key.sequence, "\r");
        // Ink also asserts raw === undefined. raw is not read by the pipeline;
        // sequence retains the carriage return used by input mapping.
        assert_eq!(key.meta, meta);
        assert!(!key.ctrl);
        assert!(!key.shift);
    }
}

#[test]
fn meta_tab_preserves_the_tab_key_and_meta_modifier() {
    let key = parse_keypress("\x1b\t");
    assert_eq!(key.name, "tab");
    assert!(key.meta);
    assert!(!key.ctrl);
    assert!(!key.shift);
    assert_eq!(key.sequence, "\x1b\t");
}

#[test]
fn ctrl_meta_letters_preserve_both_modifiers_and_the_letter_name() {
    for (control_character, name) in [("\x02", "b"), ("\x06", "f"), ("\x18", "x")] {
        let sequence = format!("\x1b{control_character}");
        let key = parse_keypress(&sequence);
        assert_eq!(key.name, name);
        assert!(key.ctrl);
        assert!(key.meta);
        assert!(!key.shift);
        assert_eq!(key.sequence, sequence);
    }
}

#[test]
fn kitty_associated_text_accepts_an_omitted_modifier_value() {
    let key = parse_keypress("\x1b[0;;229u");
    assert!(key.is_kitty_protocol);
    assert!(key.is_printable);
    assert_eq!(key.text.as_deref(), Some("\u{e5}"));
    assert_eq!(key.event_type, Some(KeyEventType::Press));
    assert!(!key.ctrl);
    assert!(!key.meta);
    assert!(!key.shift);
}

#[test]
fn meta_modifier_is_recognized_for_punctuation_and_unicode_characters() {
    for character in [".", ",", "/", "-", "\u{e9}", "\u{1f600}"] {
        let sequence = format!("\x1b{character}");
        let key = parse_keypress(&sequence);
        assert!(key.meta, "Meta modifier for {character}");
        assert_eq!(key.name, character);
        assert_eq!(key.sequence, sequence);
        assert!(!key.ctrl);
        assert!(!key.shift);
    }
}

#[test]
fn incomplete_and_multi_character_sequences_are_not_meta_characters() {
    for sequence in ["\x1b[", "\x1b[1;", "\x1bhello"] {
        assert!(!parse_keypress(sequence).meta, "Sequence {sequence}");
    }
}

#[test]
fn ctrl_f1_resolves_to_name_f1() {
    let key = parse_keypress("\x1b[1;5P");
    assert_eq!(key.name, "f1");
    assert!(key.ctrl);
    assert!(!key.shift);
    assert!(!key.meta);
}

#[test]
fn ctrl_f2_resolves_to_name_f2() {
    let key = parse_keypress("\x1b[1;5Q");
    assert_eq!(key.name, "f2");
    assert!(key.ctrl);
}

#[test]
fn ctrl_f3_resolves_to_name_f3() {
    let key = parse_keypress("\x1b[1;5R");
    assert_eq!(key.name, "f3");
    assert!(key.ctrl);
}

#[test]
fn ctrl_f4_resolves_to_name_f4() {
    let key = parse_keypress("\x1b[1;5S");
    assert_eq!(key.name, "f4");
    assert!(key.ctrl);
}

#[test]
fn unmapped_ctrl_sequence_returns_empty_name() {
    let key = parse_keypress("\x1b[1;5I");
    assert_eq!(key.name, "");
    assert!(key.ctrl);
}

#[test]
fn another_unmapped_ctrl_sequence_returns_empty_name() {
    let key = parse_keypress("\x1b[1;5X");
    assert_eq!(key.name, "");
    assert!(key.ctrl);
}

#[test]
fn shift_f1_resolves_to_name_f1_with_shift() {
    let key = parse_keypress("\x1b[1;2P");
    assert_eq!(key.name, "f1");
    assert!(key.shift);
    assert!(!key.ctrl);
}
