#![cfg(feature = "tokio")]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ink_rs::{Element, Error, Options, Scope, Text, component, testing};

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}

fn frame(terminal: &testing::TestTerminal) -> String {
    let mut parser = vt100::Parser::new(24, 100, 0);
    parser.process(terminal.writes().concat().as_bytes());
    parser.screen().contents()
}

#[tokio::test(flavor = "local", start_paused = true)]
async fn spawns_after_commit_and_state_writes_repaint() {
    let terminal = testing::TestTerminal::new(100, 24);
    let started = Rc::new(RefCell::new(false));
    let mut app = testing::mount(
        component(
            |cx: &mut Scope, started: &Rc<RefCell<bool>>| {
                let label = cx.use_state(|| "old");
                let started = started.clone();
                cx.use_future((), move || {
                    *started.borrow_mut() = true;
                    async move {
                        tokio::time::sleep(ms(100)).await;
                        label.set("new");
                    }
                });
                Text::new(label.get()).into()
            },
            started.clone(),
        ),
        Options::default(),
        &terminal,
    );
    assert!(!*started.borrow());
    assert_eq!(frame(&terminal), "old");
    tokio::time::sleep(ms(50)).await;
    app.advance(ms(50));
    assert!(*started.borrow());
    assert_eq!(frame(&terminal), "old");
    tokio::time::sleep(ms(100)).await;
    app.advance(ms(100));
    assert_eq!(frame(&terminal), "new");
}

#[derive(Clone, PartialEq)]
struct Work {
    revision: u8,
    events: Rc<RefCell<Vec<String>>>,
}

struct Finished(Work);

impl Drop for Finished {
    fn drop(&mut self) {
        self.0
            .events
            .borrow_mut()
            .push(format!("drop {}", self.0.revision));
    }
}

fn worker(cx: &mut Scope, work: &Work) -> Element {
    let label = cx.use_state(|| 0);
    let work = work.clone();
    cx.use_future(work.revision, move || async move {
        let guard = Finished(work);
        tokio::time::sleep(ms(100)).await;
        label.set(guard.0.revision);
        guard
            .0
            .events
            .borrow_mut()
            .push(format!("write {}", guard.0.revision));
    });
    Text::new(label.get().to_string()).into()
}

#[tokio::test(flavor = "local", start_paused = true)]
async fn aborts_previous_task_when_dependencies_change() {
    let terminal = testing::TestTerminal::new(100, 24);
    let mut work = Work {
        revision: 1,
        events: Rc::default(),
    };
    let mut app = testing::mount(
        component(worker, work.clone()),
        Options::default(),
        &terminal,
    );
    tokio::time::sleep(ms(50)).await;
    app.advance(ms(50));
    work.revision = 2;
    app.rerender(component(worker, work.clone()));
    tokio::time::sleep(ms(150)).await;
    app.advance(ms(150));
    assert_eq!(*work.events.borrow(), ["drop 1", "write 2", "drop 2"]);
    assert_eq!(frame(&terminal), "2");
}

#[tokio::test(flavor = "local", start_paused = true)]
async fn aborts_task_on_unmount() {
    let terminal = testing::TestTerminal::new(100, 24);
    let work = Work {
        revision: 1,
        events: Rc::default(),
    };
    let app = testing::mount(
        component(worker, work.clone()),
        Options::default(),
        &terminal,
    );
    tokio::time::sleep(ms(50)).await;
    app.unmount();
    tokio::time::sleep(ms(150)).await;
    assert_eq!(*work.events.borrow(), ["drop 1"]);
}

#[tokio::test(flavor = "local", start_paused = true)]
async fn task_panics_show_overview_and_exit_with_panic() {
    let terminal = testing::TestTerminal::new(100, 24);
    let mut app = testing::mount(
        component(
            |cx: &mut Scope, _: &()| {
                cx.use_future((), || async {
                    tokio::time::sleep(ms(100)).await;
                    panic!("future failed");
                });
                Text::new("waiting").into()
            },
            (),
        ),
        Options::default(),
        &terminal,
    );
    tokio::time::sleep(ms(150)).await;
    app.advance(ms(150));
    let frame = frame(&terminal);
    assert!(frame.contains(" ERROR "), "{frame}");
    assert!(frame.contains("future failed"), "{frame}");
    assert!(matches!(app.take_exit_result(), Some(Err(Error::Panic(_)))));
}
