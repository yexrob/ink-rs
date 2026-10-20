use std::io::{self, Write};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(unix)]
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Once, OnceLock};
use std::thread::{self, ThreadId};

use super::{Message, Post, stdin, sys};
use crate::terminal::Terminal;

// Release the registry lock before restoration. Lock order is RestoreState ->
// stdin lifecycle -> input route -> failure; normal output takes RestoreState
// -> failure, and normal mode switches take lifecycle -> route -> failure.
// The hook can enter with RestoreState held, so stdin never takes RestoreState.
// Join holds lifecycle but neither route nor failure; the reader takes route
// -> failure only. No user code runs under these locks.
struct RestoreState {
    stdout: Option<sys::Output>,
    stderr: Option<sys::Output>,
    bytes: Vec<u8>,
    restored: bool,
    failure: Arc<Mutex<Failure>>,
}

#[derive(Default)]
pub(super) struct Failure {
    failed: bool,
    error: Option<io::Error>,
}

impl Failure {
    pub(super) fn take(&mut self) -> Option<io::Error> {
        self.error.take()
    }

    pub(super) fn record(&mut self, error: io::Error) {
        if !self.failed {
            self.failed = true;
            self.error = Some(error);
        }
    }
}

impl RestoreState {
    fn record_error(&mut self, error: io::Error) {
        self.failure.lock().unwrap().record(error);
    }

    fn write(&mut self, bytes: &[u8], stderr: bool) {
        if self.restored || self.failure.lock().unwrap().failed || bytes.is_empty() {
            return;
        }
        let file = if stderr {
            &mut self.stderr
        } else {
            &mut self.stdout
        };
        if let Some(file) = file
            && let Err(error) = file.write_all(bytes).and_then(|()| file.flush())
        {
            self.record_error(error);
        }
    }

    fn restore(&mut self, bytes: Option<&[u8]>) {
        if self.restored {
            return;
        }
        let snapshot = bytes.map_or_else(|| std::mem::take(&mut self.bytes), <[u8]>::to_vec);
        self.write(&snapshot, false);
        self.restored = true;
    }
}

pub(super) struct ProcessTerminal {
    state: Arc<Mutex<RestoreState>>,
    tty: bool,
    input: Arc<stdin::Target>,
}

impl ProcessTerminal {
    pub(super) fn new(sender: Arc<dyn Post>) -> Self {
        let failure = Arc::new(Mutex::new(Failure::default()));
        let mut state = RestoreState {
            stdout: None,
            stderr: None,
            bytes: Vec::new(),
            restored: false,
            failure: failure.clone(),
        };
        match sys::stdout() {
            Ok(output) => state.stdout = Some(output),
            Err(error) => state.record_error(error),
        }
        match sys::stderr() {
            Ok(output) => state.stderr = Some(output),
            Err(error) => state.record_error(error),
        }
        Self {
            state: Arc::new(Mutex::new(state)),
            tty: sys::stdout_is_tty(),
            input: Arc::new(stdin::Target::new(sender, failure)),
        }
    }

    pub(super) fn register(&self, sender: Arc<dyn Post>) -> Registration {
        install_panic_hook();
        let events = sys::install_events(registry().default_signals.clone());
        if let Err(error) = events {
            self.state
                .lock()
                .unwrap()
                .record_error(io::Error::new(error.kind(), error.to_string()));
        }
        let entry = Arc::new(Entry {
            state: self.state.clone(),
            input: self.input.clone(),
            owner: thread::current().id(),
            sender,
            in_loop: AtomicBool::new(false),
        });
        let mut entries = registry().entries.lock().unwrap();
        if events.is_ok() {
            entries.push(entry.clone());
            registry().default_signals.store(false, Ordering::SeqCst);
        }
        Registration {
            entry,
            suspended: Vec::new(),
        }
    }
}

impl Terminal for ProcessTerminal {
    fn stop_process(&mut self) {
        sys::stop_process();
    }

    fn stdin_is_tty(&self) -> bool {
        sys::stdin_is_tty()
    }

    fn set_raw_mode(&mut self, enabled: bool) {
        if enabled
            && (self
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .restored
                || self.input.failure.lock().unwrap().failed)
        {
            return;
        }
        let result = if enabled {
            stdin::acquire(&self.input)
        } else {
            stdin::release(&self.input)
        };
        if let Err(error) = result {
            self.input.failure.lock().unwrap().record(error);
        }
    }

    fn write(&mut self, bytes: &[u8], restore: &[u8]) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.write(bytes, false);
        if !state.restored {
            state.bytes = restore.to_vec();
        }
    }

    fn write_stderr(&mut self, bytes: &[u8], restore: &[u8]) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.write(bytes, true);
        if !state.restored {
            state.bytes = restore.to_vec();
        }
    }

    fn restore(&mut self, bytes: &[u8]) {
        if let Err(error) = stdin::restore(&self.input, false) {
            self.input.failure.lock().unwrap().record(error);
        }
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .restore(Some(bytes));
    }

    fn take_error(&mut self) -> Option<io::Error> {
        self.input.failure.lock().unwrap().take()
    }

    fn size(&self) -> Option<(u16, u16)> {
        sys::size()
    }
    fn is_tty(&self) -> bool {
        self.tty
    }
}

