//! A mounted tree with frame history, driven by calls to [`TestInstance::settle`].

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::app::{App, resolve_interactive};
use crate::paint::{Frame, serialize_row};
use crate::runtime::{Runtime, WriteDestination};
use crate::terminal::Terminal;
use crate::{ColorLevel, Element, Options, OutputStream, RenderOptions};

/// Mounts at 100 columns with basic colors, matching Ink's debug test helper.
pub fn render(element: impl Into<Element>) -> TestInstance {
    render_with(
        element,
        RenderOptions {
            columns: 100,
            color_level: ColorLevel::Basic,
        },
    )
}

/// Mounts with the given options and presents one frame.
/// Passive-effect state updates remain pending until [`TestInstance::settle`].
pub fn render_with(element: impl Into<Element>, options: RenderOptions) -> TestInstance {
    let destination = Rc::new(RefCell::new(DebugOutput::default()));
    let mut runtime = Runtime::new(element.into(), options);
    runtime.set_destination(destination.clone());
    let mut instance = TestInstance {
        runtime,
        destination,
        frames: Vec::new(),
        stderr: Vec::new(),
    };
    instance.step();
    instance
}

/// A tree that retains state across frames. Dropping it runs unmount cleanups.
pub struct TestInstance {
    runtime: Runtime,
    destination: Rc<RefCell<DebugOutput>>,
    frames: Vec<String>,
    stderr: Vec<String>,
}

// Entries wait here until the step that produced them returns; a write made
// outside a step appears at the next one.
#[derive(Default)]
struct DebugOutput {
    frames: Vec<String>,
    stderr: Vec<String>,
    static_output: String,
    live: String,
}

impl DebugOutput {
    fn paint(&mut self, frame: Frame) {
        // Ink handleStaticChange resets history before onRender appends it.
        if frame.static_changed {
            self.static_output.clear();
        }
        for row in &frame.static_lines {
            self.static_output.push_str(&serialize_row(row));
            // renderer.ts preserves the newline even for a blank Static row.
            self.static_output.push('\n');
        }
        self.live = frame
            .live
            .iter()
            .map(|row| serialize_row(row))
            .collect::<Vec<_>>()
            .join("\n");
        self.frames
            .push(format!("{}{}", self.static_output, self.live));
    }
}

impl WriteDestination for RefCell<DebugOutput> {
    fn write(&self, stream: OutputStream, data: &str) {
        let mut output = self.borrow_mut();
        // Ink writeToStdout/writeToStderr debug branches replay the last frame.
        let prefix = match stream {
            OutputStream::Stdout => data,
            OutputStream::Stderr => {
                output.stderr.push(data.to_owned());
                ""
            }
        };
        let frame = format!("{prefix}{}{}", output.static_output, output.live);
        output.frames.push(frame);
    }
}

impl TestInstance {
    /// The last debug stdout entry, from a present or an external write.
    /// Entries contain accumulated Static rows and the last live output;
    /// stdout writes prefix their data, while stderr writes replay the frame.
    /// Static history resets when the first Static instance changes.
    pub fn last_frame(&self) -> &str {
        self.frames().last().map_or("", String::as_str)
    }

    /// Debug stdout entries in write order, including unchanged presents and
    /// external-write replays. Each contains accumulated Static rows before
    /// the last live output; stdout writes also prefix their data. A write
    /// made outside a step, through a retained writer, appears after the next
    /// step.
    pub fn frames(&self) -> &[String] {
        &self.frames
    }

    /// External stderr writes in write order, without frame replay or newline
    /// conversion.
    pub fn stderr(&self) -> &[String] {
        &self.stderr
    }

    /// Reconciles a new root against the mounted tree and runs one pass.
    pub fn rerender(&mut self, element: impl Into<Element>) {
        self.runtime.rerender(element.into());
        self.step();
    }

    /// Runs pending state updates and effects until the runtime is idle.
    pub fn settle(&mut self) {
        while self.step().is_some() {}
    }

    /// Unmounts the tree and runs effect cleanups.
    pub fn unmount(self) {
        drop(self);
    }

