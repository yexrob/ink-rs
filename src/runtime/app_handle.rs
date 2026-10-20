use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A thread-safe app handle. Retaining a handle keeps a live driver running.
/// Drop handles when their work ends so the app can reach its natural end.
pub struct AppHandle {
    queue: Arc<UpdateQueue>,
}

impl AppHandle {
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

#[derive(Default)]
struct QueueState {
    callbacks: VecDeque<Callback>,
    handles: usize,
    closed: bool,
    wake: Option<Arc<dyn Wake>>,
}

#[derive(Default)]
pub(crate) struct UpdateQueue {
    state: Mutex<QueueState>,
}

impl UpdateQueue {
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
