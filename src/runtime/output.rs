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
    fn write_control(&self, stream: OutputStream, bytes: &[u8]);
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
    #[track_caller]
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

    /// Queues raw bytes, such as a graphics or OSC sequence, for this stream.
    /// The bytes go out unchanged after the frame that includes every state
    /// change made before the call. When that step paints nothing, they go out
    /// at the end of the step.
    /// The frame is not erased or redrawn, no line break is added, and the bytes
    /// do not count toward the frame height. Bytes that print text or move the
    /// cursor without restoring it put the terminal out of step with the next
    /// frame's diff.
    /// While the terminal is handed over (`suspend_terminal`, Ctrl+Z),
    /// interactive apps keep the bytes until the first frame after the handover
    /// ends. Unmount during a handover drops them.
    /// Empty writes and writes after app teardown begins are ignored.
    /// Panics in debug builds during render. Without a driver
    /// ([`crate::testing::render`]), writes are dropped.
    #[track_caller]
    pub fn write_control(self, bytes: impl AsRef<[u8]>) {
        assert_eq!(
            self.thread,
            thread::current().id(),
            "output access off the UI thread"
        );
        assert_not_rendering();
        let bytes = bytes.as_ref();
        if bytes.is_empty() {
            return;
        }
        let Some(store) = resolve_store(self.runtime) else {
            return;
        };
        if store.unmounting.get() {
            return;
        }
        let destination = store.destination.borrow().clone();
        if let Some(destination) = destination {
            destination.write_control(self.stream, bytes);
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
