use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::output::Output;
use crate::paint::{Cell, Frame, serialize_row};
use crate::runtime::{ExitResult, Runtime, UpdateQueue, WriteDestination};
use crate::terminal::Terminal;
use crate::text::ansi::{Token, tokenize};
use crate::{
    ColorLevel, CursorPosition, Element, KittyFlags, KittyKeyboard, OutputStream, RenderOptions,
    WindowSize,
};

#[cfg(test)]
mod tests;

/// Options for a live app, including [`crate::testing::mount`].
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Exit with `Ok(None)` on a Ctrl+C press or repeat (default true).
    /// Those events do not reach `use_input`; a release does.
    pub exit_on_ctrl_c: bool,
    /// Stop the process group on a Ctrl+Z press or repeat (default true), as
    /// the terminal's suspend key does. The shell's `fg` resumes the app.
    /// Those events do not reach `use_input`; a release does.
    pub suspend_on_ctrl_z: bool,
    /// Opt-in kitty keyboard negotiation. Defaults to [`KittyKeyboard::Disabled`].
    pub kitty_keyboard: KittyKeyboard,
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
            exit_on_ctrl_c: true,
            suspend_on_ctrl_z: true,
            kitty_keyboard: KittyKeyboard::Disabled,
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

fn needs_external_newline(data: &str) -> bool {
    tokenize(data)
        .into_iter()
        .rev()
        .find_map(|token| match token {
            Token::Text(text) => text.chars().rev().find(|&c| c != '\r'),
            _ => None,
        })
        .is_some_and(|c| c != '\n')
}

/// Keep synchronized-update boundaries in the same write as their payload.
pub(crate) fn synchronize(bytes: &mut Vec<u8>) {
    if !bytes.is_empty() {
        bytes.splice(..0, b"\x1b[?2026h".iter().copied());
        bytes.extend_from_slice(b"\x1b[?2026l");
    }
}

// Runtime runs user code without borrowing this destination. Only completed
// frames enter it, so effects and cleanups can write during a commit.
struct AppOutput {
    output: Output,
    terminal: Box<dyn Terminal>,
    columns: usize,
    rows: usize,
    tty: bool,
    interactive: bool,
    alternate: bool,
    paste_mode: bool,
    kitty_flags: Option<KittyFlags>,
    resume_kitty_flags: Option<KittyFlags>,
    suspended: bool,
    static_rows: Vec<Vec<Cell>>,
    last_frame: Option<Frame>,
    last_cursor: Option<(usize, usize)>,
    control: Vec<(OutputStream, Vec<u8>)>,
}

impl AppOutput {
    fn restore_modes(&self) -> Vec<u8> {
        // Ink App's unmount effect disables paste before finishUnmount pops
        // kitty flags, ahead of screen restoration or the non-interactive frame.
        let mut bytes = Vec::new();
        if self.paste_mode {
            bytes.extend_from_slice(b"\x1b[?2004l");
        }
        if self.kitty_flags.is_some() {
            bytes.extend_from_slice(b"\x1b[<u");
        }
        bytes
    }

    fn restore_bytes(&self) -> Vec<u8> {
        if self.suspended && self.interactive {
            return Vec::new();
        }
        let mut bytes = Vec::new();
        if self.alternate {
            bytes.extend_from_slice(b"\x1b[?1049l\x1b[?25h");
        } else if self.interactive {
            self.output.restore_bytes(&mut bytes);
            if self.tty {
                synchronize(&mut bytes);
            }
        }
        bytes.splice(..0, self.restore_modes());
        bytes
    }

    fn set_paste_mode(&mut self, enabled: bool) {
        let enabled = enabled && self.tty;
        if self.paste_mode != enabled {
            self.paste_mode = enabled;
            let bytes = if enabled {
                b"\x1b[?2004h"
            } else {
                b"\x1b[?2004l"
            };
            self.terminal.write(bytes, &self.restore_bytes());
        }
    }

    fn enable_kitty(&mut self, flags: KittyFlags) {
        self.kitty_flags = Some(flags);
        self.terminal.write(
            format!("\x1b[>{}u", flags.bits()).as_bytes(),
            &self.restore_bytes(),
        );
    }

