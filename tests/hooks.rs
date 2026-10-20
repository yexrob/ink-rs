//! Ported from ink test/hooks.tsx @ 26d2c3f.
//! The use-stdout/use-stderr fixtures run in process; both streams feed vt100.
mod common;

use common::Screen;
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

#[test]
fn use_input_ignore_input_if_not_active() {
    thread_local! { static HANDLE: std::cell::RefCell<Option<AppHandle>> = const { std::cell::RefCell::new(None) }; }
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, _: &()| {
                HANDLE.with(|handle| *handle.borrow_mut() = Some(cx.use_app()));
                let text = cx.use_state(String::new);
                cx.use_input(true, move |input, _| {
                    text.update(|text| text.push_str(input))
                });
                cx.use_input(false, move |input, _| {
                    text.update(|text| text.push_str(input))
                });
                Text::new(text.get()).into()
            },
            (),
        ),
        Options::default(),
        &terminal,
    );
    app.input("x");
    // No timers yet: the retained handle replaces the fixture's 100 ms exit.
    HANDLE.with(|handle| handle.borrow_mut().take().unwrap().exit());
    app.advance(std::time::Duration::ZERO);
    assert!(matches!(app.exit_result(), Some(Ok(None))));
    assert_eq!(Screen::new(&terminal, 100, 24).rows()[0], "x");
}

#[test]
fn use_input_handle_ctrl_c_via_kitty_protocol_when_exit_on_ctrl_c_is_false() {
    fn input(cx: &mut Scope, _: &()) -> Element {
        let exit = cx.use_app();
        cx.use_input(true, move |input, key| {
            if input == "c" && key.ctrl {
                exit.exit();
            } else {
                panic!("Crash");
            }
        });
        Element::fragment(Vec::<Element>::new())
    }
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(input, ()),
        Options {
            exit_on_ctrl_c: false,
            ..Options::default()
        },
        &terminal,
    );
    app.input("\x1b[99;5u");
    assert!(matches!(app.exit_result(), Some(Ok(None))));
}

fn write(cx: &mut Scope, stream: &OutputStream) -> Element {
    let writer = match stream {
        OutputStream::Stdout => cx.use_stdout(),
        OutputStream::Stderr => cx.use_stderr(),
    };
    let stream = *stream;
    cx.use_effect((), move || {
        writer.write(match stream {
            OutputStream::Stdout => "Hello from Ink to stdout\n",
            OutputStream::Stderr => "Hello from Ink to stderr\n",
        });
    });
    Text::new("Hello World").into()
}

fn fixture(stream: OutputStream) -> Vec<String> {
    let terminal = TestTerminal::new(100, 24);
    let app = mount(component(write, stream), Options::default(), &terminal);
    app.unmount();
    terminal.write("exited\r\n");
    Screen::new(&terminal, 100, 24)
        .lines()
        .into_iter()
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn use_stdout_write_to_stdout() {
    assert_eq!(
        fixture(OutputStream::Stdout),
        ["Hello from Ink to stdout", "Hello World", "exited"]
    );
}

#[test]
fn use_stderr_write_to_stderr() {
    assert_eq!(
        fixture(OutputStream::Stderr),
        ["Hello from Ink to stderr", "Hello World", "exited"]
    );
}

#[test]
fn external_write_newline_before_reset_has_no_blank_row() {
    for stream in [OutputStream::Stdout, OutputStream::Stderr] {
        let terminal = TestTerminal::new(100, 24);
        let _app = mount(
            component(
                |cx: &mut Scope, stream: &OutputStream| {
                    let writer = match stream {
                        OutputStream::Stdout => cx.use_stdout(),
                        OutputStream::Stderr => cx.use_stderr(),
                    };
                    cx.use_effect((), move || writer.write("\x1b[31mabc\n\x1b[0m"));
                    Text::new("Hello World").into()
                },
                stream,
            ),
            Options {
                interactive: Some(true),
                ..Options::default()
            },
            &terminal,
        );
        assert_eq!(
            &Screen::new(&terminal, 100, 24).rows()[..2],
            ["abc", "Hello World"],
            "{stream:?}"
        );
    }
}
