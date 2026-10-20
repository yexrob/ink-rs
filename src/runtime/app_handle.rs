use std::any::Any;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::Error;

/// A thread-safe handle for submitting updates and requesting app exit.
/// Retaining a handle keeps a live driver running until an exit request.
/// Drop handles when their work ends so the app can reach its natural end.
/// The first exit request wins; requests after unmount begins are ignored.
/// A caught component panic replaces a pending successful exit.
/// Exit methods wake the driver; teardown happens in the next app step.
pub struct AppHandle {
    queue: Arc<UpdateQueue>,
}

impl AppHandle {
    /// Requests exit with `Ok(None)` as the `wait_until_exit` result.
    pub fn exit(&self) {
        self.queue.request_exit(Ok(None));
    }

    /// Requests exit with `Ok(Some(value))` as the `wait_until_exit` result.
    /// The caller can downcast the boxed value to its original type.
    pub fn exit_with(&self, value: impl Any + Send) {
        self.queue.request_exit(Ok(Some(Box::new(value))));
    }

    /// Requests exit with `Err(Error::App(error))` as the `wait_until_exit`
    /// result. Accepts errors, `&str`, and `String`; does not print the error
    /// or change the process exit code.
    pub fn exit_with_error(&self, error: impl Into<Box<dyn std::error::Error + Send + Sync>>) {
        self.queue.request_exit(Err(Error::App(error.into())));
    }

    /// Queues a callback on the UI thread, in submission order, and wakes the
    /// driver. After unmount, the callback is dropped without being called.
    pub fn update(&self, callback: impl FnOnce() + Send + 'static) {
        let mut state = self.queue.state.lock().unwrap();
        if state.closed {
            return;
        }
        state.callbacks.push_back(Box::new(callback));
        self.queue.wake(&state);
    }
}

impl Clone for AppHandle {
    fn clone(&self) -> Self {
        self.queue.handle()
    }
}

impl Drop for AppHandle {
    fn drop(&mut self) {
        let mut state = self.queue.state.lock().unwrap();
        state.handles -= 1;
        if state.handles == 0 {
            self.queue.wake(&state);
        }
    }
}

pub(crate) trait Wake: Send + Sync {
    // Called under the queue lock: notify the driver without entering user code.
    fn wake(&self);
}

type Callback = Box<dyn FnOnce() + Send>;
pub(crate) type ExitResult = Result<Option<Box<dyn Any + Send>>, Error>;

#[derive(Default)]
struct QueueState {
    callbacks: VecDeque<Callback>,
    exit: Option<ExitResult>,
    handles: usize,
    closed: bool,
    wake: Option<Arc<dyn Wake>>,
}

#[derive(Default)]
pub(crate) struct UpdateQueue {
    state: Mutex<QueueState>,
}

impl UpdateQueue {
    pub(crate) fn request_failure(&self, error: Error) {
        let mut state = self.state.lock().unwrap();
        let previous = if !state.closed && !matches!(state.exit, Some(Err(_))) {
            let previous = state.exit.replace(Err(error));
            self.wake(&state);
            previous
        } else {
            Some(Err(error))
        };
        drop(state);
        // A displaced exit value can run user code in Drop.
        drop(previous);
    }

    pub(crate) fn request_exit(&self, request: ExitResult) {
        let mut state = self.state.lock().unwrap();
        if !state.closed && state.exit.is_none() {
            state.exit = Some(request);
            self.wake(&state);
        }
        // An ignored request may own a value whose Drop submits more work.
        // Release the lock before dropping that value.
        drop(state);
    }

    pub(crate) fn has_exit(&self) -> bool {
        self.state.lock().unwrap().exit.is_some()
    }

    pub(crate) fn take_exit(&self) -> Option<ExitResult> {
        self.state.lock().unwrap().exit.take()
    }

    pub(crate) fn handle(self: &Arc<Self>) -> AppHandle {
        self.state.lock().unwrap().handles += 1;
        AppHandle {
            queue: self.clone(),
        }
    }

    pub(crate) fn set_wake(&self, wake: Arc<dyn Wake>) {
        self.state.lock().unwrap().wake = Some(wake);
    }

    fn wake(&self, state: &QueueState) {
        if let Some(wake) = &state.wake {
            wake.wake();
        }
    }

    pub(crate) fn notify(&self) {
        self.wake(&self.state.lock().unwrap());
    }

    pub(crate) fn has_pending(&self) -> bool {
        !self.state.lock().unwrap().callbacks.is_empty()
    }

    pub(crate) fn is_idle(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.handles == 0 && state.callbacks.is_empty()
    }

    pub(crate) fn drain(&self) {
        // Do not hold the queue lock across user code or captured-value drops.
        let callbacks = std::mem::take(&mut self.state.lock().unwrap().callbacks);
        for callback in callbacks {
            callback();
        }
    }

    pub(crate) fn close(&self) {
        let callbacks = {
            let mut state = self.state.lock().unwrap();
            state.closed = true;
            state.wake = None;
            std::mem::take(&mut state.callbacks)
        };
        drop(callbacks);
    }
}
