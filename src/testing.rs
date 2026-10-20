//! A mounted tree with frame history, driven by calls to [`TestInstance::settle`].

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::app::{App, resolve_interactive};
use crate::paint::serialize_row;
use crate::runtime::Runtime;
use crate::terminal::Terminal;
use crate::{ColorLevel, Element, Options, RenderOptions};

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
    let mut instance = TestInstance {
        runtime: Runtime::new(element.into(), options),
        frames: Vec::new(),
        static_output: String::new(),
    };
    instance.step();
    instance
}

/// A tree that retains state across frames. Dropping it runs unmount cleanups.
pub struct TestInstance {
    runtime: Runtime,
    frames: Vec<String>,
    static_output: String,
}

impl TestInstance {
    /// The last presented frame: accumulated Static rows, each followed by a
    /// newline, then live rows joined by newlines, matching Ink's debug output.
    /// Static history resets when the first Static instance changes.
    pub fn last_frame(&self) -> &str {
        self.frames.last().map_or("", String::as_str)
    }

    /// One debug-output frame per present, including unchanged output. Each
    /// contains accumulated Static rows and their newlines before the live rows.
    /// Static history resets when the first Static instance changes.
    pub fn frames(&self) -> &[String] {
        &self.frames
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
        self.runtime.render_if_due(Instant::now(), |frame| {
            // Ink handleStaticChange resets history before onRender appends it.
            if frame.static_changed {
                self.static_output.clear();
            }
            for row in &frame.static_lines {
                self.static_output.push_str(&serialize_row(row));
                // renderer.ts preserves the newline even for a blank Static row.
                self.static_output.push('\n');
            }
            let live = frame
                .live
                .iter()
                .map(|row| serialize_row(row))
                .collect::<Vec<_>>()
                .join("\n");
            self.frames.push(format!("{}{live}", self.static_output));
        })
    }
}

/// In-memory stdout. Clones share the write log.
#[derive(Clone)]
pub struct TestTerminal {
    writes: Rc<RefCell<Vec<String>>>,
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

    /// Returns one string per terminal write.
    pub fn writes(&self) -> Vec<String> {
        self.writes.borrow().clone()
    }

    /// Writes existing terminal history before mounting an app.
    pub fn write(&self, text: &str) {
        self.writes.borrow_mut().push(text.to_owned());
    }
}

impl Terminal for TestTerminal {
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
