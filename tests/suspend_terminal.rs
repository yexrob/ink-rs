//! Ported from Ink test/suspend-terminal.tsx (Ink 8.0.0, commit 26d2c3f).
//! Not ported: Symbol.asyncDispose (duplicates resume), incremental Static
//! (duplicates standard), screen reader (deferred), debug (no debug mode), and
//! handover rollback (terminal calls cannot throw; I/O errors end the app).
//! The child-process case lives in tests/pty.rs.

use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

type Handover = Box<dyn FnOnce(AppHandle) + Send>;
struct Callback(Arc<Mutex<Option<Handover>>>);
impl PartialEq for Callback {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
struct Writer(Rc<Cell<Option<OutputWriter>>>);
impl PartialEq for Writer {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn callback_app(cx: &mut Scope, callback: &Callback) -> Element {
    cx.use_input(true, |_, _| {});
    let handle = cx.use_app();
    let callback = callback.0.clone();
    cx.use_effect((), move || {
        let nested = handle.clone();
        handle
            .suspend_terminal(move || callback.lock().unwrap().take().unwrap()(nested))
            .unwrap();
    });
    Text::new("hello").into()
}

fn callback(f: impl FnOnce(AppHandle) + Send + 'static) -> Callback {
    Callback(Arc::new(Mutex::new(Some(Box::new(f)))))
}

fn input_app(cx: &mut Scope, _: &()) -> Element {
    cx.use_input(true, |_, _| {});
    Text::new("hello").into()
}

#[test]
fn hands_the_terminal_to_the_callback_then_restores_ink() {
    let terminal = TestTerminal::new(100, 24);
    let ran = Arc::new(Mutex::new(false));
    let flag = ran.clone();
    let _app = mount(
        component(
            callback_app,
            callback(move |_| *flag.lock().unwrap() = true),
        ),
        Options::default(),
        &terminal,
    );
    assert!(*ran.lock().unwrap());
    // The step runs teardown, callback, resume in order; false precedes f.
    // f cannot capture the Rc-backed terminal because callbacks are Send.
    assert_eq!(terminal.raw_mode_calls(), [true, false, true]);
    let output = terminal.writes().concat();
    let shown = output.find("\x1b[?25h").unwrap();
    assert!(output[shown..].contains("\x1b[?25l"));
}

#[test]
fn restores_the_terminal_even_if_the_callback_throws() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(input_app, ()), Options::default(), &terminal);
    let result = catch_unwind(AssertUnwindSafe(|| {
        app.rerender(component(callback_app, callback(|_| panic!("boom"))));
    }));
    assert_eq!(*result.unwrap_err().downcast::<&str>().unwrap(), "boom");
    // Rust has no rejected promise to catch: the panic leaves the driver
    // with the terminal handed over.
    assert_eq!(terminal.raw_mode_calls().last(), Some(&false));
    assert!(terminal.writes().concat().contains("\x1b[?25h"));
}

#[test]
fn returns_a_disposable_that_resumes_on_resume() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(input_app, ()), Options::default(), &terminal);
    app.suspend_terminal().unwrap();
    assert_eq!(terminal.raw_mode_calls().last(), Some(&false));
    assert_eq!(app.suspend_terminal(), Err(AlreadySuspended));
    app.resume_terminal();
    assert_eq!(terminal.raw_mode_calls().last(), Some(&true));
}

#[test]
fn keeps_ink_off_the_terminal_while_suspended() {
    fn example(cx: &mut Scope, writer: &Writer) -> Element {
        writer.0.set(Some(cx.use_stdout()));
        cx.use_input(true, |_, _| {});
        Text::new("hello").into()
    }
    let terminal = TestTerminal::new(100, 24);
    let writer = Rc::new(Cell::new(None));
    let mut app = mount(
        component(example, Writer(writer.clone())),
        Options::default(),
        &terminal,
    );
    app.suspend_terminal().unwrap();
    let count = terminal.writes().len();
    writer.get().unwrap().write("output while suspended");
    app.rerender(Text::new("after"));
    app.clear();
    app.advance(Duration::from_millis(100));
    assert_eq!(terminal.writes().len(), count);
    app.resume_terminal();
    assert!(terminal.writes()[count..].concat().contains("after"));
}

#[test]
fn runs_the_callback_but_skips_the_handoff_when_not_interactive() {
    let terminal = TestTerminal::new(100, 24);
    let ran = Arc::new(Mutex::new(false));
    let flag = ran.clone();
    let _app = mount(
        component(
            callback_app,
            callback(move |_| *flag.lock().unwrap() = true),
        ),
        Options {
            interactive: Some(false),
            ..Options::default()
        },
        &terminal,
    );
    assert!(*ran.lock().unwrap());
    assert!(!terminal.writes().concat().contains("\x1b[?25h"));
    assert_eq!(terminal.raw_mode_calls(), [true, false, true]);
}

#[test]
fn rejects_a_nested_suspend_while_already_suspended() {
    let terminal = TestTerminal::new(100, 24);
    let result = Arc::new(Mutex::new(None));
    let recorded = result.clone();
    let _app = mount(
        component(
            callback_app,
            callback(move |handle| {
                *recorded.lock().unwrap() = Some(handle.suspend_terminal(|| {}));
            }),
        ),
        Options::default(),
        &terminal,
    );
    assert_eq!(*result.lock().unwrap(), Some(Err(AlreadySuspended)));
}

#[test]
fn exits_and_re_enters_the_alternate_screen() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(input_app, ()),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    );
    app.suspend_terminal().unwrap();
    assert!(terminal.writes().concat().contains("\x1b[?1049l"));
    let count = terminal.writes().len();
    app.resume_terminal();
    assert!(terminal.writes()[count..].concat().contains("\x1b[?1049h"));
}

#[test]
fn shows_static_output_once_after_resume_re_enters_the_alternate_screen_standard() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        cx.use_input(true, |_, _| {});
        let handle = cx.use_app();
        cx.use_effect((), move || handle.suspend_terminal(|| {}).unwrap());
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["S1", "S2", "S3"], |item, _| Text::new(*item).into()).into(),
            Text::new("live").into(),
        ])
    }
    let terminal = TestTerminal::new(100, 5);
    let _app = mount(
        component(example, ()),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    );
    let output = terminal.writes().concat();
    let resumed = output.rfind("\x1b[?1049h").unwrap();
    let mut screen = vt100::Parser::new(5, 100, 0);
    screen.process(output[resumed..].replace('\n', "\r\n").as_bytes());
    assert_eq!(
        screen.screen().rows(0, 100).collect::<Vec<_>>(),
        ["S1", "S2", "S3", "live", ""]
    );
}
