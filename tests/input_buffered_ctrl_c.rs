//! Ported from ink test/input-buffered-ctrl-c.tsx @ 26d2c3f.
//! The step after input replaces the setImmediate wait.

use std::cell::RefCell;
use std::rc::Rc;

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn buffered(input: &str) {
    let inputs = Rc::new(RefCell::new(Vec::new()));
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, inputs: &Rc<RefCell<Vec<String>>>| {
                let inputs = inputs.clone();
                cx.use_input(true, move |value, key| {
                    inputs.borrow_mut().push(if key.ctrl {
                        format!("Ctrl+{value}")
                    } else {
                        value.to_owned()
                    })
                });
                Element::fragment(Vec::<Element>::new())
            },
            inputs.clone(),
        ),
        Options::default(),
        &terminal,
    );
    app.input(input);
    assert!(matches!(app.exit_result(), Some(Ok(None))));
    assert!(!inputs.borrow().iter().any(|input| input == "Ctrl+c"));
}

#[test]
fn ctrl_c_exits_when_buffered_with_text_ctrl_c() {
    buffered("\x03");
}

#[test]
fn ctrl_c_exits_when_buffered_with_text_ab_ctrl_c() {
    buffered("ab\x03");
}

#[test]
fn ctrl_c_exits_when_buffered_with_text_ctrl_c_ab() {
    buffered("\x03ab");
}

type Events = Rc<RefCell<Vec<String>>>;

fn respects(is_bracketed_paste: bool) {
    let inputs = Events::default();
    let pastes = Events::default();
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, (inputs, pastes): &(Events, Events)| {
                let inputs = inputs.clone();
                cx.use_input(true, move |value, key| {
                    inputs.borrow_mut().push(if key.ctrl {
                        format!("Ctrl+{value}")
                    } else {
                        value.to_owned()
                    });
                });
                let pastes = pastes.clone();
                cx.use_paste(true, move |value| {
                    pastes.borrow_mut().push(value.to_owned())
                });
                Element::fragment(Vec::<Element>::new())
            },
            (inputs.clone(), pastes.clone()),
        ),
        Options {
            exit_on_ctrl_c: is_bracketed_paste,
            ..Options::default()
        },
        &terminal,
    );
    app.input(if is_bracketed_paste {
        "\x1b[200~ab\x03cd\x1b[201~"
    } else {
        "ab\x03cd"
    });
    assert!(app.exit_result().is_none());
    assert_eq!(
        *inputs.borrow(),
        if is_bracketed_paste {
            vec![]
        } else {
            vec!["ab", "Ctrl+c", "cd"]
        }
    );
    assert_eq!(
        *pastes.borrow(),
        if is_bracketed_paste {
            vec!["ab\x03cd"]
        } else {
            vec![]
        }
    );
}

#[test]
fn buffered_ctrl_c_respects_exit_on_ctrl_c_false() {
    respects(false);
}

#[test]
fn buffered_ctrl_c_respects_bracketed_paste() {
    respects(true);
}
