// Ink test/components.tsx and test/fixtures/ci.tsx, commit 26d2c3f.
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use super::{Stop, environment, run_loop};
use crate::app::{App, resolve_interactive};
use crate::testing::TestTerminal;
use crate::{ColorLevel, Element, Options, Scope, Static, Text, component};

fn counter(cx: &mut Scope, _: &()) -> Element {
    let count = cx.use_state(|| 0);
    let handle = cx.use_app();
    cx.use_effect((), move || {
        thread::spawn(move || {
            for n in 1..=5 {
                thread::sleep(Duration::from_millis(20));
                handle.update(move || count.set(n));
            }
        });
    });
    let n = count.get();
    Element::fragment::<[Element; 2]>([
        Static::new((1..=n).collect::<Vec<_>>(), |n, _| {
            Text::new(format!("#{n}")).into()
        })
        .into(),
        Text::new(format!("Counter: {n}")).into(),
    ])
}

fn run(ci: &str, tty: bool) -> String {
    let terminal = TestTerminal::new(80, 24).with_tty(tty).without_size();
    let options = Options::default();
    let in_ci = environment::in_ci(|key| (key == "CI").then(|| ci.to_owned()));
    let mut app = App::new(
        component(counter, ()),
        options,
        Box::new(terminal.clone()),
        resolve_interactive(options.interactive, in_ci, tty),
        ColorLevel::None,
    );
    let (sender, receiver) = mpsc::channel();
    app.queue().set_wake(Arc::new(sender));
    app.render_if_due(Instant::now());
    assert!(matches!(run_loop(&mut app, &receiver), Stop::Idle));
    app.unmount();
    terminal.writes().join("")
}

#[test]
fn render_only_last_frame_when_run_in_ci() {
    let output = run("true", false);
    for count in 0..5 {
        assert!(!output.contains(&format!("Counter: {count}")));
    }
    assert!(output.contains("Counter: 5"));
    assert_eq!(output, "#1\n#2\n#3\n#4\n#5\nCounter: 5\n");
}

#[test]
fn render_all_frames_if_ci_environment_variable_equals_false() {
    let output = run("false", true);
    for count in 0..=5 {
        assert!(output.contains(&format!("Counter: {count}")));
    }
    for n in 1..=5 {
        assert_eq!(output.matches(&format!("#{n}")).count(), 1);
    }
}

// Native coverage for the loop's exit check; PTY exit tests arrive in M3.
#[test]
fn run_loop_exits_while_an_outside_handle_is_retained() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = App::new(
        Text::new("Hello").into(),
        Options::default(),
        Box::new(terminal),
        true,
        ColorLevel::None,
    );
    let (sender, receiver) = mpsc::channel();
    app.queue().set_wake(Arc::new(sender));
    app.render_if_due(Instant::now());
    let handle = app.queue().handle();
    let worker_handle = handle.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        worker_handle.exit_with("finished");
    });
    assert!(matches!(run_loop(&mut app, &receiver), Stop::Exit));
    let value = app.take_exit_result().unwrap().unwrap().unwrap();
    assert_eq!(*value.downcast::<&str>().unwrap(), "finished");
    worker.join().unwrap();
    drop(handle);
}
