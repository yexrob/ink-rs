use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use crate::testing::{TestTerminal, mount};
use crate::{Element, Error, Options, Scope, Static, Text, Transform, View, component};

struct FailOnDrop(&'static str);

impl Drop for FailOnDrop {
    fn drop(&mut self) {
        panic!("{}", self.0);
    }
}

#[test]
fn failed_render_retires_refs_attached_before_the_panic() {
    fn broken(_: &mut Scope, _: &()) -> Element {
        panic!("sibling failure");
    }
    fn parent(cx: &mut Scope, _: &()) -> Element {
        let node_ref = cx.use_node_ref();
        View::new()
            .child(
                View::new()
                    .node_ref(node_ref)
                    .child(Text::new("before panic")),
            )
            .child(component(broken, ()))
            .into()
    }
    let terminal = TestTerminal::new(100, 1000);
    let app = mount(component(parent, ()), Options::default(), &terminal);
    let Some(Err(Error::Panic(error))) = app.exit_result() else {
        panic!("expected panic");
    };
    assert_eq!(error.message(), "sibling failure");
    assert!(terminal.writes().join("").contains("ERROR"));
}

#[test]
fn panic_replaces_successful_exit_but_preserves_an_earlier_error() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<Error>();
    fn broken(cx: &mut Scope, failure: &bool) -> Element {
        let handle = cx.use_app();
        if *failure {
            handle.exit_with_error("earlier failure");
        } else {
            handle.exit_with(7);
        }
        panic!("component failure");
    }
    for failure in [false, true] {
        let terminal = TestTerminal::new(100, 1000);
        let app = mount(component(broken, failure), Options::default(), &terminal);
        match app.exit_result() {
            Some(Err(Error::App(error))) if failure => {
                assert_eq!(error.to_string(), "earlier failure")
            }
            Some(Err(Error::Panic(error))) if !failure => {
                assert_eq!(error.message(), "component failure")
            }
            result => panic!("wrong precedence: {result:?}"),
        }
        assert!(terminal.writes().join("").contains("ERROR"));
    }
}

#[test]
fn layout_failure_retires_the_tree_and_preserves_the_first_panic() {
    fn broken(cx: &mut Scope, _: &()) -> Element {
        cx.use_layout_effect((), || FailOnDrop("cleanup failure"));
        cx.use_layout_effect((), || FailOnDrop("another cleanup failure"));
        cx.use_layout_effect((), || panic!("layout failure"));
        Text::new("user tree").into()
    }
    let terminal = TestTerminal::new(100, 1000);
    let app = mount(component(broken, ()), Options::default(), &terminal);
    let Some(Err(Error::Panic(error))) = app.exit_result() else {
        panic!("expected panic");
    };
    assert_eq!(error.message(), "layout failure");
    assert!(terminal.writes().join("").contains("ERROR"));
}

#[test]
fn cleanup_failure_during_passive_replacement_keeps_the_overview() {
    fn broken(cx: &mut Scope, _: &()) -> Element {
        cx.use_layout_effect((), || FailOnDrop("layout cleanup failure"));
        cx.use_effect((), || panic!("passive failure"));
        Text::new("user tree").into()
    }
    let terminal = TestTerminal::new(100, 1000);
    let app = mount(component(broken, ()), Options::default(), &terminal);
    let Some(Err(Error::Panic(error))) = app.exit_result() else {
        panic!("expected panic");
    };
    assert_eq!(error.message(), "passive failure");
    assert!(terminal.writes().join("").contains("ERROR"));
}

#[test]
fn static_retirement_cleanup_panics_reach_the_overview() {
    fn item(cx: &mut Scope, _: &()) -> Element {
        cx.use_effect((), || FailOnDrop("static cleanup"));
        Text::new("item").into()
    }
    let terminal = TestTerminal::new(100, 1000);
    let app = mount(
        Static::new(vec![()], |_, _| component(item, ())),
        Options::default(),
        &terminal,
    );
    let Some(Err(Error::Panic(error))) = app.exit_result() else {
        panic!("expected panic");
    };
    assert_eq!(error.message(), "static cleanup");
    assert!(terminal.writes().join("").contains("ERROR"));
}

#[test]
fn non_interactive_exit_writes_the_overview_as_the_final_frame() {
    let terminal = TestTerminal::new(100, 24).with_tty(false);
    let app = mount(
        component(|_, _: &()| panic!("final overview"), ()),
        Options::default(),
        &terminal,
    );
    assert!(matches!(app.exit_result(), Some(Err(Error::Panic(_)))));
    let output = terminal.writes().join("");
    assert!(output.contains("ERROR"));
    assert!(output.contains("final overview"));
    assert!(output.ends_with('\n'));
}

#[test]
fn alternate_screen_discards_the_overview_but_retains_the_error() {
    let terminal = TestTerminal::new(100, 24);
    terminal.write("primary screen");
    let app = mount(
        component(|_, _: &()| panic!("alternate overview"), ()),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    );
    let Some(Err(Error::Panic(error))) = app.exit_result() else {
        panic!("expected panic");
    };
    assert_eq!(error.message(), "alternate overview");
    assert!(error.location().is_some());
    let mut screen = vt100::Parser::new(24, 100, 0);
    for write in terminal.writes() {
        screen.process(write.replace('\n', "\r\n").as_bytes());
    }
    assert!(!screen.screen().alternate_screen());
    assert_eq!(screen.screen().contents(), "primary screen");
}

