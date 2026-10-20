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
