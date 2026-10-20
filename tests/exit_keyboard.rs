//! Ported from ink test/exit-keyboard.tsx @ 26d2c3f.
//! exit_result().is_some() replaces the waitUntilExit promise observer.

use std::cell::Cell;
use std::rc::Rc;

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn automatic(input: &str, exit_on_ctrl_c: Option<bool>, should_exit: bool) {
    let received = Rc::new(Cell::new(false));
    let terminal = TestTerminal::new(100, 24);
    let mut options = Options::default();
    if let Some(value) = exit_on_ctrl_c {
        options.exit_on_ctrl_c = value;
    }
    let mut app = mount(
        component(
            |cx: &mut Scope, received: &Rc<Cell<bool>>| {
                let received = received.clone();
                cx.use_input(true, move |_, _| received.set(true));
                Element::fragment(Vec::<Element>::new())
            },
            received.clone(),
        ),
        options,
        &terminal,
    );
    app.input(input);
    assert_eq!(app.exit_result().is_some(), should_exit);
    assert_eq!(received.get(), !should_exit);
}

#[test]
fn automatic_exit_handles_legacy_ctrl_c() {
    automatic("\x03", Some(true), true);
}
#[test]
fn automatic_exit_handles_legacy_ctrl_c_with_an_undefined_exit_option() {
    automatic("\x03", None, true);
}
#[test]
fn automatic_exit_handles_kitty_ctrl_c() {
    automatic("\x1b[99;5u", Some(true), true);
}
#[test]
fn automatic_exit_handles_kitty_ctrl_c_repeat() {
    automatic("\x1b[99;5:2u", Some(true), true);
}
#[test]
fn automatic_exit_handles_kitty_ctrl_c_release() {
    automatic("\x1b[99;5:3u", Some(true), false);
}
#[test]
fn automatic_exit_handles_kitty_ctrl_c_with_automatic_exit_disabled() {
    automatic("\x1b[99;5u", Some(false), false);
}
#[test]
fn automatic_exit_handles_kitty_c_without_ctrl() {
    automatic("\x1b[99u", Some(true), false);
}
