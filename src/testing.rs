//! A mounted tree with frame history, driven by calls to [`TestInstance::settle`].

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::app::{App, Reason, resolve_interactive};
use crate::paint::{Frame, serialize_row};
use crate::runtime::{Runtime, WriteDestination};
use crate::terminal::Terminal;
use crate::{ColorLevel, Element, Error, Options, OutputStream, RenderOptions};

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
        now: Instant::now(),
    };
    instance.step();
    instance
}

/// A tree that retains state across frames. Dropping it runs unmount cleanups.
pub struct TestInstance {
    now: Instant,
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
    /// Feeds one stdin read at the current clock time, then settles the tree.
    /// Input reaches the runtime while any input, paste or focus holder exists.
    /// There is no raw-mode check; Ctrl+C latches an exit request that stops
    /// later input. A lone Esc waits 20 ms; call [`Self::advance`] to flush it.
    pub fn input(&mut self, bytes: impl AsRef<[u8]>) {
        self.runtime.input(bytes.as_ref(), self.now);
        self.settle();
    }

    /// Runs a step now, then every due step through the target time.
    /// Advancing by 20 ms after a lone Esc delivers it to focus and input hooks.
    pub fn advance(&mut self, duration: Duration) {
        let target = self.now + duration;
        while let Some(due) = self.step() {
            if due > target {
                break;
            }
            self.now = due;
        }
        self.now = target;
    }

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

    /// Runs pending state updates and effects without advancing the clock.
    pub fn settle(&mut self) {
        while self.step().is_some_and(|due| due <= self.now) {}
    }

    /// Unmounts the tree and runs effect cleanups.
    pub fn unmount(self) {
        drop(self);
    }

    fn step(&mut self) -> Option<Instant> {
        let due = self.runtime.render_if_due(self.now, |frame| {
            self.destination.borrow_mut().paint(frame);
        });
        let mut output = self.destination.borrow_mut();
        self.frames.append(&mut output.frames);
        self.stderr.append(&mut output.stderr);
        due
    }
}

/// In-memory terminal. Clones share ordered logs of stdout/stderr writes and
/// raw-mode switches. Stdin and stdout default to TTYs.
#[derive(Clone)]
pub struct TestTerminal {
    writes: Rc<RefCell<Vec<(OutputStream, String)>>>,
    size: Rc<Cell<Option<(u16, u16)>>>,
    resized: Rc<Cell<bool>>,
    tty: bool,
    stdin_tty: bool,
    raw_mode_calls: Rc<RefCell<Vec<bool>>>,
}

impl TestTerminal {
    /// Creates a TTY with the given dimensions.
    pub fn new(columns: u16, rows: u16) -> Self {
        Self {
            writes: Rc::default(),
            size: Rc::new(Cell::new(Some((columns, rows)))),
            resized: Rc::default(),
            tty: true,
            stdin_tty: true,
            raw_mode_calls: Rc::default(),
        }
    }

    /// Selects whether stdin is a TTY; the default is true.
    pub fn with_stdin_tty(mut self, tty: bool) -> Self {
        self.stdin_tty = tty;
        self
    }

    /// Returns every raw-mode switch argument in call order, shared by clones.
    pub fn raw_mode_calls(&self) -> Vec<bool> {
        self.raw_mode_calls.borrow().clone()
    }

    /// Selects whether stdout is a TTY.
    pub fn with_tty(mut self, tty: bool) -> Self {
        self.tty = tty;
        self
    }

    /// Makes the terminal size unknown, exercising the 80 by 24 fallback.
    pub fn without_size(self) -> Self {
        self.size.set(None);
        self
    }