    fn paint(
        &mut self,
        mut frame: Frame,
        cursor: Option<CursorPosition>,
        resize: Option<WindowSize>,
    ) {
        let mut bytes = Vec::new();
        if self.interactive {
            if let Some(previous) = resize {
                self.output.resize(
                    (
                        usize::from(previous.columns),
                        if self.tty {
                            usize::from(previous.rows)
                        } else {
                            usize::MAX
                        },
                    ),
                    (self.columns, if self.tty { self.rows } else { usize::MAX }),
                    self.alternate,
                    &mut bytes,
                );
            }
            let mut cursor = cursor.map(|position| (position.x, position.y));
            if self.alternate {
                // Ink fullStaticOutput resets on identity changes. Copy at most
                // one viewport of history, not the accumulated transcript.
                if frame.static_changed {
                    self.static_rows.clear();
                }
                self.static_rows.append(&mut frame.static_lines);
                let start = self.static_rows.len().saturating_sub(self.rows);
                let mut live = self.static_rows[start..].to_vec();
                // useCursor starts below Static; Output clips the combined rows.
                if let Some((_, y)) = &mut cursor {
                    *y = y.saturating_add(live.len());
                }
                live.append(&mut frame.live);
                frame.live = live;
            }
            // Ink renderInteractiveFrame never treats piped output as fullscreen.
            let rows = if self.tty { self.rows } else { usize::MAX };
            self.output
                .render(&frame, cursor, self.columns, rows, &mut bytes);
            frame.static_lines.clear();
            self.last_frame = Some(frame);
            self.last_cursor = cursor;
        } else {
            for row in &frame.static_lines {
                bytes.extend_from_slice(serialize_row(row).as_bytes());
                bytes.push(b'\n');
            }
        }
        self.write(bytes);
        // Control bytes follow the committed frame, even when its diff is empty.
        self.flush_control();
    }

    fn flush_control(&mut self) {
        let restore = self.restore_bytes();
        for (stream, bytes) in self.control.drain(..) {
            match stream {
                OutputStream::Stdout => self.terminal.write(&bytes, &restore),
                OutputStream::Stderr => self.terminal.write_stderr(&bytes, &restore),
            }
        }
    }

    fn write(&mut self, mut bytes: Vec<u8>) {
        if bytes.is_empty() {
            return;
        }
        if self.interactive && self.tty {
            synchronize(&mut bytes);
        }
        self.terminal.write(&bytes, &self.restore_bytes());
    }

    fn clear(&mut self) {
        if self.interactive && !self.suspended {
            let mut bytes = Vec::new();
            self.output.clear(&mut bytes);
            self.write(bytes);
        }
    }

    fn external_write(&mut self, stream: OutputStream, data: &str) {
        if self.interactive && self.suspended {
            return;
        }
        if !self.interactive {
            let restore = self.restore_bytes();
            match stream {
                OutputStream::Stdout => self.terminal.write(data.as_bytes(), &restore),
                OutputStream::Stderr => self.terminal.write_stderr(data.as_bytes(), &restore),
            }
            return;
        }
        let mut bytes = Vec::new();
        if self.tty {
            bytes.extend_from_slice(b"\x1b[?2026h");
        }
        self.output.clear(&mut bytes);
        // libuv-style raw mode preserves OPOST and enables ONLCR: leave data
        // newlines to the TTY. Unlike Ink, add a line break after text that
        // lacks one, ignoring escapes and CR, because cell-diff redraw starts
        // at column zero on the following row.
        let mut data_bytes = data.as_bytes().to_vec();
        if needs_external_newline(data) {
            data_bytes.extend_from_slice(b"\r\n");
        }
        match stream {
            OutputStream::Stdout => bytes.extend(data_bytes),
            OutputStream::Stderr => {
                if !bytes.is_empty() {
                    let mut restore = self.restore_bytes();
                    if self.tty {
                        restore.splice(..0, b"\x1b[?2026l".iter().copied());
                    }
                    self.terminal.write(&bytes, &restore);
                }
                bytes.clear();
                let mut restore = self.restore_bytes();
                if self.tty {
                    restore.splice(..0, b"\x1b[?2026l".iter().copied());
                }
                self.terminal.write_stderr(&data_bytes, &restore);
            }
        }
        // Ink restoreLastOutput restores painted output. Pending commits stay
        // in Runtime until the app's throttle permits their paint.
        if let Some(frame) = &self.last_frame {
            let rows = if self.tty { self.rows } else { usize::MAX };
            self.output
                .render(frame, self.last_cursor, self.columns, rows, &mut bytes);
        }
        if self.tty {
            bytes.extend_from_slice(b"\x1b[?2026l");
        }
        if !bytes.is_empty() {
            self.terminal.write(&bytes, &self.restore_bytes());
        }
    }

