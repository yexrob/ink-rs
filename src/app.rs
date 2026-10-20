use std::time::{Duration, Instant};

use crate::output::Output;
use crate::paint::{Cell, serialize_row};
use crate::runtime::Runtime;
use crate::terminal::Terminal;
use crate::{ColorLevel, Element, RenderOptions};

/// Options for a live app, including [`crate::testing::mount`].
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Paint rate limit, default 30. The window is ceil(1000 / max_fps)
    /// milliseconds; zero uses a zero-millisecond window, matching Ink.
    pub max_fps: u32,
    /// Override interactive mode. `None` means stdout is a TTY and not in CI.
    pub interactive: Option<bool>,
    /// Use the alternate screen when interactive and stdout is a TTY.
    pub alternate_screen: bool,
    /// Color capability. `None` asks the driver to detect the environment.
    pub color_level: Option<ColorLevel>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_fps: 30,
            interactive: None,
            alternate_screen: false,
            color_level: None,
        }
    }
}

pub(crate) fn resolve_interactive(option: Option<bool>, in_ci: bool, tty: bool) -> bool {
    option.unwrap_or(tty && !in_ci)
}

/// Keep synchronized-update boundaries in the same write as their payload.
pub(crate) fn synchronize(bytes: &mut Vec<u8>) {
    if !bytes.is_empty() {
        bytes.splice(..0, b"\x1b[?2026h".iter().copied());
        bytes.extend_from_slice(b"\x1b[?2026l");
    }
}

pub(crate) struct App {
    runtime: Option<Runtime>,
    output: Output,
    terminal: Box<dyn Terminal>,
    columns: usize,
    rows: usize,
    tty: bool,
    interactive: bool,
    alternate: bool,
    window: Duration,
    deadline: Option<Instant>,
    pending: bool,
    static_rows: Vec<Vec<Cell>>,
}

impl App {
    pub(crate) fn new(
        element: Element,
        options: Options,
        mut terminal: Box<dyn Terminal>,
        interactive: bool,
        color_level: ColorLevel,
    ) -> Self {
        let (columns, rows) = terminal.size().unwrap_or((80, 24));
        let columns = columns.max(1);
        let rows = rows.max(1);
        let tty = terminal.is_tty();
        let alternate = options.alternate_screen && interactive && tty;
        if alternate {
            // design.md starts the alternate frame at the first row.
            terminal.write(b"\x1b[?1049h\x1b[H");
        }
        Self {
            runtime: Some(Runtime::new(
                element,
                RenderOptions {
                    columns,
                    color_level,
                },
            )),
            output: Output::new(),
            terminal,
            columns: usize::from(columns),
            rows: usize::from(rows),
            tty,
            interactive,
            alternate,
            window: Duration::from_millis(if options.max_fps == 0 {
                0
            } else {
                u64::from(1000_u32.div_ceil(options.max_fps))
            }),
            deadline: None,
            pending: false,
            static_rows: Vec::new(),
        }
    }

    pub(crate) fn rerender(&mut self, element: Element) {
        if let Some(runtime) = &mut self.runtime {
            runtime.rerender(element);
        }
    }

    pub(crate) fn render_if_due(&mut self, now: Instant) -> Option<Instant> {
        let runtime = self.runtime.as_mut()?;
        let commit = runtime.has_work().then(|| runtime.commit());
        if commit.is_some() {
            self.pending = true;
        }
        let immediate = commit.as_ref().is_some_and(|commit| {
            commit.static_items || (self.interactive && commit.static_changed)
        });
        let due = self.interactive
            && self.pending
            && self.deadline.is_none_or(|deadline| now >= deadline);
        if immediate || due {
            let bytes = self.paint();
            self.write(bytes);
            if !immediate {
                self.deadline = Some(now + self.window);
            }
        }
        let runtime = self.runtime.as_mut().unwrap();
        if commit.is_some() {
            runtime.passive_effects();
        }
        if immediate || due {
            runtime.after_write();
        }
        if runtime.has_work() {
            Some(now)
        } else if self.pending && self.interactive {
            self.deadline
        } else {
            None
        }
    }

    fn paint(&mut self) -> Vec<u8> {
        let mut frame = self.runtime.as_mut().unwrap().paint();
        let mut bytes = Vec::new();
        if self.interactive {
            if self.alternate {
                // Ink fullStaticOutput resets on identity changes. Copy at most
                // one viewport of history, not the accumulated transcript.
                if frame.static_changed {
                    self.static_rows.clear();
                }
                self.static_rows.append(&mut frame.static_lines);
                let start = self.static_rows.len().saturating_sub(self.rows);
                let mut live = self.static_rows[start..].to_vec();
                live.append(&mut frame.live);
                frame.live = live;
            }
            // Ink renderInteractiveFrame never treats piped output as fullscreen.
            let rows = if self.tty { self.rows } else { usize::MAX };
            self.output
                .render(&frame, None, self.columns, rows, &mut bytes);
        } else {
            for row in &frame.static_lines {
                bytes.extend_from_slice(serialize_row(row).as_bytes());
                bytes.push(b'\n');
            }
        }
        self.pending = false;
        bytes
    }

    fn write(&mut self, mut bytes: Vec<u8>) {
        if bytes.is_empty() {
            return;
        }
        if self.interactive && self.tty {
            synchronize(&mut bytes);
        }
        self.terminal.write(&bytes);
    }

    pub(crate) fn clear(&mut self) {
        if self.interactive && self.runtime.is_some() {
            let mut bytes = Vec::new();
            self.output.clear(&mut bytes);
            self.write(bytes);
        }
    }

    pub(crate) fn unmount(&mut self) {
        if self.runtime.is_none() {
            return;
        }
        if std::thread::panicking() {
            // Cleanup runs user code: a second panic aborts the process, so
            // restore before dropping the tree rather than Ink's normal order.
            self.restore(Vec::new());
            drop(self.runtime.take());
            return;
        }
        let mut final_output = Vec::new();
        let runtime = self.runtime.as_mut().unwrap();
        let committed = runtime.has_work();
        if committed {
            runtime.commit();
            self.pending = true;
        }
        if self.interactive {
            if self.pending {
                let bytes = self.paint();
                self.write(bytes);
            }
        } else {
            let frame = self.runtime.as_mut().unwrap().paint();
            let mut static_output = Vec::new();
            for row in &frame.static_lines {
                static_output.extend_from_slice(serialize_row(row).as_bytes());
                static_output.push(b'\n');
            }
            self.write(static_output);
            final_output = frame
                .live
                .iter()
                .map(|row| serialize_row(row))
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes();
            final_output.push(b'\n');
        }
        let runtime = self.runtime.as_mut().unwrap();
        if committed {
            runtime.passive_effects();
        }
        runtime.after_write();
        // Ink finishUnmount restores the terminal after React cleanups.
        drop(self.runtime.take());
        self.restore(final_output);
    }

    fn restore(&mut self, mut final_output: Vec<u8>) {
        if self.alternate {
            final_output.extend_from_slice(b"\x1b[?1049l\x1b[?25h");
            self.terminal.write(&final_output);
        } else {
            if self.interactive {
                self.output.done(&mut final_output);
            }
            self.write(final_output);
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.unmount();
    }
}
