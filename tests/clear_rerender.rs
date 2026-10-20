//! Ported from ink test/clear-rerender.tsx @ 26d2c3f.
mod common;

use common::Screen;
use ink_rs::{
    Element, Options, Scope, State, Text, component,
    testing::{TestTerminal, mount},
};
use std::cell::Cell;
use std::time::Duration;

thread_local! { static COUNT: Cell<Option<State<u32>>> = const { Cell::new(None) }; }

fn hello(cx: &mut Scope, _: &()) -> Element {
    let count = cx.use_state(|| 0);
    COUNT.set(Some(count));
    Text::new("Hello").into()
}

#[test]
fn rerender_restores_unchanged_content_after_clear_standard() {
    let terminal = TestTerminal::new(80, 8);
    let mut app = mount(component(hello, ()), Options::default(), &terminal);
    app.clear();
    app.rerender(component(hello, ()));
    app.advance(Duration::from_millis(34));
    assert_eq!(Screen::new(&terminal, 80, 8).rows()[0], "Hello");
}

#[test]
fn state_update_restores_unchanged_content_after_clear_standard() {
    let terminal = TestTerminal::new(80, 8);
    let mut app = mount(component(hello, ()), Options::default(), &terminal);
    app.clear();
    COUNT.get().unwrap().update(|n| *n += 1);
    app.advance(Duration::from_millis(34));
    assert_eq!(Screen::new(&terminal, 80, 8).rows()[0], "Hello");
}

#[test]
fn unmount_does_not_restore_cleared_content_standard() {
    let terminal = TestTerminal::new(80, 8);
    let mut app = mount(Text::new("Hello"), Options::default(), &terminal);
    app.clear();
    app.unmount();
    assert!(
        Screen::new(&terminal, 80, 8)
            .lines()
            .iter()
            .all(String::is_empty)
    );
}