    fn restore(&mut self, mut final_output: Vec<u8>) {
        if self.interactive && self.suspended {
            self.terminal.restore(&[]);
            return;
        }
        if self.alternate {
            final_output.extend_from_slice(b"\x1b[?1049l\x1b[?25h");
        } else {
            if self.interactive {
                if let Some(frame) = &self.last_frame {
                    self.output.uncap(frame, self.columns, &mut final_output);
                }
                self.output.done(&mut final_output);
            }
            if self.interactive && self.tty {
                synchronize(&mut final_output);
            }
        }
        final_output.splice(..0, self.restore_modes());
        self.terminal.restore(&final_output);
        self.paste_mode = false;
        self.kitty_flags = None;
    }
}

impl WriteDestination for RefCell<AppOutput> {
    fn write(&self, stream: OutputStream, data: &str) {
        self.borrow_mut().external_write(stream, data);
    }

    fn write_control(&self, stream: OutputStream, bytes: &[u8]) {
        self.borrow_mut().control.push((stream, bytes.to_vec()));
    }
}

pub(crate) enum Reason {
    Api,
    Stop,
}

pub(crate) struct App {
    api_suspended: bool,
    stop_suspended: bool,
    raw_mode: bool,
    kitty_query: Option<(KittyFlags, Instant)>,
    runtime: Option<Runtime>,
    queue: Arc<UpdateQueue>,
    exit_result: Option<ExitResult>,
    destination: Rc<RefCell<AppOutput>>,
    interactive: bool,
    window: Duration,
    deadline: Option<Instant>,
    pending: bool,
    resize_from: Option<WindowSize>,
}

impl App {
    pub(crate) fn is_suspended(&self) -> bool {
        self.api_suspended || self.stop_suspended
    }

    pub(crate) fn begin_suspension(&mut self) -> Option<Box<dyn FnOnce() + Send>> {
        if self.runtime.is_none() || self.queue.has_exit() {
            return None;
        }
        let callback = self.queue.take_suspension()?;
        self.suspend(Reason::Api);
        Some(callback)
    }

    pub(crate) fn suspend(&mut self, reason: Reason) {
        let suspended = self.is_suspended();
        match reason {
            Reason::Api => self.api_suspended = true,
            Reason::Stop => self.stop_suspended = true,
        }
        if suspended || self.runtime.is_none() {
            return;
        }
        self.kitty_query = None;
        self.runtime.as_mut().unwrap().pause_input();
        let mut output = self.destination.borrow_mut();
        let mut bytes = output.restore_modes();
        if output.interactive {
            if output.alternate {
                bytes.extend_from_slice(b"\x1b[?1049l\x1b[?25h");
            } else {
                let mut screen = Vec::new();
                output.output.clear(&mut screen);
                output.output.done(&mut screen);
                if output.tty {
                    synchronize(&mut screen);
                }
                bytes.extend(screen);
            }
        }
        output.terminal.write(&bytes, &[]);
        output.paste_mode = false;
        output.resume_kitty_flags = output.kitty_flags.take();
        output.suspended = true;
        if self.raw_mode {
            output.terminal.set_raw_mode(false);
            self.raw_mode = false;
        }
    }

