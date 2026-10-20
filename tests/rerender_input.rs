//! Ported from ink test/rerender-input.tsx @ 26d2c3f.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

type ArrowEvents = Rc<RefCell<Vec<(String, bool, bool)>>>;

#[test]
fn rerender_keeps_a_pending_escape_keypress() {
    let keys = Rc::new(RefCell::new(Vec::new()));
    let example = |cx: &mut Scope, (label, keys): &(&str, Rc<RefCell<Vec<&'static str>>>)| {
        let keys = keys.clone();
        cx.use_input(true, move |_, key| {
            keys.borrow_mut()
                .push(if key.escape { "escape" } else { "other" })
        });
        Text::new(*label).into()
    };
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(example, ("a", keys.clone())),
        Options::default(),
        &terminal,
    );
    app.input("\x1b");
    app.rerender(component(example, ("b", keys.clone())));
    app.advance(Duration::from_millis(20));
    assert_eq!(*keys.borrow(), ["escape"]);
}

#[test]
fn rerender_keeps_an_escape_sequence_that_spans_two_reads() {
    let keys = Rc::new(RefCell::new(Vec::new()));
    let example = |cx: &mut Scope, (label, keys): &(&str, ArrowEvents)| {
        let keys = keys.clone();
        cx.use_input(true, move |input, key| {
            keys.borrow_mut()
                .push((input.to_owned(), key.up_arrow, key.shift))
        });
        Text::new(*label).into()
    };
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(example, ("a", keys.clone())),
        Options::default(),
        &terminal,
    );
    app.input("\x1b[");
    app.rerender(component(example, ("b", keys.clone())));
    app.input("A");
    assert_eq!(*keys.borrow(), [("".to_owned(), true, false)]);
}

type Events = Rc<RefCell<Vec<String>>>;

#[test]
fn rerender_keeps_a_bracketed_paste_that_spans_two_reads() {
    let inputs = Events::default();
    let pastes = Events::default();
    let example = |cx: &mut Scope, (label, inputs, pastes): &(&str, Events, Events)| {
        let inputs = inputs.clone();
        cx.use_input(true, move |input, _| {
            inputs.borrow_mut().push(input.to_owned())
        });
        let pastes = pastes.clone();
        cx.use_paste(true, move |text| pastes.borrow_mut().push(text.to_owned()));
        Text::new(*label).into()
    };
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(example, ("a", inputs.clone(), pastes.clone())),
        Options::default(),
        &terminal,
    );
    app.input("\x1b[200~part1");
    app.rerender(component(example, ("b", inputs.clone(), pastes.clone())));
    app.input("part2\x1b[201~");
    assert_eq!(*pastes.borrow(), ["part1part2"]);
    assert!(inputs.borrow().is_empty());
}