    /// Changes the shared size and queues a resize event for the next app step.
    /// A zero dimension is unknown and falls back to 80 columns or 24 rows.
    pub fn resize(&self, columns: u16, rows: u16) {
        self.size.set(Some((columns, rows)));
        self.resized.set(true);
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
    fn stdin_is_tty(&self) -> bool {
        self.stdin_tty
    }

    fn set_raw_mode(&mut self, enabled: bool) {
        self.raw_mode_calls.borrow_mut().push(enabled);
    }

    fn write_stderr(&mut self, bytes: &[u8], _restore: &[u8]) {
        self.writes.borrow_mut().push((
            OutputStream::Stderr,
            std::str::from_utf8(bytes)
                .expect("terminal output is UTF-8")
                .to_owned(),
        ));
    }

    fn write(&mut self, bytes: &[u8], _restore: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        TestTerminal::write(
            self,
            std::str::from_utf8(bytes).expect("terminal output is UTF-8"),
        );
    }

    fn size(&self) -> Option<(u16, u16)> {
        self.size.get()
    }

    fn is_tty(&self) -> bool {
        self.tty
    }
}

/// Mounts and runs the first step. CI is false and the default color level is
/// Basic, matching Ink's test helpers rather than the process environment.
pub fn mount(element: impl Into<Element>, options: Options, terminal: &TestTerminal) -> TestApp {
    let interactive = resolve_interactive(options.interactive, false, terminal.is_tty());
    let now = Instant::now();
    let mut app = TestApp {
        app: App::new(
            element.into(),
            options,
            Box::new(terminal.clone()),
            interactive,
            options.color_level.unwrap_or(ColorLevel::Basic),
            now,
        ),
        now,
        terminal: terminal.clone(),
    };
    app.step();
    app
}

/// A live app driven by a fake clock. Drop unmounts and restores the terminal.
pub struct TestApp {
    app: App,
    now: Instant,
    terminal: TestTerminal,
}

impl TestApp {
    /// Hands over the terminal until [`Self::resume_terminal`].
    /// Rejects a queued or active API suspension.
    pub fn suspend_terminal(&mut self) -> Result<(), crate::AlreadySuspended> {
        self.app.queue().request_suspension(Box::new(|| {}))?;
        drop(self.app.begin_suspension());
        Ok(())
    }

    /// Reclaims the terminal and runs a step.
    pub fn resume_terminal(&mut self) {
        self.app.resume(Reason::Api);
        self.step();
    }

    /// Feeds one stdin read at the current clock time, then runs a step.
    /// Input reaches active [`crate::Scope::use_input`], [`crate::Scope::use_paste`]
    /// or [`crate::Scope::use_focus`] hooks. A lone Esc or an incomplete
    /// sequence waits 20 ms; call [`Self::advance`] to flush it.
    pub fn input(&mut self, bytes: impl AsRef<[u8]>) {
        self.app.input(bytes.as_ref(), self.now);
        self.step();
    }

    /// Returns `None` while running, then the combined exit result.
    /// Exits come from [`crate::AppHandle`] requests, which the next app step
    /// honours after passive effects and terminal restoration.
    pub fn exit_result(&self) -> Option<&Result<Option<Box<dyn Any + Send>>, Error>> {
        self.app.exit_result()
    }

    /// Takes the exit result, including an owned exit value or panic payload.
    pub fn take_exit_result(&mut self) -> Option<Result<Option<Box<dyn Any + Send>>, Error>> {
        self.app.take_exit_result()
    }

    /// Reconciles the root and runs a step at the current clock time.
    pub fn rerender(&mut self, element: impl Into<Element>) {
        self.app.rerender(element.into());
        self.step();
    }

    /// Changes the terminal size and runs a resize step at the current time.
    pub fn resize(&mut self, columns: u16, rows: u16) {
        self.terminal.resize(columns, rows);
        self.step();
    }

    fn step(&mut self) -> Option<Instant> {
        if self.terminal.resized.replace(false) {
            self.app.resize(self.terminal.size());
        }
        let due = loop {
            let due = self.app.render_if_due(self.now);
            let Some(callback) = self.app.begin_suspension() else {
                break due;
            };
            callback();
            self.app.resume(Reason::Api);
        };
        // Effects can enqueue an event while a commit is running. Like an
        // event-source poll, deliver it on the next step, not inside Runtime.
        if self.terminal.resized.get() {
            Some(self.now)
        } else {
            due
        }
    }

    /// Runs a step now, then every due step through the target time.
    pub fn advance(&mut self, duration: Duration) {
        let target = self.now + duration;
        while let Some(due) = self.step() {
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
