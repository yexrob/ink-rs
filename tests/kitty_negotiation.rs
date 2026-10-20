//! Ported from ink test/kitty-negotiation.tsx @ 26d2c3f.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ink_rs::testing::{TestApp, TestTerminal, mount};
use ink_rs::*;

#[test]
fn associated_text_requests_include_all_key_reporting() {
    // The public flag constants replace resolveFlags; observe the negotiated bytes.
    for (flags, expected) in [
        (KittyFlags::REPORT_ASSOCIATED_TEXT, "\x1b[>24u"),
        (
            KittyFlags::DISAMBIGUATE_ESCAPE_CODES | KittyFlags::REPORT_ASSOCIATED_TEXT,
            "\x1b[>25u",
        ),
    ] {
        let terminal = TestTerminal::new(100, 24);
        let _app = mount(
            Text::new("Hello"),
            Options {
                kitty_keyboard: KittyKeyboard::Enabled(flags),
                ..Options::default()
            },
            &terminal,
        );
        assert!(terminal.writes().iter().any(|write| write == expected));
    }
}

fn detection(terminal: &TestTerminal, inputs: Rc<RefCell<Vec<String>>>) -> TestApp {
    mount(
        component(
            |cx: &mut Scope, inputs: &Rc<RefCell<Vec<String>>>| {
                let inputs = inputs.clone();
                cx.use_input(true, move |input, _| {
                    inputs.borrow_mut().push(input.to_owned())
                });
                Element::fragment(Vec::<Element>::new())
            },
            inputs,
        ),
        Options {
            interactive: Some(true),
            kitty_keyboard: KittyKeyboard::Auto(KittyFlags::default()),
            ..Options::default()
        },
        terminal,
    )
}

#[test]
fn auto_detection_consumes_responses_without_duplicating_input() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Rc::new(RefCell::new(Vec::new()));
    let mut app = detection(&terminal, inputs.clone());
    app.input("x\x1b[?0u");
    app.advance(Duration::from_millis(30));
    assert_eq!(*inputs.borrow(), ["x"]);
    assert!(terminal.writes().iter().any(|write| write == "\x1b[>1u"));
}

#[test]
fn pasted_query_responses_remain_input_and_do_not_enable_the_protocol() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Rc::new(RefCell::new(Vec::new()));
    let mut app = detection(&terminal, inputs.clone());
    app.input("\x1b[200~\x1b[?0u\x1b[201~");
    app.advance(Duration::from_millis(30));
    assert_eq!(*inputs.borrow(), ["[?0u"]);
    assert!(!terminal.writes().iter().any(|write| write == "\x1b[>1u"));
}
