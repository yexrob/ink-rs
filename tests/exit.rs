//! Ported from ink test/exit.tsx @ 26d2c3f.
mod common;

use common::to_markup;
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::{Element, Error, Options, Scope, Static, Text, component};

// The remaining PTY fixture tests belong to a later phase.

#[test]
fn exit_on_exit_with_error_and_static_output() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let handle = cx.use_app();
        cx.use_effect((), move || handle.exit_with_error("errored"));
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["A", "B", "C"], |item, _| Text::new(*item).key(*item)).into(),
            Text::new("Dynamic").into(),
        ])
    }
    let terminal = TestTerminal::new(100, 24);
    let app = mount(component(example, ()), Options::default(), &terminal);
    // The Ink fixture prints the caught error. Read the result in process.
    let Err(Error::App(error)) = app.exit_result().unwrap() else {
        panic!("expected an app error");
    };
    assert_eq!(error.to_string(), "errored");
    let output = terminal.writes().join("");
    assert!(output.contains('A'));
    assert!(output.contains('B'));
    assert!(output.contains('C'));
    let cleaned = to_markup(&output).join("\n");
    assert_eq!(cleaned.matches('A').count(), 1);
}