    pub(crate) fn resume(&mut self, reason: Reason) {
        if self.runtime.is_none() || self.queue.has_exit() {
            return;
        }
        let flag = match reason {
            Reason::Api => &mut self.api_suspended,
            Reason::Stop => &mut self.stop_suspended,
        };
        if !std::mem::take(flag) {
            return;
        }
        if matches!(reason, Reason::Api) {
            self.queue.resume();
        }
        if self.is_suspended() {
            return;
        }
        if self.interactive {
            self.resize(self.terminal_size());
        }
        let mut output = self.destination.borrow_mut();
        output.suspended = false;
        let mut bytes = Vec::new();
        if output.alternate {
            bytes.extend_from_slice(b"\x1b[?1049h\x1b[H");
        }
        output.kitty_flags = output.resume_kitty_flags.take();
        if let Some(flags) = output.kitty_flags {
            bytes.extend_from_slice(format!("\x1b[>{}u", flags.bits()).as_bytes());
        }
        let restore = output.restore_bytes();
        output.terminal.write(&bytes, &restore);
        if self.interactive {
            output.output = Output::new();
            output.last_frame = None;
            output.last_cursor = None;
            self.resize_from = None;
            self.pending = true;
            self.deadline = None;
        }
    }

    pub(crate) fn raw_mode(&self) -> bool {
        self.raw_mode
    }

    pub(crate) fn queue(&self) -> Arc<UpdateQueue> {
        self.queue.clone()
    }

    pub(crate) fn exit_result(&self) -> Option<&ExitResult> {
        self.exit_result.as_ref()
    }

    pub(crate) fn take_exit_result(&mut self) -> Option<ExitResult> {
        self.exit_result.take()
    }

    pub(crate) fn terminal_size(&self) -> Option<(u16, u16)> {
        self.destination.borrow().terminal.size()
    }

    pub(crate) fn take_error(&mut self) -> Option<std::io::Error> {
        self.destination.borrow_mut().terminal.take_error()
    }

    pub(crate) fn new(
        element: Element,
        options: Options,
        mut terminal: Box<dyn Terminal>,
        interactive: bool,
        color_level: ColorLevel,
        now: Instant,
    ) -> Self {
        crate::runtime::panic::install();
        let size = WindowSize::resolve(terminal.size());
        let WindowSize { columns, rows } = size;
        let tty = terminal.is_tty();
        let alternate = options.alternate_screen && interactive && tty;
        if alternate {
            // design.md starts the alternate frame at the first row.
            terminal.write(b"\x1b[?1049h\x1b[H", b"\x1b[?1049l\x1b[?25h");
        }
        let mut runtime = Runtime::new(
            element,
            RenderOptions {
                columns,
                color_level,
            },
        );
        runtime.resize(size);
        runtime.catch = true;
        runtime.exit_on_ctrl_c = options.exit_on_ctrl_c;
        runtime.suspend_on_ctrl_z = options.suspend_on_ctrl_z;
        let destination = Rc::new(RefCell::new(AppOutput {
            output: Output::new(),
            terminal,
            columns: usize::from(columns),
            rows: usize::from(rows),
            tty,
            interactive,
            alternate,
            paste_mode: false,
            kitty_flags: None,
            resume_kitty_flags: None,
            suspended: false,
            static_rows: Vec::new(),
            last_frame: None,
            last_cursor: None,
            control: Vec::new(),
        }));
        // Ink initKittyKeyboard follows setAlternateScreen, before the first frame.
        let mut kitty_query = None;
        {
            let mut output = destination.borrow_mut();
            if tty && output.terminal.stdin_is_tty() {
                match options.kitty_keyboard {
                    KittyKeyboard::Enabled(flags) => output.enable_kitty(flags),
                    KittyKeyboard::Auto(flags) if interactive => {
                        kitty_query = Some((flags, now + Duration::from_millis(200)));
                        let restore = output.restore_bytes();
                        output.terminal.write(b"\x1b[?u", &restore);
                    }
                    _ => {}
                }
            }
        }
        runtime.set_destination(destination.clone());
        Self {
            api_suspended: false,
            stop_suspended: false,
            raw_mode: false,
            kitty_query,
            queue: runtime.queue(),
            runtime: Some(runtime),
            exit_result: None,
            destination,
            interactive,
            window: Duration::from_millis(if options.max_fps == 0 {
                0
            } else {
                u64::from(1000_u32.div_ceil(options.max_fps))
            }),
            deadline: None,
            pending: false,
            resize_from: None,
        }
    }

