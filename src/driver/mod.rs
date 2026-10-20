use std::any::Any;
use std::io;
use std::sync::{Arc, mpsc};
use std::time::Instant;

use crate::app::{App, resolve_interactive};
use crate::runtime::{UpdateQueue, Wake};
use crate::terminal::Terminal;
use crate::{AppHandle, Element, Error, Options};
use process::{ProcessTerminal, Registration};

mod environment;
mod process;

#[cfg(test)]
mod tests;

pub(super) enum Message {
    Wake,
    Resize,
    Signal,
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
    let terminal = ProcessTerminal::new();
    let env = |key: &str| std::env::var_os(key).map(|value| value.to_string_lossy().into_owned());
    let interactive = resolve_interactive(
        options.interactive,
        environment::in_ci(env),
        terminal.is_tty(),
    );
    let color = options
        .color_level
        .unwrap_or_else(|| environment::color_level(terminal.is_tty(), env));
    let (sender, receiver) = mpsc::channel();
    let registration = terminal.register(sender.clone());
    let mut app = App::new(
        element.into(),
        options,
        Box::new(terminal),
        interactive,
        color,
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
    ///
    /// To end a blocked wait from another thread, take an [`Instance::handle`]
    /// before waiting and call [`AppHandle::exit`] on it.
    /// Termination signals restore the terminal and retain their default effect.
    pub fn wait_until_exit(mut self) -> Result<Option<Box<dyn Any + Send>>, Error> {
        self.registration.as_mut().unwrap().enter_loop();
        let result = run_loop(&mut self.app, &self.receiver);
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

fn natural_end(next: Option<Instant>, queue: &UpdateQueue) -> bool {
    // Ink waitUntilExit uses Node's beforeExit. M3 input and M4 timers extend
    // this one liveness condition rather than adding polls to the loop.
    next.is_none() && queue.is_idle()
}

fn run_loop(app: &mut App, receiver: &mpsc::Receiver<Message>) -> Stop {
    let queue = app.queue();
    let mut message = None;
    loop {
        for event in message.take().into_iter().chain(receiver.try_iter()) {
            match event {
                Message::Wake => {}
                Message::Resize => app.resize(app.terminal_size()),
                Message::Signal => return Stop::Signal,
            }
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
        if natural_end(next, &queue) {
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
