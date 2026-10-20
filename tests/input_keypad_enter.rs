//! Ported from ink test/input-keypad-enter.tsx @ 26d2c3f.

use std::cell::RefCell;
use std::rc::Rc;

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn submits(sequence: &str) {
    let events = Rc::new(RefCell::new(Vec::new()));
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, events: &Rc<RefCell<Vec<(String, bool)>>>| {
                let events = events.clone();
                cx.use_input(true, move |input, key| {
                    events.borrow_mut().push((input.to_owned(), key.enter))
                });
                Element::fragment(Vec::<Element>::new())
            },
            events.clone(),
        ),
        Options::default(),
        &terminal,
    );
    app.input(sequence);
    assert_eq!(*events.borrow(), [("\r".to_owned(), true)]);
}

#[test]
fn normal_return_submits_through_use_input() {
    submits("\r");
}

#[test]
fn application_keypad_enter_submits_through_use_input() {
    submits("\x1bOM");
}
