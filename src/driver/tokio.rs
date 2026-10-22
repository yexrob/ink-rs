//! A driver for a Tokio local runtime or `tokio::task::LocalSet`.
//!
//! Both [`render`] and [`Instance::wait_until_exit`] must run inside one:
//! [`crate::Scope::use_future`] calls `tokio::task::spawn_local`.
//! Dropping the `wait_until_exit` future unmounts the app.
//! A sync `ink_rs::render(..).wait_until_exit()` on the same thread blocks
//! every task on it.
//!
//! ```no_run
//! use ink_rs::{Options, Scope, Text, component};
//!
//! #[tokio::main(flavor = "local")]
//! async fn main() -> Result<(), ink_rs::Error> {
//!     let element = component(|cx: &mut Scope, _: &()| {
//!         let label = cx.use_state(|| "waiting");
//!         cx.use_future((), move || async move {
//!             tokio::time::sleep(std::time::Duration::from_millis(100)).await;
//!             label.set("done");
//!         });
//!         Text::new(label.get()).into()
//!     }, ());
//!     ink_rs::tokio::render(element, Options::default())
//!         .wait_until_exit().await.map(drop)
//! }
//! ```

use std::any::Any;
use std::sync::Arc;
use std::time::Instant;

use ::tokio::sync::mpsc::{self, UnboundedReceiver};

use super::{Looper, Message, Mounted, Next, Stop, mount};
use crate::{AppHandle, Element, Error, Options};

fn now() -> Instant {
    // All app deadlines share Tokio's clock, including when time is paused.
    ::tokio::time::Instant::now().into_std()
}

/// Mounts on process stdout and runs the first step before returning.
/// Run inside a `tokio::task::LocalSet` or local runtime, as must
/// [`Instance::wait_until_exit`]. Non-interactive live frames wait for unmount.
pub fn render(element: impl Into<Element>, options: Options) -> Instance {
    let (sender, receiver) = mpsc::unbounded_channel();
    Instance {
        mounted: mount(element.into(), options, Arc::new(sender), now),
        receiver,
    }
}

/// A mounted app that is not `Send`. Dropping it unmounts the tree and
/// restores the terminal; its [`AppHandle`] can cross threads.
pub struct Instance {
    mounted: Mounted,
    receiver: UnboundedReceiver<Message>,
}

impl Instance {
    /// Replaces the root and runs a step at the Tokio clock's current time.
    pub fn rerender(&mut self, element: impl Into<Element>) {
        self.mounted.rerender(element.into());
    }

    /// Clears the painted frame and runs a step at the Tokio clock's current time.
    pub fn clear(&mut self) {
        self.mounted.clear();
    }

    /// Returns a thread-safe update and exit handle that keeps the loop alive.
    pub fn handle(&self) -> AppHandle {
        self.mounted.handle()
    }

    /// Finishes pending work, runs cleanups, and restores the terminal.
    pub fn unmount(self) {
        drop(self);
    }

    /// Drives updates until an exit request or the natural end.
    /// Run inside a `tokio::task::LocalSet` or local runtime.
    /// Dropping this future unmounts the app and restores the terminal.
    ///
    /// Results match [`crate::Instance::wait_until_exit`]: exit values are
    /// returned, I/O and app errors propagate, and component panics display
    /// an overview before returning [`Error::Panic`]. With
    /// [`crate::Termination::Handled`], termination signals retain their default
    /// effect after all app cleanups finish.
    pub async fn wait_until_exit(mut self) -> Result<Option<Box<dyn Any + Send>>, Error> {
        let registration = self.mounted.registration.as_mut().unwrap();
        // Tokio loops on one thread are concurrent, not nested blocking waits.
        registration.rejoin_loop();
        let mut looper = Looper::default();
        let mut message = None;
        let result = loop {
            match looper.turn(
                &mut self.mounted.app,
                registration,
                message.take(),
                &mut || self.receiver.try_recv().ok(),
                &now,
            ) {
                Next::Stop(stop) => break stop,
                Next::Wait(deadline) => {
                    message = match deadline {
                        Some(deadline) => {
                            ::tokio::time::timeout_at(deadline.into(), self.receiver.recv())
                                .await
                                .ok()
                                .flatten()
                        }
                        None => self.receiver.recv().await,
                    };
                }
                Next::Suspend(callback) => {
                    // The child may block; leave the UI thread free for tasks.
                    if let Err(error) = ::tokio::task::spawn_blocking(callback).await {
                        std::panic::resume_unwind(error.into_panic());
                    }
                    looper.resume(&mut self.mounted.app, registration);
                }
            }
        };
        registration.leave_loop();
        // Release Stop acknowledgements that raced the loop's end.
        while let Ok(message) = self.receiver.try_recv() {
            drop(message);
        }
        self.mounted.app.unmount();
        let error = self.mounted.app.take_error();
        self.mounted.registration.take();
        match result {
            Stop::Signal => {
                // Other apps on this thread must clean up; the last registration
                // terminates the process. Parking here would prevent that.
                std::future::pending::<()>().await;
                unreachable!()
            }
            Stop::Error(error) => Err(error.into()),
            Stop::Exit => self.mounted.app.take_exit_result().expect("exit result"),
            Stop::Idle => error.map_or(Ok(None), |error| Err(error.into())),
        }
    }
}
