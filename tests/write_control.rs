use ink_rs::testing::{TestApp, TestTerminal, mount};
use ink_rs::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

const CONTROL: &str = "\x1b]2;title\x07";

#[derive(Clone, Copy)]
struct Handles {
    stdout: OutputWriter,
    stderr: OutputWriter,
    state: State<&'static str>,
}

struct Capture(Rc<Cell<Option<Handles>>>);

impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn mount_writer(interactive: bool) -> (TestApp, TestTerminal, Handles) {
    fn example(cx: &mut Scope, capture: &Capture) -> Element {
        let state = cx.use_state(|| "before");
        capture.0.set(Some(Handles {
            stdout: cx.use_stdout(),
            stderr: cx.use_stderr(),
            state,
        }));
        Text::new(state.get()).into()
    }

    let terminal = TestTerminal::new(100, 24);
    let capture = Rc::new(Cell::new(None));
    let app = mount(
        component(example, Capture(capture.clone())),
        Options {
            interactive: Some(interactive),
            max_fps: 1,
            ..Options::default()
        },
        &terminal,
    );
    (app, terminal, capture.get().unwrap())
}

#[test]
fn control_bytes_follow_the_throttled_frame_of_an_earlier_state_change() {
    let (mut app, terminal, handles) = mount_writer(true);
    let before = terminal.writes();
    handles.state.set("after");
    handles.stdout.write_control(CONTROL);
    app.advance(Duration::from_millis(999));
    assert_eq!(terminal.writes(), before);

    app.advance(Duration::from_millis(1));
    let writes = terminal.writes();
    assert_eq!(writes.len(), before.len() + 2);
    assert!(writes[before.len()].contains("after"));
    assert_eq!(writes[before.len() + 1], CONTROL);
}

#[test]
fn control_bytes_go_out_in_the_same_step_when_no_frame_waits() {
    let (mut app, terminal, handles) = mount_writer(true);
    for (stream, writer) in [
        (OutputStream::Stdout, handles.stdout),
        (OutputStream::Stderr, handles.stderr),
    ] {
        let before = terminal.output();
        writer.write_control(CONTROL);
        assert_eq!(terminal.output(), before);
        app.advance(Duration::ZERO);
        let output = terminal.output();
        assert_eq!(&output[..before.len()], before);
        assert_eq!(&output[before.len()..], [(stream, CONTROL.to_owned())]);
    }
}

#[test]
fn control_bytes_wait_for_the_first_frame_after_a_terminal_handover() {
    let (mut app, terminal, handles) = mount_writer(true);
    app.suspend_terminal().unwrap();
    let before = terminal.writes();
    handles.stdout.write_control(CONTROL);
    app.advance(Duration::ZERO);
    assert_eq!(terminal.writes(), before);

    app.resume_terminal();
    let writes = terminal.writes();
    assert_eq!(writes.len(), before.len() + 2);
    assert!(writes[before.len()].contains("before"));
    assert_eq!(writes[before.len() + 1], CONTROL);
}

#[test]
fn non_interactive_control_bytes_go_out_at_the_end_of_the_step() {
    let (mut app, terminal, handles) = mount_writer(false);
    handles.state.set("after");
    handles.stdout.write_control(CONTROL);
    assert!(terminal.output().is_empty());
    app.advance(Duration::ZERO);
    assert_eq!(terminal.writes(), [CONTROL]);
}

#[test]
fn unmount_writes_queued_control_bytes_after_the_final_frame() {
    let (mut app, terminal, handles) = mount_writer(true);
    let before = terminal.writes();
    handles.state.set("after");
    handles.stdout.write_control(CONTROL);
    app.advance(Duration::ZERO);
    assert_eq!(terminal.writes(), before);

    app.unmount();
    let writes = terminal.writes();
    assert_eq!(writes.len(), before.len() + 3);
    assert!(writes[before.len()].contains("after"));
    assert_eq!(writes[before.len() + 1], CONTROL);
    assert_eq!(writes[before.len() + 2], "\x1b[?2026h\x1b[?25h\x1b[?2026l");
}
