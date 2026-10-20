//! Ported from ink test/hooks-use-paste.tsx @ 26d2c3f.
//! PTY exited text becomes exit_result; fixture handlers exit on a match.

use std::cell::Cell;
use std::rc::Rc;

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn paste_demo(cx: &mut Scope, mode: &&'static str) -> Element {
    let mode = *mode;
    let exit = cx.use_app();
    cx.use_paste(true, move |text| {
        if matches!(
            (mode, text),
            ("basic", "hello world")
                | ("escapeSequences", "hello\x1b[Aworld")
                | ("noUseInput", "hello")
        ) {
            exit.exit();
        }
    });
    cx.use_input(mode == "noUseInput", |input, _| {
        panic!("useInput received input during paste: {input:?}")
    });
    Element::fragment(Vec::<Element>::new())
}

fn fixture(mode: &'static str, input: &str) {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(paste_demo, mode), Options::default(), &terminal);
    if mode == "basic" {
        assert!(terminal.writes().iter().any(|write| write == "\x1b[?2004h"));
    }
    app.input(input);
    assert!(matches!(app.exit_result(), Some(Ok(None))));
    if mode == "basic" {
        // ink-rs includes the paste disable in the terminal restore write.
        assert!(
            terminal
                .writes()
                .iter()
                .any(|write| write.contains("\x1b[?2004l"))
        );
    }
}

#[test]
fn use_paste_receives_bracketed_paste_as_single_text_blob() {
    fixture("basic", "\x1b[200~hello world\x1b[201~");
}

#[test]
fn use_paste_paste_content_with_escape_sequences_is_delivered_verbatim() {
    fixture("escapeSequences", "\x1b[200~hello\x1b[Aworld\x1b[201~");
}

#[test]
fn use_paste_use_input_does_not_receive_bracketed_paste_content() {
    fixture("noUseInput", "\x1b[200~hello\x1b[201~");
}

#[test]
fn use_paste_multiple_simultaneous_hooks_both_receive_the_same_paste_event() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, _: &()| {
                let received = cx.use_memo((), || Rc::new(Cell::new(0)));
                let exit = cx.use_app();
                let on_paste = move |text: &str| {
                    if text != "hello" {
                        return;
                    }
                    received.set(received.get() + 1);
                    if received.get() >= 2 {
                        exit.exit();
                    }
                };
                cx.use_paste(true, on_paste.clone());
                cx.use_paste(true, on_paste);
                Element::fragment(Vec::<Element>::new())
            },
            (),
        ),
        Options::default(),
        &terminal,
    );
    app.input("\x1b[200~hello\x1b[201~");
    assert!(matches!(app.exit_result(), Some(Ok(None))));
}
