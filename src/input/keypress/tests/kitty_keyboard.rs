// Ported from ink test/kitty-keyboard.tsx @ 26d2c3f.
// KK-24 duplicates KK-03; KK-25 asserts sequence/raw; KK-64..75 await App wiring.

use super::*;

#[test]
fn kitty_protocol_simple_character() {
    let result = parse_keypress("\u{1b}[97u");
    assert_eq!(result.name, "a");
    assert!(!result.ctrl);
    assert!(!result.shift);
    assert!(!result.meta);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_uppercase_character_shift() {
    let result = parse_keypress("\u{1b}[97;2u");
    assert_eq!(result.name, "a");
    assert!(result.shift);
    assert!(!result.ctrl);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_ctrl_modifier() {
    let result = parse_keypress("\u{1b}[97;5u");
    assert_eq!(result.name, "a");
    assert!(result.ctrl);
    assert!(!result.shift);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_alt_option_modifier() {
    let result = parse_keypress("\u{1b}[97;3u");
    assert_eq!(result.name, "a");
    assert!(result.meta);
    assert!(!result.ctrl);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_super_modifier() {
    let result = parse_keypress("\u{1b}[97;9u");
    assert_eq!(result.name, "a");
    assert!(result.super_key);
    assert!(!result.ctrl);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_hyper_modifier() {
    let result = parse_keypress("\u{1b}[97;17u");
    assert_eq!(result.name, "a");
    assert!(result.hyper);
    assert!(!result.super_key);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_meta_modifier() {
    let result = parse_keypress("\u{1b}[97;33u");
    assert_eq!(result.name, "a");
    assert!(result.meta);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_caps_lock() {
    let result = parse_keypress("\u{1b}[97;65u");
    assert_eq!(result.name, "a");
    assert!(result.caps_lock);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_num_lock() {
    let result = parse_keypress("\u{1b}[97;129u");
    assert_eq!(result.name, "a");
    assert!(result.num_lock);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_combined_modifiers_ctrl_shift() {
    let result = parse_keypress("\u{1b}[97;6u");
    assert_eq!(result.name, "a");
    assert!(result.ctrl);
    assert!(result.shift);
    assert!(!result.meta);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_combined_modifiers_super_ctrl() {
    let result = parse_keypress("\u{1b}[115;13u");
    assert_eq!(result.name, "s");
    assert!(result.super_key);
    assert!(result.ctrl);
    assert!(!result.shift);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_escape_key() {
    let result = parse_keypress("\u{1b}[27u");
    assert_eq!(result.name, "escape");
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_return_enter_key() {
    let result = parse_keypress("\u{1b}[13u");
    assert_eq!(result.name, "return");
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_tab_key() {
    let result = parse_keypress("\u{1b}[9u");
    assert_eq!(result.name, "tab");
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_backspace_release() {
    let result = parse_keypress("\u{1b}[127;1:3u");
    assert_eq!(result.name, "backspace");
    assert_eq!(result.event_type, Some(KeyEventType::Release));
}

#[test]
fn kitty_protocol_backspace_key_codepoint_127() {
    let result = parse_keypress("\u{1b}[127u");
    assert_eq!(result.name, "backspace");
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn legacy_parser_meta_backspace_0x7f() {
    let result = parse_keypress("\u{1b}\u{7f}");
    assert_eq!(result.name, "backspace");
    assert!(result.meta);
}

#[test]
fn kitty_protocol_space_key() {
    let result = parse_keypress("\u{1b}[32u");
    assert_eq!(result.name, "space");
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_event_type_press() {
    let result = parse_keypress("\u{1b}[97;1:1u");
    assert_eq!(result.name, "a");
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_event_type_repeat() {
    let result = parse_keypress("\u{1b}[97;1:2u");
    assert_eq!(result.name, "a");
    assert_eq!(result.event_type, Some(KeyEventType::Repeat));
}

#[test]
fn kitty_protocol_event_type_release() {
    let result = parse_keypress("\u{1b}[97;1:3u");
    assert_eq!(result.name, "a");
    assert_eq!(result.event_type, Some(KeyEventType::Release));
}

#[test]
fn kitty_protocol_number_keys() {
    let result = parse_keypress("\u{1b}[49u");
    assert_eq!(result.name, "1");
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_special_character() {
    let result = parse_keypress("\u{1b}[64u");
    assert_eq!(result.name, "@");
    assert_eq!(result.event_type, Some(KeyEventType::Press));
}

#[test]
fn kitty_protocol_text_as_codepoints_field() {
    let result = parse_keypress("\u{1b}[97;2:1;65u");
    assert_eq!(result.name, "a");
    assert_eq!(result.text.as_deref(), Some("A"));
    assert!(result.shift);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_text_as_codepoints_with_multiple_codepoints() {
    let result = parse_keypress("\u{1b}[97;1:1;72:101u");
    assert_eq!(result.text.as_deref(), Some("He"));
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_supplementary_unicode_codepoint() {
    let result = parse_keypress("\u{1b}[128512u");
    assert_eq!(result.name, "\u{1f600}");
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_text_as_codepoints_with_supplementary_unicode() {
    let result = parse_keypress("\u{1b}[97;1:1;128512u");
    assert_eq!(result.text.as_deref(), Some("\u{1f600}"));
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_text_defaults_to_character_from_codepoint() {
    let result = parse_keypress("\u{1b}[97u");
    assert_eq!(result.text.as_deref(), Some("a"));
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_arrow_keys_with_event_type() {
    let up = parse_keypress("\u{1b}[1;1:1A");
    assert_eq!(up.name, "up");
    assert_eq!(up.event_type, Some(KeyEventType::Press));
    assert!(up.is_kitty_protocol);

    let down = parse_keypress("\u{1b}[1;1:3B");
    assert_eq!(down.name, "down");
    assert_eq!(down.event_type, Some(KeyEventType::Release));
    assert!(down.is_kitty_protocol);

    let right = parse_keypress("\u{1b}[1;1:2C");
    assert_eq!(right.name, "right");
    assert_eq!(right.event_type, Some(KeyEventType::Repeat));
    assert!(right.is_kitty_protocol);

    let left = parse_keypress("\u{1b}[1;1:1D");
    assert_eq!(left.name, "left");
    assert_eq!(left.event_type, Some(KeyEventType::Press));
    assert!(left.is_kitty_protocol);
}

#[test]
fn kitty_protocol_arrow_keys_with_modifiers() {
    let result = parse_keypress("\u{1b}[1;5:1A");
    assert_eq!(result.name, "up");
    assert!(result.ctrl);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_home_and_end_keys() {
    let home = parse_keypress("\u{1b}[1;1:1H");
    assert_eq!(home.name, "home");
    assert_eq!(home.event_type, Some(KeyEventType::Press));
    assert!(home.is_kitty_protocol);

    let end = parse_keypress("\u{1b}[1;1:1F");
    assert_eq!(end.name, "end");
    assert_eq!(end.event_type, Some(KeyEventType::Press));
    assert!(end.is_kitty_protocol);
}

#[test]
fn kitty_protocol_tilde_terminated_special_keys() {
    let del = parse_keypress("\u{1b}[3;1:1~");
    assert_eq!(del.name, "delete");
    assert_eq!(del.event_type, Some(KeyEventType::Press));
    assert!(del.is_kitty_protocol);

    let ins = parse_keypress("\u{1b}[2;1:1~");
    assert_eq!(ins.name, "insert");
    assert!(ins.is_kitty_protocol);

    let pgup = parse_keypress("\u{1b}[5;1:1~");
    assert_eq!(pgup.name, "pageup");
    assert!(pgup.is_kitty_protocol);

    let f5 = parse_keypress("\u{1b}[15;1:1~");
    assert_eq!(f5.name, "f5");
    assert!(f5.is_kitty_protocol);
}

#[test]
fn kitty_protocol_tilde_keys_with_modifiers() {
    let result = parse_keypress("\u{1b}[3;2:1~");
    assert_eq!(result.name, "delete");
    assert!(result.shift);
    assert_eq!(result.event_type, Some(KeyEventType::Press));
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_invalid_codepoint_above_u_10ffff_returns_safe_empty_keypress() {
    let result = parse_keypress("\u{1b}[1114112u");
    assert_eq!(result.name, "");
    assert!(!result.ctrl);
    assert!(result.is_kitty_protocol);
    assert!(!result.is_printable);
}

#[test]
fn kitty_protocol_surrogate_codepoint_returns_safe_empty_keypress() {
    let result = parse_keypress("\u{1b}[55296u");
    assert_eq!(result.name, "");
    assert!(!result.ctrl);
    assert!(result.is_kitty_protocol);
    assert!(!result.is_printable);
}

#[test]
fn kitty_protocol_invalid_text_codepoint_replaced_with_fallback() {
    let result = parse_keypress("\u{1b}[97;1:1;1114112u");
    assert_eq!(result.name, "a");
    assert_eq!(result.text.as_deref(), Some("?"));
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_malformed_modifier_0_does_not_set_all_flags() {
    let result = parse_keypress("\u{1b}[97;0u");
    assert_eq!(result.name, "a");
    assert!(!result.ctrl);
    assert!(!result.shift);
    assert!(!result.meta);
    assert!(!result.super_key);
    assert!(result.is_kitty_protocol);
}

#[test]
fn non_kitty_sequences_fall_back_to_legacy_parsing() {
    let result = parse_keypress("\u{1b}[A");
    assert_eq!(result.name, "up");
    // Ink uses undefined for absent kitty flags; Rust uses false.
    assert!(!result.is_kitty_protocol);
}

#[test]
fn non_kitty_sequences_ctrl_c() {
    let result = parse_keypress("\u{3}");
    assert_eq!(result.name, "c");
    assert!(result.ctrl);
    // Ink uses undefined for absent kitty flags; Rust uses false.
    assert!(!result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_is_printable_is_true_for_regular_characters() {
    let result = parse_keypress("\u{1b}[97u");
    assert!(result.is_printable);
}

#[test]
fn kitty_protocol_is_printable_is_true_for_digits() {
    let result = parse_keypress("\u{1b}[49u");
    assert!(result.is_printable);
}

#[test]
fn kitty_protocol_is_printable_is_true_for_symbols() {
    let result = parse_keypress("\u{1b}[64u");
    assert!(result.is_printable);
}

#[test]
fn kitty_protocol_is_printable_is_true_for_emoji() {
    let result = parse_keypress("\u{1b}[128512u");
    assert!(result.is_printable);
}

#[test]
fn kitty_protocol_is_printable_is_false_for_escape() {
    let result = parse_keypress("\u{1b}[27u");
    assert!(!result.is_printable);
}

#[test]
fn kitty_protocol_is_printable_is_true_for_return() {
    let result = parse_keypress("\u{1b}[13u");
    assert!(result.is_printable);
}

#[test]
fn kitty_protocol_is_printable_is_false_for_tab() {
    let result = parse_keypress("\u{1b}[9u");
    assert!(!result.is_printable);
}

#[test]
fn kitty_protocol_is_printable_is_true_for_space() {
    let result = parse_keypress("\u{1b}[32u");
    assert!(result.is_printable);
}

#[test]
fn kitty_protocol_is_printable_is_false_for_backspace() {
    let result = parse_keypress("\u{1b}[127u");
    assert!(!result.is_printable);
}

#[test]
fn kitty_protocol_ctrl_letters_preserve_their_input_character() {
    let result = parse_keypress("\u{1b}[97;5u");
    assert!(result.is_printable);
    assert_eq!(result.text.as_deref(), Some("a"));
}

#[test]
fn kitty_protocol_is_printable_is_false_for_special_keys_arrows() {
    let result = parse_keypress("\u{1b}[1;1:1A");
    assert!(!result.is_printable);
}

#[test]
fn kitty_protocol_capslock_57358_is_non_printable() {
    let result = parse_keypress("\u{1b}[57358u");
    assert_eq!(result.name, "capslock");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_printscreen_57361_is_non_printable() {
    let result = parse_keypress("\u{1b}[57361u");
    assert_eq!(result.name, "printscreen");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_f13_57376_is_non_printable() {
    let result = parse_keypress("\u{1b}[57376u");
    assert_eq!(result.name, "f13");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_media_key_57428_mediaplay_is_non_printable() {
    let result = parse_keypress("\u{1b}[57428u");
    assert_eq!(result.name, "mediaplay");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_modifier_only_key_57441_leftshift_is_non_printable() {
    let result = parse_keypress("\u{1b}[57441;2u");
    assert_eq!(result.name, "leftshift");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_modifier_only_key_57442_leftcontrol_is_non_printable() {
    let result = parse_keypress("\u{1b}[57442;5u");
    assert_eq!(result.name, "leftcontrol");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_kp_keys_57399_kp0_are_non_printable() {
    let result = parse_keypress("\u{1b}[57399u");
    assert_eq!(result.name, "kp0");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_scrolllock_57359_is_non_printable() {
    let result = parse_keypress("\u{1b}[57359u");
    assert_eq!(result.name, "scrolllock");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_numlock_57360_is_non_printable() {
    let result = parse_keypress("\u{1b}[57360u");
    assert_eq!(result.name, "numlock");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_pause_57362_is_non_printable() {
    let result = parse_keypress("\u{1b}[57362u");
    assert_eq!(result.name, "pause");
    assert!(!result.is_printable);
    assert!(result.is_kitty_protocol);
}

#[test]
fn kitty_protocol_volume_keys_are_non_printable() {
    let lower = parse_keypress("\u{1b}[57438u");
    assert_eq!(lower.name, "lowervolume");
    assert!(!lower.is_printable);

    let raise = parse_keypress("\u{1b}[57439u");
    assert_eq!(raise.name, "raisevolume");
    assert!(!raise.is_printable);

    let mute = parse_keypress("\u{1b}[57440u");
    assert_eq!(mute.name, "mutevolume");
    assert!(!mute.is_printable);
}

#[test]
fn kitty_protocol_space_key_has_text_field_set_to_space_character() {
    let result = parse_keypress("\u{1b}[32u");
    assert_eq!(result.text.as_deref(), Some(" "));
}

#[test]
fn kitty_protocol_return_key_has_text_field_set_to_carriage_return() {
    let result = parse_keypress("\u{1b}[13u");
    assert_eq!(result.text.as_deref(), Some("\r"));
}
