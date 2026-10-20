//! Ported from ink test/hooks.tsx @ 26d2c3f.
//! The use-stdout/use-stderr fixtures run in process; both streams feed vt100.
mod common;

use common::Screen;
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

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
