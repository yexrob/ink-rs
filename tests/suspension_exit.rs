//! Ported from Ink test/suspension-exit.tsx (Ink 8.0.0, commit 26d2c3f).
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn example(cx: &mut Scope, callback: &bool) -> Element {
    cx.use_input(true, |_, _| {});
    let handle = cx.use_app();
    let callback = *callback;
    cx.use_effect(callback, move || {
        if callback {
            let exit = handle.clone();
            handle.suspend_terminal(move || exit.exit()).unwrap();
        }
    });
    Text::new("hello").into()
}

#[test]
fn resuming_after_unmount_does_not_re_enable_terminal_input_callback_false() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(example, false), Options::default(), &terminal);
    app.suspend_terminal().unwrap();
    let count = terminal.writes().len();
    app.unmount();
    // unmount(self) consumes TestApp, so a subsequent resume cannot be written.
    assert_eq!(terminal.raw_mode_calls().last(), Some(&false));
    assert_eq!(terminal.writes().len(), count);
}

#[test]
fn resuming_after_unmount_does_not_re_enable_terminal_input_callback_true() {
    let terminal = TestTerminal::new(100, 24);
    let app = mount(component(example, true), Options::default(), &terminal);
    assert_eq!(terminal.raw_mode_calls().last(), Some(&false));
    assert!(app.exit_result().is_some());
}
