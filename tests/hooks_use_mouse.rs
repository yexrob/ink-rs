use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn options() -> Options {
    Options {
        alternate_screen: true,
        interactive: Some(true),
        ..Options::default()
    }
}

#[derive(PartialEq)]
struct HitProps {
    events: Rc<RefCell<Vec<(MouseEvent, bool)>>>,
    history: usize,
}

fn hit_box(cx: &mut Scope, props: &HitProps) -> Element {
    let node = cx.use_node_ref();
    let events = props.events.clone();
    cx.use_mouse(Some(MouseTracking::Click), move |event| {
        events
            .borrow_mut()
            .push((event, measure_element(node).contains(event.x, event.y)));
    });
    cx.use_input(true, |text, _| panic!("mouse reached use_input: {text:?}"));
    Element::fragment(vec![
        Element::from(Static::new(
            (0..props.history).collect::<Vec<_>>(),
            |item, _| Text::new(item.to_string()).into(),
        )),
        View::new()
            .height(4)
            .padding_left(2)
            .padding_top(1)
            .child(View::new().node_ref(node).width(6).height(2))
            .into(),
    ])
}

#[test]
fn clicks_use_live_frame_rect_with_static_replay_and_top_cut() {
    for (history, rows, origin) in [(0, 10, 0), (2, 10, 2), (8, 6, 2), (8, 3, -1)] {
        let terminal = TestTerminal::new(20, rows);
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut app = mount(
            component(
                hit_box,
                HitProps {
                    events: events.clone(),
                    history,
                },
            ),
            options(),
            &terminal,
        );
        app.input(format!("\x1b[<0;3;{}M", origin + 2));
        app.input(format!("\x1b[<0;9;{}M", origin + 2));
        let received = events.borrow();
        assert_eq!(received.len(), 2);
        assert_eq!(received[0].0.kind, MouseKind::Press(MouseButton::Left));
        assert_eq!(
            (received[0].0.x, received[0].0.y, received[0].1),
            (2, 1, true)
        );
        assert_eq!(
            (received[1].0.x, received[1].0.y, received[1].1),
            (8, 1, false)
        );
        drop(received);
        if origin > 0 {
            app.input("\x1b[<0;3;1M");
            assert_eq!(events.borrow()[2].0.y, -origin);
            assert!(!events.borrow()[2].1);
        }
    }
}

fn tracking_hooks(cx: &mut Scope, tracking: &Option<MouseTracking>) -> Element {
    cx.use_mouse(Some(MouseTracking::Click), |_| {});
    cx.use_mouse(*tracking, |_| {});
    Text::new("mouse").into()
}

#[test]
fn mouse_modes_follow_highest_hook_and_restore_on_suspend_and_unmount() {
    let terminal = TestTerminal::new(20, 10);
    let mut app = mount(
        component(tracking_hooks, Some(MouseTracking::Drag)),
        options(),
        &terminal,
    );
    assert_eq!(terminal.raw_mode_calls(), [true]);
    assert!(
        terminal
            .writes()
            .iter()
            .any(|write| write == "\x1b[?1006h\x1b[?1002h")
    );

    let start = terminal.writes().len();
    app.rerender(component(tracking_hooks, Some(MouseTracking::Motion)));
    assert_eq!(
        terminal.writes()[start..].concat(),
        "\x1b[?1002l\x1b[?1003h"
    );

    let start = terminal.writes().len();
    app.suspend_terminal().unwrap();
    assert!(
        terminal.writes()[start..]
            .concat()
            .starts_with("\x1b[?1003l\x1b[?1006l")
    );
    assert_eq!(terminal.raw_mode_calls(), [true, false]);
    let start = terminal.writes().len();
    app.resume_terminal();
    assert!(
        terminal.writes()[start..]
            .concat()
            .contains("\x1b[?1006h\x1b[?1003h")
    );
    assert_eq!(terminal.raw_mode_calls(), [true, false, true]);

    let start = terminal.writes().len();
    app.rerender(component(tracking_hooks, None));
    assert_eq!(
        terminal.writes()[start..].concat(),
        "\x1b[?1003l\x1b[?1000h"
    );
    let start = terminal.writes().len();
    app.unmount();
    assert!(
        terminal.writes()[start..]
            .concat()
            .contains("\x1b[?1000l\x1b[?1006l")
    );
    assert_eq!(terminal.raw_mode_calls(), [true, false, true, false]);
}

#[test]
fn unsupported_surfaces_do_not_enable_or_deliver_mouse() {
    for (alternate, stdout_tty, stdin_tty) in [
        (false, true, true),
        (true, false, true),
        (true, true, false),
    ] {
        let terminal = TestTerminal::new(20, 10)
            .with_tty(stdout_tty)
            .with_stdin_tty(stdin_tty);
        let mut app = mount(
            component(
                |cx: &mut Scope, _: &()| {
                    cx.use_mouse(Some(MouseTracking::Motion), |_| panic!("unsupported mouse"));
                    Text::new("mouse").into()
                },
                (),
            ),
            Options {
                alternate_screen: alternate,
                ..options()
            },
            &terminal,
        );
        app.input("\x1b[<0;1;1M");
        assert!(app.exit_result().is_none());
        if !stdin_tty {
            assert!(terminal.raw_mode_calls().is_empty());
        }
        app.unmount();
        let output = terminal.writes().concat();
        for mode in [1000, 1002, 1003, 1006] {
            assert!(!output.contains(&format!("\x1b[?{mode}")));
        }
    }
}

#[test]
fn mouse_escape_timeout_does_not_leak_into_use_input() {
    let terminal = TestTerminal::new(20, 10);
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut app = mount(
        component(
            |cx: &mut Scope, events: &Rc<RefCell<Vec<String>>>| {
                let events = events.clone();
                cx.use_input(true, move |text, _| events.borrow_mut().push(text.into()));
                Text::new("input").into()
            },
            events.clone(),
        ),
        options(),
        &terminal,
    );
    app.input("\x1b[<0;10;");
    app.advance(Duration::from_millis(20));
    assert!(events.borrow().is_empty());
    app.input("x");
    assert_eq!(*events.borrow(), ["x"]);
}
