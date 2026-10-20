use std::fs::File;
use std::io::{self, IsTerminal, Write};
use std::os::fd::AsFd;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once, OnceLock, mpsc};
use std::thread::{self, ThreadId};

use signal_hook::consts::{SIGCONT, SIGHUP, SIGINT, SIGTERM, SIGTSTP, SIGWINCH};
use signal_hook::iterator::Signals;

use super::{Message, stdin};
use crate::terminal::Terminal;

// Release the registry lock before restoration. Lock order is RestoreState ->
// stdin lifecycle -> input route -> failure; normal output takes RestoreState
// -> failure, and normal mode switches take lifecycle -> route -> failure.
// The hook can enter with RestoreState held, so stdin never takes RestoreState.
// Join holds lifecycle but neither route nor failure; the reader takes route
// -> failure only. No user code runs under these locks.
struct RestoreState {
    stdout: Option<File>,
    stderr: Option<File>,
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
    pub(super) fn new(sender: mpsc::Sender<Message>) -> Self {
        let failure = Arc::new(Mutex::new(Failure::default()));
        let mut state = RestoreState {
            stdout: None,
            stderr: None,
            bytes: Vec::new(),
            restored: false,
            failure: failure.clone(),
        };
        // Duplicated fds bypass std::io::Stdout's LineWriter: a synchronized
        // operation containing newlines reaches File::write_all as one buffer.
        match io::stdout().as_fd().try_clone_to_owned() {
            Ok(fd) => state.stdout = Some(File::from(fd)),
            Err(error) => state.record_error(error),
        }
        match io::stderr().as_fd().try_clone_to_owned() {
            Ok(fd) => state.stderr = Some(File::from(fd)),
            Err(error) => state.record_error(error),
        }
        Self {
            state: Arc::new(Mutex::new(state)),
            tty: io::stdout().is_terminal(),
            input: Arc::new(stdin::Target::new(sender, failure)),
        }
    }

    pub(super) fn register(&self, sender: mpsc::Sender<Message>) -> Registration {
        install_panic_hook();
        let signals = install_signals();
        if let Err(error) = signals {
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
        if signals.is_ok() {
            entries.push(entry.clone());
            registry().default_signals.store(false, Ordering::SeqCst);
        }
        Registration {
            entry,
            suspended: None,
        }
    }
}

impl Terminal for ProcessTerminal {
    fn stop_process(&mut self) {
        let _ = rustix::process::kill_current_process_group(rustix::process::Signal::TSTP);
    }

    fn stdin_is_tty(&self) -> bool {
        io::stdin().is_terminal()
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
        rustix::termios::tcgetwinsize(io::stdout())
            .ok()
            .map(|size| (size.ws_col, size.ws_row))
    }
    fn is_tty(&self) -> bool {
        self.tty
    }
}

struct Entry {
    input: Arc<stdin::Target>,
    state: Arc<Mutex<RestoreState>>,
    owner: ThreadId,
    sender: mpsc::Sender<Message>,
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
    termination: Mutex<Option<i32>>,
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
    suspended: Option<Arc<Entry>>,
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
        // A nested wait suspends the caller's loop. Treat it as outside the
        // loop rather than waiting for cleanups that cannot run on this thread.
        self.suspended = entries
            .iter()
            .find(|entry| entry.owner == self.entry.owner && entry.in_loop.load(Ordering::SeqCst))
            .cloned();
        if let Some(entry) = &self.suspended {
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
        if let Some(entry) = &self.suspended {
            entry.in_loop.store(true, Ordering::SeqCst);
        }
        if entries.is_empty() {
            registry().default_signals.store(true, Ordering::SeqCst);
            if let Some(signal) = *registry().termination.lock().unwrap() {
                terminate(signal);
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

fn install_signals() -> &'static io::Result<()> {
    static SIGNALS: OnceLock<io::Result<()>> = OnceLock::new();
    SIGNALS.get_or_init(|| {
        // signal-hook retains its OS handler after unregister. Leave conditional
        // defaults installed, gated by live apps (Ink signal-exit unsubscribes).
        for signal in [SIGINT, SIGTERM, SIGHUP] {
            signal_hook::flag::register_conditional_default(
                signal,
                registry().default_signals.clone(),
            )?;
        }
        let mut signals = Signals::new([SIGINT, SIGTERM, SIGHUP, SIGWINCH, SIGTSTP, SIGCONT])?;
        thread::Builder::new()
            .name("ink-signals".into())
            .spawn(move || {
                for signal in signals.forever() {
                    if signal == SIGTSTP {
                        // Session-leader groups have no shell to continue them.
                        // Default TSTP ignores orphaned groups, but emulation uses
                        // SIGSTOP. A full orphan check needs other processes' parents.
                        if rustix::process::getsid(None).ok() == Some(rustix::process::getpgrp()) {
                            continue;
                        }
                        let (ack, done) = mpsc::channel::<()>();
                        {
                            let entries = registry().entries.lock().unwrap();
                            for entry in entries.iter() {
                                // A callback has handed over; the render-to-wait
                                // gap stops without teardown. Neither has a loop.
                                if entry.in_loop.load(Ordering::SeqCst) {
                                    let _ = entry.sender.send(Message::Stop(ack.clone()));
                                }
                            }
                        }
                        drop(ack);
                        while done.recv().is_ok() {}
                        let _ = signal_hook::low_level::emulate_default_handler(SIGTSTP);
                        continue;
                    }
                    let entries = registry().entries.lock().unwrap();
                    if signal == SIGCONT {
                        for entry in entries.iter() {
                            let _ = entry.sender.send(Message::Continue);
                        }
                        continue;
                    }
                    if signal == SIGWINCH {
                        for entry in entries.iter() {
                            let _ = entry.sender.send(Message::Resize);
                        }
                        continue;
                    }
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
                        terminate(signal);
                    }
                    for entry in entries.iter() {
                        let _ = entry.sender.send(Message::Signal);
                    }
                }
            })?;
        Ok(())
    })
}

pub(super) fn terminate(signal: i32) -> ! {
    let _ = signal_hook::low_level::emulate_default_handler(signal);
    // The supported termination signals do not return. Do not resume UI work if
    // the OS rejects the emulation.
    signal_hook::low_level::abort();
}
