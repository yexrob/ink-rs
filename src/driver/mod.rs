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
mod sys;
#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(test)]
mod tests;

pub(super) enum Message {
    Wake,
    Resize,
    Signal,
    #[cfg(unix)]
    Stop(mpsc::Sender<()>),
    #[cfg(unix)]
    Continue,
    Input(Vec<u8>),
    InputStarted,
    InputEnded,
}

pub(super) trait Post: Send + Sync + 'static {
    fn post(&self, message: Message);
}

impl Post for mpsc::Sender<Message> {
    fn post(&self, message: Message) {
        let _ = self.send(message);
    }
}

#[cfg(feature = "tokio")]
impl Post for ::tokio::sync::mpsc::UnboundedSender<Message> {
    fn post(&self, message: Message) {
        let _ = self.send(message);
    }
}

impl<T: Post> Wake for T {
    fn wake(&self) {
        self.post(Message::Wake);
    }
}

/// Mounts on process stdout and runs the first step before returning.
/// Non-interactive apps defer their live frame until unmount, as Ink does.
/// Call [`Instance::wait_until_exit`] to drive subsequent updates.
pub fn render(element: impl Into<Element>, options: Options) -> Instance {
    let (sender, receiver) = mpsc::channel();
    Instance {
        mounted: mount(element.into(), options, Arc::new(sender), Instant::now),
        receiver,
    }
}

fn mount<S: Post>(
    element: Element,
    mut options: Options,
    sender: Arc<S>,
    now: fn() -> Instant,
) -> Mounted {
    if !sys::JOB_CONTROL {
        options.suspend_on_ctrl_z = false;
    }
    let terminal = ProcessTerminal::new(sender.clone());
    let env = |key: &str| std::env::var_os(key).map(|value| value.to_string_lossy().into_owned());
    let interactive = resolve_interactive(
        options.interactive,
        environment::in_ci(env),
        terminal.is_tty(),
    );
    let color = options
        .color_level
        .unwrap_or_else(|| environment::color_level(terminal.is_tty(), cfg!(windows), env));
    let registration = terminal.register(sender.clone());
    let mut app = App::new(
        element,
        options,
        Box::new(terminal),
        interactive,
        color,
        now(),
    );
    app.queue().set_wake(sender);
    app.render_if_due(now());
    Mounted {
        app,
        registration: Some(registration),
        now,
    }
}

/// A mounted, single-threaded app. It is not `Send`; handles can cross threads.
/// Dropping it unmounts the tree and restores the terminal.
pub struct Instance {
    mounted: Mounted,
    receiver: mpsc::Receiver<Message>,
}

impl Instance {
    /// Replaces the root and runs a step at the current time.
    pub fn rerender(&mut self, element: impl Into<Element>) {
        self.mounted.rerender(element.into());
    }

    /// Clears the painted frame and runs a step at the current time.
    pub fn clear(&mut self) {
        self.mounted.clear();
    }

    /// Returns a thread-safe update and exit handle that keeps the event loop alive.
    pub fn handle(&self) -> AppHandle {
        self.mounted.handle()
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
        self.mounted.registration.as_mut().unwrap().enter_loop();
        let result = run_loop(
            &mut self.mounted.app,
            &self.receiver,
            self.mounted.registration.as_mut().unwrap(),
        );
        self.mounted.registration.as_mut().unwrap().leave_loop();
        // A Stop racing the loop's end must release its acknowledgement even
        // when the Signal result parks this thread with its receiver alive.
        for message in self.receiver.try_iter() {
            drop(message);
        }
        self.mounted.app.unmount();
        let error = self.mounted.app.take_error();
        self.mounted.registration.take();
        match result {
            Stop::Signal => {
                // Other UI threads must finish their cleanups before the last
                // registration re-raises the signal. Never return to user code.
                loop {
                    std::thread::park();
                }
            }
            Stop::Error(error) => Err(error.into()),
            Stop::Exit => self.mounted.app.take_exit_result().expect("exit result"),
            Stop::Idle => error.map_or(Ok(None), |error| Err(error.into())),
        }
    }
}

