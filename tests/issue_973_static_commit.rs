//! Ported from ink test/issue-973-static-commit.tsx @ 26d2c3f.
mod common;

use common::{Screen, assert_no_full_clear};
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;
use std::cell::Cell;
use std::time::Duration;

thread_local! { static PHASE: Cell<Option<State<u32>>> = const { Cell::new(None) }; }

fn fixture(cx: &mut Scope, _: &()) -> Element {
    let phase = cx.use_state(|| 0);
    PHASE.set(Some(phase));
    let items = if phase.get() == 0 {
        vec![]
    } else {
        vec!["line 1\nline 2\nline 3\nline 4\nline 5\nline 6"]
    };
    Element::fragment::<[Element; 2]>([
        Static::new(items, |item, _| Text::new(*item).into()).into(),
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new(if phase.get() == 2 {
                "INPUT BOX"
            } else {
                "input box"
            }))
            .into(),
    ])
}

#[test]
fn issue_973_static_item_taller_than_viewport_keeps_its_last_line() {
    let terminal = TestTerminal::new(100, 6);
    let mut app = mount(component(fixture, ()), Options::default(), &terminal);
    for phase in [1, 2] {
        PHASE.get().unwrap().set(phase);
        app.advance(Duration::from_millis(34));
    }
    let lines = Screen::new(&terminal, 100, 6).lines();
    assert!(lines.iter().any(|line| line == "line 6"));
    assert!(lines.iter().any(|line| line == "INPUT BOX"));
    assert_no_full_clear(&terminal);
}
