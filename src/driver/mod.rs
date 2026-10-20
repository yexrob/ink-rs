use std::any::Any;
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, mpsc};
use std::time::Instant;

use crate::app::{App, Reason, resolve_interactive};
use crate::runtime::{UpdateQueue, Wake};
use crate::terminal::Terminal;
use crate::{AppHandle, Element, Error, Options};
use process::{ProcessTerminal, Registration};

mod environment;
mod process;
mod stdin;

#[cfg(test)]
mod tests;

pub(super) enum Message {
    Wake,
    Resize,
    Signal,
    Stop(mpsc::Sender<()>),
    Continue,
    Input(Vec<u8>),
    InputStarted,
    InputEnded,
}

impl Wake for mpsc::Sender<Message> {
    fn wake(&self) {
        let _ = self.send(Message::Wake);
    }
}

/// Mounts on process stdout and runs the first step before returning.
/// Non-interactive apps defer their live frame until unmount, as Ink does.
/// Call [`Instance::wait_until_exit`] to drive subsequent updates.
pub fn render(element: impl Into<Element>, options: Options) -> Instance {
    let (sender, receiver) = mpsc::channel();
    let terminal = ProcessTerminal::new(sender.clone());
    let env = |key: &str| std::env::var_os(key).map(|value| value.to_string_lossy().into_owned());
    let interactive = resolve_interactive(
        options.interactive,
        environment::in_ci(env),
        terminal.is_tty(),
    );
    let color = options
        .color_level
        .unwrap_or_else(|| environment::color_level(terminal.is_tty(), env));
    let registration = terminal.register(sender.clone());
    let mut app = App::new(
        element.into(),
        options,
        Box::new(terminal),
        interactive,
        color,
        Instant::now(),
    );
    app.queue().set_wake(Arc::new(sender));
    app.render_if_due(Instant::now());
    Instance {
        app,
        registration: Some(registration),
        receiver,
    }
}

/// A mounted, single-threaded app. It is not `Send`; handles can cross threads.
/// Dropping it unmounts the tree and restores the terminal.
pub struct Instance {
    app: App,
    registration: Option<Registration>,
    receiver: mpsc::Receiver<Message>,
}

impl Instance {
    /// Replaces the root and runs a step at the current time.
    pub fn rerender(&mut self, element: impl Into<Element>) {
        self.app.rerender(element.into());
        self.app.render_if_due(Instant::now());
    }

    /// Clears the painted frame and runs a step at the current time.
    pub fn clear(&mut self) {
        self.app.clear();
        self.app.render_if_due(Instant::now());
    }

    /// Returns a thread-safe update and exit handle that keeps the event loop alive.
    pub fn handle(&self) -> AppHandle {
        self.app.queue().handle()
    }

    /// Finishes pending work, runs cleanups, and restores the terminal.
    pub fn unmount(self) {
        drop(self);
    }

