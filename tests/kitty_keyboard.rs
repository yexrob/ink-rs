//! Ported from ink test/kitty-keyboard.tsx @ 26d2c3f.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ink_rs::testing::{TestApp, TestTerminal, mount};
use ink_rs::*;

fn enabled(terminal: &TestTerminal) -> TestApp {
    mount(
        Text::new("Hello"),
        Options {
            kitty_keyboard: KittyKeyboard::Enabled(KittyFlags::default()),
            ..Options::default()
        },
        terminal,
    )
}

#[test]
fn kitty_protocol_writes_enable_sequence_on_init_when_mode_is_enabled() {
    let terminal = TestTerminal::new(100, 24);
    let app = enabled(&terminal);
    assert!(terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    app.unmount();
}

#[test]
fn kitty_protocol_writes_disable_sequence_on_unmount() {
    let terminal = TestTerminal::new(100, 24);
    let app = enabled(&terminal);
    app.unmount();
    // ink-rs includes the kitty pop in the terminal restore write.
    assert!(
        terminal
            .writes()
            .iter()
            .any(|write| write.contains("\x1b[<u"))
    );
}

#[test]
fn kitty_protocol_not_enabled_when_stdin_is_not_a_tty() {
    let terminal = TestTerminal::new(100, 24).with_stdin_tty(false);
    let app = enabled(&terminal);
    assert!(!terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    app.unmount();
}

#[test]
fn kitty_protocol_not_enabled_when_stdout_is_not_a_tty() {
    let terminal = TestTerminal::new(100, 24).with_tty(false);
    let app = enabled(&terminal);
    assert!(!terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    app.unmount();
}

type Inputs = Rc<RefCell<Vec<String>>>;

fn detection(terminal: &TestTerminal, inputs: Inputs) -> (TestApp, AppHandle) {
    thread_local! {
        static HANDLE: RefCell<Option<AppHandle>> = const { RefCell::new(None) };
    }
    let app = mount(
        component(
            |cx: &mut Scope, inputs: &Inputs| {
                HANDLE.with(|handle| *handle.borrow_mut() = Some(cx.use_app()));
                let inputs = inputs.clone();
                cx.use_input(true, move |input, _| {
                    inputs.borrow_mut().push(input.to_owned())
                });
                Text::new("Hello").into()
            },
            inputs,
        ),
        Options {
            interactive: Some(true),
            kitty_keyboard: KittyKeyboard::Auto(KittyFlags::default()),
            ..Options::default()
        },
        terminal,
    );
    (
        app,
        HANDLE.with(|handle| handle.borrow_mut().take().unwrap()),
    )
}

#[test]
fn kitty_protocol_auto_detection_does_not_enable_protocol_after_unmount() {
    let terminal = TestTerminal::new(100, 24);
    let (mut app, exit) = detection(&terminal, Rc::default());
    // An exit request unmounts without consuming TestApp, allowing late input.
    exit.exit();
    app.advance(Duration::ZERO);
    app.input("\x1b[?1u");
    app.advance(Duration::from_millis(30));
    assert!(
        !terminal
            .writes()
            .iter()
            .any(|write| write.contains("\x1b[>1u"))
    );
}

#[test]
fn kitty_protocol_auto_detection_handles_synchronous_query_response() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Inputs::default();
    let (mut app, _) = detection(&terminal, inputs.clone());
    // Ink buffers the response before its reader attaches. Feed it after mount,
    // when ink-rs has enabled raw mode and can read it through the input path.
    app.input("\x1b[?1u");
    app.advance(Duration::from_millis(30));
    assert!(terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    assert!(inputs.borrow().is_empty());
}

#[test]
fn kitty_protocol_auto_detection_preserves_split_utf8_input_bytes() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Inputs::default();
    let (mut app, _) = detection(&terminal, inputs.clone());
    app.input([0xf0, 0x9f]);
    app.advance(Duration::from_millis(1));
    app.input([0x92, 0xa9]);
    app.advance(Duration::from_millis(250));
    assert_eq!(*inputs.borrow(), ["\u{1f4a9}"]);
}

#[test]
fn kitty_protocol_auto_detection_handles_split_responses_with_surrounding_input() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Inputs::default();
    let (mut app, _) = detection(&terminal, inputs.clone());
    app.input("before\x1b[?");
    app.advance(Duration::from_millis(1));
    app.input("0uafter");
    app.advance(Duration::from_millis(30));
    assert!(terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    assert_eq!(*inputs.borrow(), ["before", "after"]);
}

#[test]
fn kitty_protocol_auto_detection_timeout_preserves_query_prefix_without_digits() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Inputs::default();
    let (mut app, _) = detection(&terminal, inputs.clone());
    app.input("\x1b[?");
    app.advance(Duration::from_millis(250));
    assert!(!terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    assert_eq!(*inputs.borrow(), ["[?"]);
}

#[test]
fn kitty_protocol_auto_detection_does_not_treat_query_response_without_digits_as_a_response() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Inputs::default();
    let (mut app, _) = detection(&terminal, inputs.clone());
    app.input("\x1b[?u");
    app.advance(Duration::from_millis(250));
    assert!(!terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    assert!(inputs.borrow().is_empty());
}

#[test]
fn kitty_protocol_auto_detection_does_not_treat_invalid_query_like_sequence_as_a_response() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Inputs::default();
    let (mut app, _) = detection(&terminal, inputs.clone());
    app.input("\x1b[?1x");
    app.advance(Duration::from_millis(250));
    assert!(!terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    assert!(inputs.borrow().is_empty());
}