    fn step(&mut self) -> Option<Instant> {
        let due = self.runtime.render_if_due(Instant::now(), |frame| {
            self.destination.borrow_mut().paint(frame);
        });
        let mut output = self.destination.borrow_mut();
        self.frames.append(&mut output.frames);
        self.stderr.append(&mut output.stderr);
        due
    }
}

/// In-memory terminal. Clones share an ordered log of stdout and stderr writes.
#[derive(Clone)]
pub struct TestTerminal {
    writes: Rc<RefCell<Vec<(OutputStream, String)>>>,
    size: Option<(u16, u16)>,
    tty: bool,
}

impl TestTerminal {
    /// Creates a TTY with the given dimensions.
    pub fn new(columns: u16, rows: u16) -> Self {
        Self {
            writes: Rc::default(),
            size: Some((columns, rows)),
            tty: true,
        }
    }

    /// Selects whether stdout is a TTY.
    pub fn with_tty(mut self, tty: bool) -> Self {
        self.tty = tty;
        self
    }

    /// Makes the terminal size unknown, exercising the 80 by 24 fallback.
    pub fn without_size(mut self) -> Self {
        self.size = None;
        self
    }

    /// Returns one string per stdout write, in write order.
    pub fn writes(&self) -> Vec<String> {
        self.stream_writes(OutputStream::Stdout)
    }

    /// Returns one string per stderr write, in write order.
    pub fn stderr(&self) -> Vec<String> {
        self.stream_writes(OutputStream::Stderr)
    }

    /// Returns both streams in write order, as on a shared PTY.
    pub fn output(&self) -> Vec<(OutputStream, String)> {
        self.writes.borrow().clone()
    }

    fn stream_writes(&self, stream: OutputStream) -> Vec<String> {
        self.writes
            .borrow()
            .iter()
            .filter(|(target, _)| *target == stream)
            .map(|(_, data)| data.clone())
            .collect()
    }

    /// Writes stdout bytes, such as shell history before mount or a prompt
    /// after unmount.
    pub fn write(&self, text: &str) {
        self.writes
            .borrow_mut()
            .push((OutputStream::Stdout, text.to_owned()));
    }
}

impl Terminal for TestTerminal {
    fn write_stderr(&mut self, bytes: &[u8]) {
        self.writes.borrow_mut().push((
            OutputStream::Stderr,
            std::str::from_utf8(bytes)
                .expect("terminal output is UTF-8")
                .to_owned(),
        ));
    }

    fn write(&mut self, bytes: &[u8]) {
        TestTerminal::write(
            self,
            std::str::from_utf8(bytes).expect("terminal output is UTF-8"),
        );
    }

    fn size(&self) -> Option<(u16, u16)> {
        self.size
    }

    fn is_tty(&self) -> bool {
        self.tty
    }
}

/// Mounts and runs the first step. CI is false and the default color level is
/// Basic, matching Ink's test helpers rather than the process environment.
pub fn mount(element: impl Into<Element>, options: Options, terminal: &TestTerminal) -> TestApp {
    let interactive = resolve_interactive(options.interactive, false, terminal.is_tty());
    let mut app = TestApp {
        app: App::new(
            element.into(),
            options,
            Box::new(terminal.clone()),
            interactive,
            options.color_level.unwrap_or(ColorLevel::Basic),
        ),
        now: Instant::now(),
    };
    app.app.render_if_due(app.now);
    app
}

/// A live app driven by a fake clock. Drop unmounts and restores the terminal.
pub struct TestApp {
    app: App,
    now: Instant,
}

impl TestApp {
    /// Reconciles the root and runs a step at the current clock time.
    pub fn rerender(&mut self, element: impl Into<Element>) {
        self.app.rerender(element.into());
        self.app.render_if_due(self.now);
    }

    /// Runs a step now, then every due step through the target time.
    pub fn advance(&mut self, duration: Duration) {
        let target = self.now + duration;
        while let Some(due) = self.app.render_if_due(self.now) {
            if due > target {
                break;
            }
            self.now = due;
        }
        self.now = target;
    }

    /// Erases the frame without discarding pending work.
    pub fn clear(&mut self) {
        self.app.clear();
    }

    /// Finishes pending work, runs cleanups, and restores the terminal.
    pub fn unmount(mut self) {
        self.app.unmount();
    }
}