    /// Runs on the calling thread until an exit request or the natural end.
    ///
    /// Returns `Ok(None)` for `exit()` or the natural end, `Ok(Some(value))`
    /// for `exit_with(value)`, and `Err(Error::App(error))` for
    /// `exit_with_error(error)`. Terminal I/O failures, including a closed
    /// stdout pipe, return `Err(Error::Io(error))`.
    /// Component and live-effect panics return `Err(Error::Panic(error))`
    /// after displaying an overview and restoring the terminal.
    /// Call `std::panic::resume_unwind(error.into_payload())` to resume the
    /// panic with exit code 101. `panic = "abort"` cannot be caught.
    ///
    /// To end a blocked wait from another thread, take an [`Instance::handle`]
    /// before waiting and call [`AppHandle::exit`] on it.
    /// Termination signals restore the terminal and retain their default effect.
    pub fn wait_until_exit(mut self) -> Result<Option<Box<dyn Any + Send>>, Error> {
        self.registration.as_mut().unwrap().enter_loop();
        let result = run_loop(
            &mut self.app,
            &self.receiver,
            self.registration.as_mut().unwrap(),
        );
        self.registration.as_mut().unwrap().leave_loop();
        // A Stop racing the loop's end must release its acknowledgement even
        // when the Signal result parks this thread with its receiver alive.
        for message in self.receiver.try_iter() {
            drop(message);
        }
        self.app.unmount();
        let error = self.app.take_error();
        self.registration.take();
        match result {
            Stop::Signal => {
                // Other UI threads must finish their cleanups before the last
                // registration re-raises the signal. Never return to user code.
                loop {
                    std::thread::park();
                }
            }
            Stop::Error(error) => Err(error.into()),
            Stop::Exit => self.app.take_exit_result().expect("exit result"),
            Stop::Idle => error.map_or(Ok(None), |error| Err(error.into())),
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.app.unmount();
        self.registration.take();
    }
}

enum Stop {
    Idle,
    Exit,
    Error(io::Error),
    Signal,
}

fn natural_end(
    next: Option<Instant>,
    queue: &UpdateQueue,
    input_alive: bool,
    suspended: bool,
) -> bool {
    // Ink waitUntilExit uses Node's beforeExit; raw mode is stdin.ref().
    // Suspension releases raw mode but must not end an app waiting for fg.
    next.is_none() && queue.is_idle() && !input_alive && !suspended
}

fn run_loop(
    app: &mut App,
    receiver: &mpsc::Receiver<Message>,
    registration: &mut Registration,
) -> Stop {
    let queue = app.queue();
    let mut message = None;
    let mut input_ended = false;
    let mut kept = VecDeque::new();
    loop {
        let mut input = Vec::new();
        for event in kept
            .drain(..)
            .chain(message.take())
            .chain(receiver.try_iter())
        {
            if let Message::Input(bytes) = event {
                input.extend(bytes);
                continue;
            }
            // Ink handleReadable drains stdin.read(). Keep non-input messages
            // as barriers so resize and termination retain their queue order.
            if !input.is_empty() {
                app.input(&input, Instant::now());
                input.clear();
            }
            match event {
                Message::Wake => {}
                Message::Resize => app.resize(app.terminal_size()),
                Message::Signal => return Stop::Signal,
                Message::Stop(ack) => {
                    app.suspend(Reason::Stop);
                    drop(ack);
                }
                Message::Continue => app.resume(Reason::Stop),
                Message::InputStarted => input_ended = false,
                Message::InputEnded => input_ended = true,
                Message::Input(_) => unreachable!(),
            }
        }
        if !input.is_empty() {
            app.input(&input, Instant::now());
        }
        if let Some(error) = app.take_error() {
            return Stop::Error(error);
        }
        let next = app.render_if_due(Instant::now());
        if let Some(error) = app.take_error() {
            return Stop::Error(error);
        }
        if app.exit_result().is_some() {
            return Stop::Exit;
        }
        if let Some(callback) = app.begin_suspension() {
            registration.leave_loop();
            // leave_loop serializes with Stop fan-out. Release every preceding
            // ack before user code can block, preserving other message order.
            for event in receiver.try_iter() {
                if let Message::Stop(ack) = event {
                    app.suspend(Reason::Stop);
                    drop(ack);
                } else {
                    kept.push_back(event);
                }
            }
            callback();
            registration.rejoin_loop();
            app.resume(Reason::Api);
            continue;
        }
        if natural_end(
            next,
            &queue,
            app.raw_mode() && !input_ended,
            app.is_suspended(),
        ) {
            // An acquire during this step can restart stdin after EOF. Observe
            // its Started notification before deciding that input is dead.
            if let Ok(event) = receiver.try_recv() {
                message = Some(event);
                continue;
            }
            return Stop::Idle;
        }
        // mpsc retains notifications sent between the liveness check and recv.
        // Last-handle drop and queue insertion both send while holding the queue
        // lock; is_idle observes either the handle or its queued callback.
        message = match next {
            Some(deadline) => receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .ok(),
            None => receiver.recv().ok(),
        };
    }
}