struct Entry {
    input: Arc<stdin::Target>,
    state: Arc<Mutex<RestoreState>>,
    owner: ThreadId,
    sender: Arc<dyn Post>,
    in_loop: AtomicBool,
}

impl Entry {
    fn restore(&self, emergency: bool) {
        if let Err(error) = stdin::restore(&self.input, emergency) {
            self.input.failure.lock().unwrap().record(error);
        }
        // The panic hook may interrupt an operation holding RestoreState.
        // Do not deadlock on re-entry; raw mode has been released above.
        if emergency && thread::panicking() {
            if let Ok(mut state) = self.state.try_lock() {
                state.restore(None);
            }
        } else {
            self.state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .restore(None);
        }
    }
}

struct Registry {
    entries: Mutex<Vec<Arc<Entry>>>,
    default_signals: Arc<AtomicBool>,
    termination: Mutex<Option<sys::Termination>>,
}

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| Registry {
        entries: Mutex::new(Vec::new()),
        default_signals: Arc::new(AtomicBool::new(true)),
        termination: Mutex::new(None),
    })
}

pub(super) struct Registration {
    entry: Arc<Entry>,
    suspended: Vec<Arc<Entry>>,
}

impl Registration {
    // A callback holding the terminal has no loop: termination signals
    // restore and terminate without waiting for cleanups.
    pub(super) fn leave_loop(&mut self) {
        let _entries = registry().entries.lock().unwrap();
        self.entry.in_loop.store(false, Ordering::SeqCst);
    }

    pub(super) fn rejoin_loop(&mut self) {
        let _entries = registry().entries.lock().unwrap();
        self.entry.in_loop.store(true, Ordering::SeqCst);
    }

    pub(super) fn enter_loop(&mut self) {
        let entries = registry().entries.lock().unwrap();
        // A sync wait blocks every loop on this thread, including concurrent
        // Tokio apps. Their cleanups cannot run until this registration drops.
        self.suspended = entries
            .iter()
            .filter(|entry| entry.owner == self.entry.owner && entry.in_loop.load(Ordering::SeqCst))
            .cloned()
            .collect();
        for entry in &self.suspended {
            entry.in_loop.store(false, Ordering::SeqCst);
        }
        self.entry.in_loop.store(true, Ordering::SeqCst);
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        // Restore even if mounting or a cleanup panicked before App could finish.
        self.entry.restore(false);
        let mut entries = registry().entries.lock().unwrap();
        entries.retain(|entry| !Arc::ptr_eq(entry, &self.entry));
        for entry in &self.suspended {
            entry.in_loop.store(true, Ordering::SeqCst);
        }
        if entries.is_empty() {
            registry().default_signals.store(true, Ordering::SeqCst);
            if let Some(signal) = *registry().termination.lock().unwrap() {
                sys::terminate(signal);
            }
        }
    }
}

fn install_panic_hook() {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let _install = crate::runtime::panic::HOOK_INSTALL
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            // If installed after the core hook, delegate capture to it. If
            // installed before it, caught panics never reach this hook.
            if crate::runtime::panic::capturing() {
                previous(info);
                return;
            }
            // Do not hold the registry while waiting on a terminal: its owner
            // may be in this hook trying to snapshot the same registry.
            let entries = registry().entries.lock().unwrap().clone();
            for entry in entries.iter().rev() {
                if cfg!(panic = "abort") || entry.owner == thread::current().id() {
                    entry.restore(true);
                }
            }
            previous(info);
        }));
    });
}

pub(super) fn resize() {
    let entries = registry().entries.lock().unwrap();
    for entry in entries.iter() {
        entry.sender.post(Message::Resize);
    }
}

#[cfg(unix)]
pub(super) fn stop() {
    let (ack, done) = mpsc::channel::<()>();
    {
        let entries = registry().entries.lock().unwrap();
        for entry in entries.iter() {
            // A callback has handed over; the render-to-wait
            // gap stops without teardown. Neither has a loop.
            if entry.in_loop.load(Ordering::SeqCst) {
                entry.sender.post(Message::Stop(ack.clone()));
            }
        }
    }
    drop(ack);
    while done.recv().is_ok() {}
}

#[cfg(unix)]
pub(super) fn resume() {
    let entries = registry().entries.lock().unwrap();
    for entry in entries.iter() {
        entry.sender.post(Message::Continue);
    }
}

pub(super) fn signal(signal: sys::Termination) {
    let entries = registry().entries.lock().unwrap();
    let signal = *registry().termination.lock().unwrap().get_or_insert(signal);
    // Publish termination before a registration can disappear,
    // then release the registry before taking terminal locks.
    let snapshot = entries.clone();
    drop(entries);
    let entries = snapshot;
    if entries.is_empty()
        || entries
            .iter()
            .any(|entry| !entry.in_loop.load(Ordering::SeqCst))
    {
        // No loop can run cleanups for these apps. Restore every live
        // terminal before the process-wide default signal action.
        for entry in entries.iter().rev() {
            entry.restore(true);
        }
        sys::terminate(signal);
    }
    for entry in entries.iter() {
        entry.sender.post(Message::Signal);
    }
}
