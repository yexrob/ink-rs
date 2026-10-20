use std::rc::Rc;
use std::thread::{self, ThreadId};

use super::state::{Store, assert_not_rendering, resolve_store};

/// A position in cells from the live frame's top-left, below Static output.
/// Coordinates are zero-based; the output layer clips them to the visible frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CursorPosition {
    /// Column in the live frame.
    pub x: usize,
    /// Row in the live frame.
    pub y: usize,
}

/// The destination stream of an external write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

pub(crate) trait WriteDestination {
    fn write(&self, stream: OutputStream, data: &str);
}

/// A copyable UI-thread writer returned by [`super::Scope::use_stdout`] or
/// [`super::Scope::use_stderr`]. It is `Send + Sync`, but writing from a thread
/// other than the owning UI thread panics. Use [`super::AppHandle::update`].
#[derive(Clone, Copy)]
pub struct OutputWriter {
    runtime: u64,
    stream: OutputStream,
    thread: ThreadId,
}

impl OutputWriter {
    pub(super) fn new(store: &Store, stream: OutputStream) -> Self {
        Self {
            runtime: store.id,
            stream,
            thread: thread::current().id(),
        }
    }

    /// Writes to the selected stream and restores the last painted frame.
    /// Empty writes and writes after app teardown begins are ignored.
    /// Panics in debug builds during render. Without a driver, writes are dropped.
    pub fn write(self, data: &str) {
        assert_eq!(
            self.thread,
            thread::current().id(),
            "output access off the UI thread"
        );
        assert_not_rendering();
        if data.is_empty() {
            return;
        }
        let Some(store) = resolve_store(self.runtime) else {
            return;
        };
        if store.unmounting.get() {
            return;
        }
        // Release the registry and destination borrows before calling the driver.
        let destination = store.destination.borrow().clone();
        if let Some(destination) = destination {
            destination.write(self.stream, data);
        }
    }
}

pub(super) struct CursorCleanup(pub(super) Rc<Store>);

impl Drop for CursorCleanup {
    fn drop(&mut self) {
        // Ink use-cursor.ts clears the shared position regardless of its owner.
        self.0.cursor.set(None);
    }
}