struct Mounted {
    app: App,
    registration: Option<Registration>,
    now: fn() -> Instant,
}

impl Mounted {
    fn rerender(&mut self, element: Element) {
        self.app.rerender(element);
        self.app.render_if_due((self.now)());
    }

    fn clear(&mut self) {
        self.app.clear();
        self.app.render_if_due((self.now)());
    }

    fn handle(&self) -> AppHandle {
        self.app.queue().handle()
    }
}

impl Drop for Mounted {
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

#[derive(Default)]
struct Looper {
    input_ended: bool,
    kept: VecDeque<Message>,
}

enum Next {
    Wait(Option<Instant>),
    Suspend(Box<dyn FnOnce() + Send>),
    Stop(Stop),
}

impl Looper {
    fn turn(
        &mut self,
        app: &mut App,
        registration: &mut Registration,
        mut first: Option<Message>,
        poll: &mut dyn FnMut() -> Option<Message>,
        now: &dyn Fn() -> Instant,
    ) -> Next {
        let queue = app.queue();
        loop {
            let mut input = Vec::new();
            for event in self
                .kept
                .drain(..)
                .chain(first.take())
                .chain(std::iter::from_fn(&mut *poll))
            {
                if let Message::Input(bytes) = event {
                    input.extend(bytes);
                    continue;
                }
                // Ink handleReadable drains stdin.read(). Keep non-input messages
                // as barriers so resize and termination retain their queue order.
                if !input.is_empty() {
                    app.input(&input, now());
                    input.clear();
                }
                match event {
                    Message::Wake => {}
                    Message::Resize => app.resize(app.terminal_size()),
                    Message::Signal => return Next::Stop(Stop::Signal),
                    #[cfg(unix)]
                    Message::Stop(ack) => {
                        app.suspend(Reason::Stop);
                        drop(ack);
                    }
                    #[cfg(unix)]
                    Message::Continue => app.resume(Reason::Stop),
                    Message::InputStarted => self.input_ended = false,
                    Message::InputEnded => self.input_ended = true,
                    Message::Input(_) => unreachable!(),
                }
            }
            if !input.is_empty() {
                app.input(&input, now());
            }
            if let Some(error) = app.take_error() {
                return Next::Stop(Stop::Error(error));
            }
            let next = app.render_if_due(now());
            if let Some(error) = app.take_error() {
                return Next::Stop(Stop::Error(error));
            }
            if app.exit_result().is_some() {
                return Next::Stop(Stop::Exit);
            }
            if let Some(callback) = app.begin_suspension() {
                registration.leave_loop();
                // leave_loop serializes with Stop fan-out. Release every preceding
                // ack before user code can block, preserving other message order.
                for event in std::iter::from_fn(&mut *poll) {
                    #[cfg(unix)]
                    if let Message::Stop(ack) = event {
                        app.suspend(Reason::Stop);
                        drop(ack);
                        continue;
                    }
                    self.kept.push_back(event);
                }
                return Next::Suspend(callback);
            }
            if natural_end(
                next,
                &queue,
                app.raw_mode() && !self.input_ended,
                app.is_suspended(),
            ) {
                // An acquire during this step can restart stdin after EOF. Observe
                // its Started notification before deciding that input is dead.
                if let Some(event) = poll() {
                    first = Some(event);
                    continue;
                }
                return Next::Stop(Stop::Idle);
            }
            return Next::Wait(next);
        }
    }

    fn resume(&mut self, app: &mut App, registration: &mut Registration) {
        registration.rejoin_loop();
        app.resume(Reason::Api);
    }
}

fn run_loop(
    app: &mut App,
    receiver: &mpsc::Receiver<Message>,
    registration: &mut Registration,
) -> Stop {
    let mut looper = Looper::default();
    let mut message = None;
    loop {
        match looper.turn(
            app,
            registration,
            message.take(),
            &mut || receiver.try_recv().ok(),
            &Instant::now,
        ) {
            Next::Stop(stop) => return stop,
            Next::Suspend(callback) => {
                callback();
                looper.resume(app, registration);
            }
            Next::Wait(next) => {
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
    }
}
