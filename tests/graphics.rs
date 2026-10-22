use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

const QUERY: &str = "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\";
const OK: &str = "\x1b_Gi=31;OK\x1b\\";
const DA1: &str = "\x1b[?1;2c";

fn reader(cx: &mut Scope, values: &Rc<RefCell<Vec<Graphics>>>) -> Element {
    values.borrow_mut().push(cx.use_graphics());
    Element::empty()
}

#[test]
fn probe_queries_run_once_and_replies_schedule_readers_without_input_hooks() {
    let terminal = TestTerminal::new(80, 24);
    let values = Rc::new(RefCell::new(Vec::new()));
    let mut app = mount(
        component(reader, values.clone()),
        Options::default(),
        &terminal,
    );
    assert_eq!(values.borrow()[0].support, GraphicsSupport::Probing);
    assert_eq!(terminal.raw_mode_calls(), [true]);
    let writes = terminal.writes();
    assert_eq!(&writes[writes.len() - 3..], [QUERY, "\x1b[16t", "\x1b[c"]);
    app.input("x");
    app.input("\x1b_Gi=");
    app.input("31;OK\x1b");
    app.input("\\\x1b[6;20;");
    app.input("10t\x1b[?1;");
    app.input("2c");
    assert_eq!(
        values.borrow().last(),
        Some(&Graphics {
            support: GraphicsSupport::Kitty,
            cell_pixels: Some((10, 20)),
        })
    );
    assert!(values.borrow().len() > 1);
    assert_eq!(terminal.raw_mode_calls(), [true, false]);
    app.rerender(component(reader, values.clone()));
    assert_eq!(
        terminal
            .writes()
            .iter()
            .filter(|write| *write == QUERY)
            .count(),
        1
    );
}

#[test]
fn ctrl_c_exits_during_the_probe() {
    let terminal = TestTerminal::new(80, 24);
    let values = Rc::new(RefCell::new(Vec::new()));
    let mut app = mount(
        component(reader, values.clone()),
        Options::default(),
        &terminal,
    );
    app.input("\x03");
    assert!(app.exit_result().is_some());
}

#[test]
fn probe_is_lazy_and_shared_by_components() {
    let terminal = TestTerminal::new(80, 24);
    let mut app = mount(Element::empty(), Options::default(), &terminal);
    assert!(!terminal.writes().concat().contains(QUERY));
    assert!(terminal.raw_mode_calls().is_empty());
    let first = Rc::new(RefCell::new(Vec::new()));
    let second = Rc::new(RefCell::new(Vec::new()));
    app.rerender(Element::fragment([
        component(reader, first.clone()),
        component(reader, second.clone()),
    ]));
    app.input(format!("{OK}{DA1}"));
    for values in [first, second] {
        assert_eq!(values.borrow().len(), 2);
        assert_eq!(values.borrow()[1].support, GraphicsSupport::Kitty);
    }
    assert_eq!(
        terminal
            .writes()
            .iter()
            .filter(|write| *write == QUERY)
            .count(),
        1
    );
}

#[test]
fn da1_or_timeout_finishes_with_the_evidence_seen() {
    for (reply, timeout, expected) in [
        (DA1, false, GraphicsSupport::Unsupported),
        ("", true, GraphicsSupport::Unsupported),
        (OK, true, GraphicsSupport::Kitty),
        (
            "\x1b_Gi=31;ENOENT:no image\x1b\\",
            true,
            GraphicsSupport::Unsupported,
        ),
        ("\x1b_Gi=32;OK\x1b\\", true, GraphicsSupport::Unsupported),
    ] {
        let terminal = TestTerminal::new(80, 24);
        let values = Rc::new(RefCell::new(Vec::new()));
        let mut app = mount(
            component(reader, values.clone()),
            Options::default(),
            &terminal,
        );
        app.input(reply);
        if timeout {
            app.advance(Duration::from_millis(999));
            assert_eq!(
                values.borrow().last().unwrap().support,
                GraphicsSupport::Probing
            );
            app.advance(Duration::from_millis(1));
            app.advance(Duration::ZERO);
        }
        assert_eq!(values.borrow().last().unwrap().support, expected);
        assert_eq!(terminal.raw_mode_calls(), [true, false]);
    }
}