#[test]
fn transforms_input_updates_and_final_cleanup_remain_uncaught() {
    fn input(cx: &mut Scope, _: &()) -> Element {
        cx.use_input(true, |_, _| panic!("input failure"));
        Text::new("input").into()
    }
    fn update(cx: &mut Scope, _: &()) -> Element {
        let handle = cx.use_app();
        cx.use_effect((), move || handle.update(|| panic!("update failure")));
        Text::new("update").into()
    }
    fn cleanup(cx: &mut Scope, _: &()) -> Element {
        cx.use_effect((), || FailOnDrop("teardown failure"));
        Text::new("cleanup").into()
    }
    let terminal = TestTerminal::new(100, 24);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            mount(
                Transform::new(|_, _| panic!("transform failure")).child(Text::new("transform")),
                Options::default(),
                &terminal,
            );
        }))
        .is_err()
    );
    let mut app = mount(component(input, ()), Options::default(), &terminal);
    assert!(catch_unwind(AssertUnwindSafe(|| app.input("x"))).is_err());
    drop(app);
    let mut app = mount(component(update, ()), Options::default(), &terminal);
    assert!(catch_unwind(AssertUnwindSafe(|| app.advance(Duration::ZERO))).is_err());
    drop(app);
    let app = mount(component(cleanup, ()), Options::default(), &terminal);
    assert!(catch_unwind(AssertUnwindSafe(|| app.unmount())).is_err());
}

#[test]
#[cfg(debug_assertions)]
fn state_output_and_focus_assertions_report_the_callers_location() {
    fn state(cx: &mut Scope, _: &()) -> Element {
        let state = cx.use_state(|| 0);
        state.set(1);
        Text::new("state").into()
    }
    fn output(cx: &mut Scope, _: &()) -> Element {
        cx.use_stdout().write("forbidden");
        Text::new("output").into()
    }
    fn focus(cx: &mut Scope, _: &()) -> Element {
        cx.use_focus_manager().focus_next();
        Text::new("focus").into()
    }
    for element in [
        component(state, ()),
        component(output, ()),
        component(focus, ()),
    ] {
        let terminal = TestTerminal::new(100, 1000);
        let app = mount(element, Options::default(), &terminal);
        let Some(Err(Error::Panic(error))) = app.exit_result() else {
            panic!("expected render assertion");
        };
        assert_eq!(error.location().unwrap().file, file!());
    }
}

#[test]
fn ctrl_z_stops_the_process_but_a_kitty_release_reaches_input() {
    use crate::{KeyEvent, KeyEventType, KittyFlags, KittyKeyboard};
    use std::{cell::RefCell, rc::Rc};
    for kitty_keyboard in [
        KittyKeyboard::Disabled,
        KittyKeyboard::Enabled(
            KittyFlags::DISAMBIGUATE_ESCAPE_CODES | KittyFlags::REPORT_EVENT_TYPES,
        ),
    ] {
        let inputs = Rc::new(RefCell::new(Vec::new()));
        let terminal = TestTerminal::new(100, 24);
        let mut app = mount(
            component(
                |cx: &mut Scope, inputs: &Rc<RefCell<Vec<(String, KeyEvent)>>>| {
                    let inputs = inputs.clone();
                    cx.use_input(true, move |input, key| {
                        inputs.borrow_mut().push((input.into(), key))
                    });
                    Text::new("input").into()
                },
                inputs.clone(),
            ),
            Options {
                kitty_keyboard,
                ..Options::default()
            },
            &terminal,
        );
        app.input("\x1a");
        assert_eq!(terminal.stop_process_calls(), 1);
        assert!(inputs.borrow().is_empty());
        app.input("\x1b[122;5:3u");
        assert_eq!(terminal.stop_process_calls(), 1);
        let inputs = inputs.borrow();
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].0, "z");
        assert!(inputs[0].1.ctrl);
        assert_eq!(inputs[0].1.event_type, Some(KeyEventType::Release));
    }
}

#[test]
fn disabling_suspend_on_ctrl_z_passes_the_key_to_input() {
    use crate::KeyEvent;
    use std::{cell::RefCell, rc::Rc};
    let inputs = Rc::new(RefCell::new(Vec::new()));
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, inputs: &Rc<RefCell<Vec<(String, KeyEvent)>>>| {
                let inputs = inputs.clone();
                cx.use_input(true, move |input, key| {
                    inputs.borrow_mut().push((input.into(), key))
                });
                Text::new("input").into()
            },
            inputs.clone(),
        ),
        Options {
            suspend_on_ctrl_z: false,
            ..Options::default()
        },
        &terminal,
    );
    app.input("\x1a");
    assert_eq!(terminal.stop_process_calls(), 0);
    let inputs = inputs.borrow();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].0, "z");
    assert!(inputs[0].1.ctrl);
}
