//! Ported from Ink test/suspension-input-disable.tsx (Ink 8.0.0, commit 26d2c3f).
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(PartialEq)]
struct Input {
    active: bool,
    paste: bool,
    inputs: Rc<RefCell<Vec<String>>>,
}
fn example(cx: &mut Scope, props: &Input) -> Element {
    let inputs = props.inputs.clone();
    cx.use_input(props.active, move |input, _| {
        inputs.borrow_mut().push(input.into())
    });
    cx.use_paste(props.paste, |_| {});
    Element::fragment(Vec::<Element>::new())
}
fn input(active: bool, paste: bool) -> Element {
    component(
        example,
        Input {
            active,
            paste,
            inputs: Rc::default(),
        },
    )
}

#[test]
fn resuming_does_not_enable_raw_mode_for_a_disabled_input_hook() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(input(true, false), Options::default(), &terminal);
    app.suspend_terminal().unwrap();
    let calls = terminal.raw_mode_calls().len();
    app.rerender(input(false, false));
    app.resume_terminal();
    assert!(terminal.raw_mode_calls()[calls..].is_empty());
}

fn disabled_paste(other_input: bool) {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(input(other_input, true), Options::default(), &terminal);
    app.suspend_terminal().unwrap();
    let count = terminal.writes().len();
    app.rerender(input(other_input, false));
    app.resume_terminal();
    assert!(!terminal.writes()[count..].concat().contains("\x1b[?2004"));
    assert_eq!(terminal.raw_mode_calls().last(), Some(&other_input));
}

#[test]
fn resuming_does_not_restore_disabled_bracketed_paste_other_input_active_false() {
    disabled_paste(false);
}
#[test]
fn resuming_does_not_restore_disabled_bracketed_paste_other_input_active_true() {
    disabled_paste(true);
}

#[test]
fn resuming_enables_raw_mode_for_an_input_hook_activated_while_suspended() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(input(false, false), Options::default(), &terminal);
    app.suspend_terminal().unwrap();
    app.rerender(input(true, false));
    app.resume_terminal();
    assert_eq!(terminal.raw_mode_calls().last(), Some(&true));
}

#[test]
fn an_input_hook_activated_while_suspended_does_not_take_input_until_resume() {
    let terminal = TestTerminal::new(100, 24);
    let inputs = Rc::new(RefCell::new(Vec::new()));
    let mut app = mount(
        component(
            example,
            Input {
                active: false,
                paste: false,
                inputs: inputs.clone(),
            },
        ),
        Options::default(),
        &terminal,
    );
    app.suspend_terminal().unwrap();
    let calls = terminal.raw_mode_calls().len();
    app.rerender(component(
        example,
        Input {
            active: true,
            paste: false,
            inputs: inputs.clone(),
        },
    ));
    assert!(terminal.raw_mode_calls()[calls..].is_empty());
    app.input("x");
    assert!(inputs.borrow().is_empty());
    app.resume_terminal();
    assert_eq!(terminal.raw_mode_calls()[calls..], [true]);
    app.input("y");
    assert_eq!(*inputs.borrow(), ["y"]);
}

#[test]
fn a_paste_hook_activated_while_suspended_does_not_enable_bracketed_paste_until_resume() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(input(false, false), Options::default(), &terminal);
    app.suspend_terminal().unwrap();
    let calls = terminal.raw_mode_calls().len();
    let writes = terminal.writes().len();
    app.rerender(input(false, true));
    assert_eq!(
        terminal.writes()[writes..]
            .concat()
            .matches("\x1b[?2004h")
            .count(),
        0
    );
    assert!(terminal.raw_mode_calls()[calls..].is_empty());
    app.resume_terminal();
    assert_eq!(
        terminal.writes()[writes..]
            .concat()
            .matches("\x1b[?2004h")
            .count(),
        1
    );
    assert_eq!(terminal.raw_mode_calls()[calls..], [true]);
}

#[test]
fn suspending_in_the_same_commit_that_disables_the_last_input_hook_disables_raw_mode_before_the_callback_runs()
 {
    struct Props(bool, Arc<AtomicBool>);
    impl PartialEq for Props {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0 && Arc::ptr_eq(&self.1, &other.1)
        }
    }
    fn example(cx: &mut Scope, props: &Props) -> Element {
        cx.use_input(!props.0, |_, _| {});
        let handle = cx.use_app();
        let suspended = props.0;
        let ran = props.1.clone();
        cx.use_effect(suspended, move || {
            if suspended {
                handle
                    .suspend_terminal(move || ran.store(true, Ordering::SeqCst))
                    .unwrap();
            }
        });
        Element::fragment(Vec::<Element>::new())
    }
    let terminal = TestTerminal::new(100, 24);
    let ran = Arc::new(AtomicBool::new(false));
    let mut app = mount(
        component(example, Props(false, ran.clone())),
        Options::default(),
        &terminal,
    );
    let calls = terminal.raw_mode_calls().len();
    app.rerender(component(example, Props(true, ran.clone())));
    assert!(ran.load(Ordering::SeqCst));
    // End-of-step mode sync precedes the driver's call to f.
    assert_eq!(terminal.raw_mode_calls()[calls..], [false]);
}