#[test]
fn replies_are_not_keys_and_keyboard_negotiation_can_overlap() {
    let terminal = TestTerminal::new(80, 24);
    let inputs = Rc::new(RefCell::new(Vec::new()));
    let mut app = mount(
        component(
            |cx: &mut Scope, inputs: &Rc<RefCell<Vec<String>>>| {
                cx.use_graphics();
                let inputs = inputs.clone();
                cx.use_input(true, move |text, _| inputs.borrow_mut().push(text.into()));
                Element::empty()
            },
            inputs.clone(),
        ),
        Options {
            kitty_keyboard: KittyKeyboard::Auto(KittyFlags::default()),
            ..Options::default()
        },
        &terminal,
    );
    app.input(format!("a{OK}\x1b[6;20;10t{DA1}\x1b[?0ub"));
    app.input("\x1b]10;rgb:ffff/ffff/ffff\x07\x1bP1$r0m\x1b\\\x1b_X\x07");
    assert_eq!(*inputs.borrow(), ["a", "b"]);
    assert!(terminal.writes().iter().any(|write| write == "\x1b[>1u"));
    assert_eq!(terminal.raw_mode_calls(), [true]);
}

#[test]
fn disabled_apps_and_debug_renders_do_not_probe() {
    for (terminal, interactive) in [
        (TestTerminal::new(80, 24), false),
        (TestTerminal::new(80, 24).with_tty(false), true),
        (TestTerminal::new(80, 24).with_stdin_tty(false), true),
    ] {
        let values = Rc::new(RefCell::new(Vec::new()));
        let _app = mount(
            component(reader, values.clone()),
            Options {
                interactive: Some(interactive),
                ..Options::default()
            },
            &terminal,
        );
        assert_eq!(values.borrow()[0].support, GraphicsSupport::Unsupported);
        assert!(!terminal.writes().concat().contains(QUERY));
        if !interactive {
            assert!(terminal.writes().is_empty());
        }
        assert!(terminal.raw_mode_calls().is_empty());
    }
    let values = Rc::new(RefCell::new(Vec::new()));
    let _app = testing::render(component(reader, values.clone()));
    assert_eq!(values.borrow()[0].support, GraphicsSupport::Unsupported);
    render_to_string(component(reader, values.clone()), RenderOptions::default());
    assert!(
        values
            .borrow()
            .iter()
            .all(|value| value.support == GraphicsSupport::Unsupported)
    );
}

#[test]
fn terminal_handover_cancels_the_probe_without_restarting() {
    let terminal = TestTerminal::new(80, 24);
    let values = Rc::new(RefCell::new(Vec::new()));
    let mut app = mount(
        component(reader, values.clone()),
        Options::default(),
        &terminal,
    );
    app.input(OK);
    app.suspend_terminal().unwrap();
    app.resume_terminal();
    assert_eq!(
        values.borrow().last().unwrap().support,
        GraphicsSupport::Unsupported
    );
    assert_eq!(terminal.raw_mode_calls(), [true, false]);
    assert_eq!(
        terminal
            .writes()
            .iter()
            .filter(|write| *write == QUERY)
            .count(),
        1
    );
}

#[test]
fn alt_string_introducers_keep_the_escape_timeout() {
    for introducer in ["P", "_"] {
        let inputs = Rc::new(RefCell::new(Vec::new()));
        let terminal = TestTerminal::new(80, 24);
        let mut app = mount(
            component(
                |cx: &mut Scope, inputs: &Rc<RefCell<Vec<(String, bool)>>>| {
                    let inputs = inputs.clone();
                    cx.use_input(true, move |text, key| {
                        inputs.borrow_mut().push((text.into(), key.meta))
                    });
                    Element::empty()
                },
                inputs.clone(),
            ),
            Options::default(),
            &terminal,
        );
        app.input(format!("\x1b{introducer}"));
        assert!(inputs.borrow().is_empty());
        app.advance(Duration::from_millis(20));
        app.input("x");
        assert_eq!(
            *inputs.borrow(),
            [(introducer.into(), true), ("x".into(), false)]
        );
    }
}
