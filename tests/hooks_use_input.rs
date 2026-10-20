//! Ported from ink test/hooks-use-input.tsx @ 26d2c3f.
//! PTY exited text/status becomes exit_result; fixture mismatches still panic.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

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
                        "ctrlSpace" => input == " " && key.ctrl && !key.meta,
                        "ctrlPunctuation" => input == "_" && key.ctrl && !key.meta,
                        "lowercase" => input == "q",
                        "uppercase" => (input == "Q" && key.shift) || (input == "\r" && !key.shift),
                        "pastedCarriageReturn" => input == "\rtest",
                        "pastedTab" => input == "\ttest",
                        "bracketedPaste" => input == "hello",
                        "escape" => key.escape,
                        "escapeNoMeta" => key.escape && !key.meta,
                        "ctrl" => input == "f" && key.ctrl,
                        "meta" => input == "m" && key.meta,
                        "ctrlMeta" => input == "b" && key.ctrl && key.meta,
                        "metaBackspace" => input.is_empty() && key.meta && key.backspace,
                        "escapeBracketPrefix" => input == "[" && !key.meta,
                        "metaUpperO" => input == "O" && key.meta,
                        "tab" => input.is_empty() && key.tab && !key.ctrl,
                        "metaTab" => input.is_empty() && key.tab && key.meta && !key.ctrl,
                        "shiftTab" => input.is_empty() && key.tab && key.shift,
                        "backspace" => input.is_empty() && key.backspace,
                        "delete" => input.is_empty() && key.delete,
                        "returnMeta" => key.enter && key.meta,
                        "ctrlF1" => input.is_empty() && key.ctrl,
                        "unmappedCtrlSequence" => input == "q",
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
    if matches!(bytes, "\x1b" | "\x1b[" | "\x1bO") {
        app.advance(Duration::from_millis(20));
    }
    assert!(matches!(app.exit_result(), Some(Ok(None))));
}

#[test]
fn use_input_handle_legacy_ctrl_space() {
    fixture("ctrlSpace", "\0");
}

#[test]
fn use_input_handle_ctrl_punctuation_shortcut() {
    fixture("ctrlPunctuation", "\x1f");
}

#[test]
fn use_input_handle_lowercase_character() {
    fixture("lowercase", "q");
}

#[test]
fn use_input_handle_uppercase_character() {
    fixture("uppercase", "Q");
}

#[test]
fn use_input_carriage_return_should_not_count_as_an_uppercase_character() {
    fixture("uppercase", "\r");
}

#[test]
fn use_input_pasted_carriage_return() {
    fixture("pastedCarriageReturn", "\rtest");
}

#[test]
fn use_input_pasted_tab() {
    fixture("pastedTab", "\ttest");
}

#[test]
fn use_input_receives_bracketed_paste_when_no_use_paste_handler_is_active() {
    fixture("bracketedPaste", "\x1b[200~hello\x1b[201~");
}

#[test]
fn use_input_handle_escape() {
    fixture("escape", "\x1b");
}

#[test]
fn use_input_escape_does_not_set_meta() {
    fixture("escapeNoMeta", "\x1b");
}

#[test]
fn use_input_handle_ctrl() {
    fixture("ctrl", "\x06");
}

#[test]
fn use_input_handle_meta() {
    fixture("meta", "\x1bm");
}

#[test]
fn use_input_handle_ctrl_meta_letter() {
    fixture("ctrlMeta", "\x1b\x02");
}

#[test]
fn use_input_handle_meta_backspace_0x7f() {
    fixture("metaBackspace", "\x1b\x7f");
}

#[test]
fn use_input_flushes_esc_bracket_prefix_as_literal_input() {
    fixture("escapeBracketPrefix", "\x1b[");
}

#[test]
fn use_input_handle_meta_o_with_pending_flush() {
    fixture("metaUpperO", "\x1bO");
}

#[test]
fn use_input_handle_tab() {
    fixture("tab", "\t");
}

#[test]
fn use_input_handle_meta_tab() {
    fixture("metaTab", "\x1b\t");
}

#[test]
fn use_input_handle_shift_tab() {
    fixture("shiftTab", "\x1b[Z");
}

#[test]
fn use_input_handle_backspace() {
    fixture("backspace", "\x08");
}

#[test]
fn use_input_handle_delete() {
    fixture("delete", "\x1b[3~");
}

#[test]
fn use_input_handle_option_return_macos() {
    fixture("returnMeta", "\x1b\r");
}

#[test]
fn use_input_handle_ctrl_f1_without_crashing() {
    fixture("ctrlF1", "\x1b[1;5P");
}

#[test]
fn use_input_handle_unmapped_ctrl_escape_sequence_without_crashing() {
    fixture("unmappedCtrlSequence", "\x1b[1;5Iq");
}

fn drops(sequence: &str) {
    let events = Rc::new(RefCell::new(Vec::new()));
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, events: &Rc<RefCell<Vec<(String, bool)>>>| {
                let events = events.clone();
                cx.use_input(true, move |input, key| {
                    events.borrow_mut().push((input.to_owned(), key.escape))
                });
                Element::fragment(Vec::<Element>::new())
            },
            events.clone(),
        ),
        Options::default(),
        &terminal,
    );
    app.input(sequence);
    app.input("q");
    assert_eq!(*events.borrow(), [("q".to_owned(), false)]);
}

#[test]
fn use_input_drops_unmapped_control_sequence_focus_in() {
    drops("\x1b[I");
}

#[test]
fn use_input_drops_unmapped_control_sequence_focus_out() {
    drops("\x1b[O");
}

#[test]
fn use_input_drops_unmapped_control_sequence_cursor_position_report() {
    drops("\x1b[24;80R");
}

#[test]
fn use_input_drops_unmapped_control_sequence_sgr_mouse_report() {
    drops("\x1b[<0;10;20M");
}

#[test]
fn use_input_drops_unmapped_control_sequence_primary_device_attributes() {
    drops("\x1b[?62;1;4c");
}

#[test]
fn use_input_drops_unmapped_control_sequence_stray_bracketed_paste_end() {
    drops("\x1b[201~");
}

#[test]
fn use_input_drops_unmapped_control_sequence_unmapped_legacy_csi() {
    drops("\x1b[[Z");
}

#[test]
fn use_input_drops_unmapped_control_sequence_unmapped_modified_ss3() {
    drops("\x1bO1;5Z");
}
