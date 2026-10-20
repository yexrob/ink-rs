//! Ported from Ink test/suspension-output.tsx (Ink 8.0.0, commit 26d2c3f).
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn non_interactive_suspension_keeps_the_latest_rendered_frame() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        Text::new("before"),
        Options {
            interactive: Some(false),
            ..Options::default()
        },
        &terminal,
    );
    app.suspend_terminal().unwrap();
    app.rerender(Text::new("after"));
    app.unmount();
    assert_eq!(terminal.writes().concat(), "after\n");
}

struct Writer(Rc<Cell<Option<OutputWriter>>>);
impl PartialEq for Writer {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
fn preserves_write(stderr: bool) {
    fn example(cx: &mut Scope, props: &(bool, Writer)) -> Element {
        let stdout = cx.use_stdout();
        let stderr = cx.use_stderr();
        props.1.0.set(Some(if props.0 { stderr } else { stdout }));
        Text::new("frame").into()
    }
    let terminal = TestTerminal::new(100, 24);
    let writer = Rc::new(Cell::new(None));
    let mut app = mount(
        component(example, (stderr, Writer(writer.clone()))),
        Options {
            interactive: Some(false),
            ..Options::default()
        },
        &terminal,
    );
    app.suspend_terminal().unwrap();
    writer.get().unwrap().write("child result\n");
    let writes = if stderr {
        terminal.stderr()
    } else {
        terminal.writes()
    };
    assert!(writes.iter().any(|write| write == "child result\n"));
}

#[test]
fn non_interactive_suspension_preserves_stdout_writes() {
    preserves_write(false);
}

#[test]
fn non_interactive_suspension_preserves_stderr_writes() {
    preserves_write(true);
}