    pub(crate) fn resize(&mut self, size: Option<(u16, u16)>) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };
        let size = WindowSize::resolve(size);
        let mut destination = self.destination.borrow_mut();
        if self.interactive && !destination.suspended && self.resize_from.is_none() {
            self.resize_from = Some(WindowSize {
                columns: destination.columns as u16,
                rows: destination.rows as u16,
            });
        }
        destination.columns = usize::from(size.columns);
        destination.rows = usize::from(size.rows);
        runtime.resize(size);
    }

    pub(crate) fn rerender(&mut self, element: Element) {
        if let Some(runtime) = &mut self.runtime {
            runtime.rerender(element);
        }
    }

    pub(crate) fn render_if_due(&mut self, now: Instant) -> Option<Instant> {
        let suspended = self.is_suspended();
        if self
            .kitty_query
            .is_some_and(|(_, deadline)| now >= deadline)
        {
            self.kitty_query = None;
        }
        let runtime = self.runtime.as_mut()?;
        runtime.tick_animations(now, self.window);
        runtime.flush_input(now);
        if runtime.take_stop_request() {
            self.destination.borrow_mut().terminal.stop_process();
        }
        runtime.queue().drain();
        let commit = runtime.has_work().then(|| {
            let commit = runtime.commit();
            if Self::report_panic(runtime) {
                runtime.commit()
            } else {
                commit
            }
        });
        if commit.is_some() {
            self.pending = true;
        }
        let immediate = !(self.interactive && suspended)
            && (self.resize_from.is_some()
                || commit.as_ref().is_some_and(|commit| {
                    commit.static_items || (self.interactive && commit.static_changed)
                }));
        let due = self.interactive
            && !suspended
            && self.pending
            && self.deadline.is_none_or(|deadline| now >= deadline);
        if immediate || due {
            self.paint();
            if !immediate {
                self.deadline = Some(now + self.window);
            }
        }
        let runtime = self.runtime.as_mut().unwrap();
        if commit.is_some() {
            runtime.passive_effects();
        }
        if immediate || due {
            let cursor = runtime.cursor();
            runtime.after_write();
            self.pending |= cursor != runtime.cursor();
        }
        Self::report_panic(runtime);
        let stdin_tty = self.destination.borrow().terminal.stdin_is_tty();
        let holders = runtime.input_holders();
        // Ink use-focus.ts skips setRawMode on unsupported stdin, unlike
        // use_input and use_paste, which retain their Unsupported error.
        if !stdin_tty && holders > runtime.focus_holders() {
            self.queue.request_exit(Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "raw mode is not supported on stdin",
            )
            .into()));
        }
        let raw_mode = stdin_tty && holders > 0;
        let paste_mode = raw_mode && runtime.paste_holders() > 0;
        // Ink switches paste inside effects; end-of-step comparison avoids
        // mode writes for a use_paste swap in one commit. Disable paste before
        // raw mode, and enable it after raw mode succeeds (use-paste.ts).
        if !suspended {
            if !paste_mode {
                self.destination.borrow_mut().set_paste_mode(false);
            }
            if raw_mode != self.raw_mode {
                let mut destination = self.destination.borrow_mut();
                // Ink defers disable to a microtask; the end of a step gives
                // same-commit holder swaps the same terminal-mode lifetime.
                destination.terminal.set_raw_mode(raw_mode);
                self.raw_mode = raw_mode;
            }
            self.destination
                .borrow_mut()
                .set_paste_mode(self.raw_mode && paste_mode);
        }
        // With no frame to wait for, control bytes leave at the end of this step.
        if !self.interactive || !(suspended || self.pending || runtime.has_work()) {
            self.destination.borrow_mut().flush_control();
        }
        if self.queue.has_exit() {
            self.unmount();
            return None;
        }
        let wake = if runtime.has_work() || runtime.queue().has_pending() {
            Some(now)
        } else if self.pending && self.interactive && !suspended {
            Some(self.deadline.unwrap_or(now))
        } else {
            None
        };
        wake.into_iter()
            .chain(runtime.input_deadline())
            .chain(runtime.animation_deadline(self.window))
            .min()
    }

    pub(crate) fn input(&mut self, bytes: &[u8], now: Instant) {
        if self.is_suspended() {
            return;
        }
        if self
            .kitty_query
            .is_some_and(|(_, deadline)| now >= deadline)
        {
            self.kitty_query = None;
        }
        if self.raw_mode
            && let Some(runtime) = &mut self.runtime
        {
            if runtime.input(bytes, now)
                && let Some((flags, _)) = self.kitty_query.take()
            {
                self.destination.borrow_mut().enable_kitty(flags);
            }
            if runtime.take_stop_request() {
                self.destination.borrow_mut().terminal.stop_process();
            }
        }
    }

    fn paint(&mut self) {
        let runtime = self.runtime.as_mut().unwrap();
        let frame = runtime.paint();
        self.destination
            .borrow_mut()
            .paint(frame, runtime.cursor(), self.resize_from.take());
        self.pending = false;
    }

    fn report_panic(runtime: &mut Runtime) -> bool {
        let Some(error) = runtime.take_panic() else {
            return false;
        };
        // Ink ErrorBoundary renders ErrorOverview before handleExit unmounts.
        runtime.rerender(crate::error_overview::build(
            error.message(),
            error
                .location()
                .map(|location| (location.file.as_str(), location.line, location.column)),
            error.backtrace().as_deref(),
        ));
        runtime.queue().request_failure(crate::Error::Panic(error));
        true
    }

    pub(crate) fn clear(&mut self) {
        if self.runtime.is_some() {
            self.destination.borrow_mut().clear();
        }
    }

    pub(crate) fn unmount(&mut self) {
        if self.runtime.is_none() {
            return;
        }
        self.kitty_query = None;
        if self.raw_mode {
            self.destination.borrow_mut().terminal.set_raw_mode(false);
            self.raw_mode = false;
        }
        if std::thread::panicking() {
            // Cleanup runs user code: a second panic aborts the process, so
            // restore before dropping the tree rather than Ink's normal order.
            self.runtime.as_mut().unwrap().begin_unmount();
            self.destination.borrow_mut().control.clear();
            self.destination.borrow_mut().restore(Vec::new());
            drop(self.runtime.take());
            return;
        }
        let mut final_output = Vec::new();
        let runtime = self.runtime.as_mut().unwrap();
        let committed = runtime.has_work();
        if committed {
            runtime.commit();
            if Self::report_panic(runtime) {
                runtime.commit();
            }
            self.pending = true;
        }
        if self.interactive {
            if self.pending && !self.is_suspended() {
                self.paint();
            }
        } else {
            let frame = self.runtime.as_mut().unwrap().paint();
            let mut static_output = Vec::new();
            for row in &frame.static_lines {
                static_output.extend_from_slice(serialize_row(row).as_bytes());
                static_output.push(b'\n');
            }
            self.destination.borrow_mut().write(static_output);
            final_output = frame
                .live
                .iter()
                .map(|row| serialize_row(row))
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes();
            final_output.push(b'\n');
        }
        {
            let mut output = self.destination.borrow_mut();
            // Finish queued control writes before cleanup, unless the terminal is handed over.
            if self.interactive && self.is_suspended() {
                output.control.clear();
            } else {
                output.flush_control();
            }
        }
        let runtime = self.runtime.as_mut().unwrap();
        runtime.begin_unmount();
        if committed {
            runtime.passive_effects();
        }
        if !(self.interactive && (self.api_suspended || self.stop_suspended)) {
            runtime.after_write();
        }
        // Ink finishUnmount restores the terminal after React cleanups.
        drop(self.runtime.take());
        self.destination.borrow_mut().restore(final_output);
        // Keep the request in the queue until it is closed, so cleanup and
        // worker requests cannot replace the first result during teardown.
        if let Some(result) = self.queue.take_exit() {
            let error = self.take_error();
            self.exit_result = Some(match result {
                Err(error) => Err(error),
                Ok(value) => error.map_or(Ok(value), |error| Err(error.into())),
            });
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.unmount();
    }
}
