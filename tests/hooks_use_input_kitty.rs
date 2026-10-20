//! Ported from ink test/hooks-use-input-kitty.tsx @ 26d2c3f.
//! PTY exited text/status becomes exit_result; fixture mismatches still panic.

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn fixture(mode: &'static str, bytes: &str) {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, mode: &&str| {
                let mode = *mode;
                let exit = cx.use_app();
                cx.use_input(true, move |input, key| {
                    let matched = match mode {
                        "specialModifiers" => {
                            input.is_empty()
                                && key.up_arrow
                                && key.super_key
                                && !key.meta
                                && key.event_type == Some(KeyEventType::Press)
                        }
                        "keypadNavigation" => {
                            input.is_empty()
                                && key.down_arrow
                                && key.ctrl
                                && key.event_type == Some(KeyEventType::Repeat)
                        }
                        "alternateKeys" => {
                            input == "A"
                                && key.ctrl
                                && key.shift
                                && key.event_type == Some(KeyEventType::Repeat)
                        }
                        "keypadText" => input == "0" && key.num_lock,
                        "keypadEnter" => input == "\r" && key.enter,
                        "associatedText" => {
                            input == "\u{e5}"
                                && key.event_type == Some(KeyEventType::Press)
                                && !key.ctrl
                                && !key.meta
                                && !key.shift
                        }
                        "super" => input == "s" && key.super_key,
                        "hyper" => input == "h" && key.hyper,
                        "capsLock" => key.caps_lock,
                        "numLock" => key.num_lock,
                        "capsLockText" => input == "A" && key.caps_lock && !key.shift,
                        "superCtrl" => input == "s" && key.super_key && key.ctrl,
                        "press" => input == "a" && key.event_type == Some(KeyEventType::Press),
                        "repeat" => key.event_type == Some(KeyEventType::Repeat),
                        "release" => key.event_type == Some(KeyEventType::Release),
                        "escapeKitty" => key.escape,
                        "backspace" => input.is_empty() && key.backspace,
                        "delete" => input.is_empty() && key.delete,
                        "nonPrintable" => input.is_empty(),
                        "space" => input == " ",
                        "returnKey" => input == "\r",
                        "ctrlLetter" => input == "a" && key.ctrl,
                        _ => false,
                    };
                    if matched {
                        exit.exit();
                    } else {
                        panic!("Unexpected input: {input:?} {key:?}");
                    }
                });
                Element::fragment(Vec::<Element>::new())
            },
            mode,
        ),
        Options::default(),
        &terminal,
    );
    app.input(bytes);
    assert!(matches!(app.exit_result(), Some(Ok(None))));
}

#[test]
fn use_input_handle_kitty_special_key_modifiers_without_event_type() {
    fixture("specialModifiers", "\x1b[1;9A");
}

#[test]
fn use_input_handle_kitty_keypad_navigation() {
    fixture("keypadNavigation", "\x1b[57420;5:2u");
}

#[test]
fn use_input_handle_kitty_alternate_key_codes() {
    fixture("alternateKeys", "\x1b[97:65:113;6:2;65u");
}

#[test]
fn use_input_handle_kitty_keypad_associated_text() {
    fixture("keypadText", "\x1b[57399;129;48u");
}

#[test]
fn use_input_handle_kitty_keypad_enter() {
    fixture("keypadEnter", "\x1b[57414u");
}

#[test]
fn use_input_kitty_associated_text_with_omitted_modifiers() {
    fixture("associatedText", "\x1b[0;;229u");
}

#[test]
fn use_input_handle_kitty_protocol_super_modifier() {
    fixture("super", "\x1b[115;9u");
}

#[test]
fn use_input_handle_kitty_protocol_hyper_modifier() {
    fixture("hyper", "\x1b[104;17u");
}

#[test]
fn use_input_handle_kitty_protocol_capslock() {
    fixture("capsLock", "\x1b[97;65u");
}

#[test]
fn use_input_handle_kitty_protocol_numlock() {
    fixture("numLock", "\x1b[97;129u");
}

#[test]
fn use_input_kitty_capslock_text_does_not_imply_shift() {
    fixture("capsLockText", "\x1b[97;65;65u");
}

#[test]
fn use_input_handle_kitty_protocol_super_ctrl() {
    fixture("superCtrl", "\x1b[115;13u");
}

#[test]
fn use_input_handle_kitty_protocol_press_event() {
    fixture("press", "\x1b[97;1:1u");
}

#[test]
fn use_input_handle_kitty_protocol_repeat_event() {
    fixture("repeat", "\x1b[97;1:2u");
}

#[test]
fn use_input_handle_kitty_protocol_release_event() {
    fixture("release", "\x1b[97;1:3u");
}

#[test]
fn use_input_handle_kitty_protocol_escape_key() {
    fixture("escapeKitty", "\x1b[27u");
}

#[test]
fn use_input_handle_kitty_protocol_backspace_codepoint_127() {
    fixture("backspace", "\x1b[127u");
}

#[test]
fn use_input_handle_kitty_protocol_delete() {
    fixture("delete", "\x1b[3;1:1~");
}

#[test]
fn use_input_non_printable_kitty_key_capslock_produces_empty_input() {
    fixture("nonPrintable", "\x1b[57358u");
}

#[test]
fn use_input_non_printable_kitty_key_f13_produces_empty_input() {
    fixture("nonPrintable", "\x1b[57376u");
}

#[test]
fn use_input_non_printable_kitty_key_printscreen_produces_empty_input() {
    fixture("nonPrintable", "\x1b[57361u");
}

#[test]
fn use_input_kitty_protocol_space_key_produces_space_input() {
    fixture("space", "\x1b[32u");
}

#[test]
fn use_input_kitty_protocol_return_key_produces_carriage_return_input() {
    fixture("returnKey", "\x1b[13u");
}

#[test]
fn use_input_kitty_protocol_ctrl_letter_produces_input() {
    fixture("ctrlLetter", "\x1b[97;5u");
}
