use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use super::process::Failure;
use super::{Message, Post, sys};

pub(super) struct Target {
    pub(super) sender: Arc<dyn Post>,
    pub(super) failure: Arc<Mutex<Failure>>,
    closed: AtomicBool,
}

impl Target {
    pub(super) fn new(sender: Arc<dyn Post>, failure: Arc<Mutex<Failure>>) -> Self {
        Self {
            sender,
            failure,
            closed: AtomicBool::new(false),
        }
    }
}

#[derive(Default)]
pub(super) struct Route {
    pub(super) holders: Vec<Arc<Target>>,
    pub(super) ended: bool,
}

#[derive(Default)]
struct Stdin {
    saved: Option<sys::SavedMode>,
    reader: Option<sys::Reader>,
    route: Arc<Mutex<Route>>,
}

fn stdin() -> &'static Mutex<Stdin> {
    static STDIN: OnceLock<Mutex<Stdin>> = OnceLock::new();
    STDIN.get_or_init(Mutex::default)
}

pub(super) fn acquire(target: &Arc<Target>) -> io::Result<()> {
    let mut state = stdin().lock().unwrap();
    // Restoration and acquisition serialize on the lifecycle lock. A signal
    // may restore an app while its UI thread is between the raw-mode checks.
    if target.closed.load(Ordering::Relaxed) {
        return Ok(());
    }
    if state
        .route
        .lock()
        .unwrap()
        .holders
        .iter()
        .any(|held| Arc::ptr_eq(held, target))
    {
        return Ok(());
    }
    if state.saved.is_none() {
        state.saved = Some(sys::enter_raw_mode()?);
    }
    if state.route.lock().unwrap().ended {
        state.stop_reader()?;
    }
    state.route.lock().unwrap().holders.push(target.clone());
    if state.reader.is_none() {
        if let Err(error) = state.start_reader() {
            state.route.lock().unwrap().holders.pop();
            if state.route.lock().unwrap().holders.is_empty()
                && let Some(saved) = &state.saved
            {
                // Retain the snapshot if rollback fails, for restoration to retry.
                if sys::restore_mode(saved, true).is_ok() {
                    state.saved = None;
                }
            }
            return Err(error);
        }
    } else {
        target.sender.post(Message::InputStarted);
    }
    Ok(())
}

pub(super) fn release(target: &Arc<Target>) -> io::Result<()> {
    release_inner(target, false, false)
}

pub(super) fn restore(target: &Arc<Target>, emergency: bool) -> io::Result<()> {
    release_inner(target, emergency, true)
}

fn release_inner(target: &Arc<Target>, emergency: bool, close: bool) -> io::Result<()> {
    let mut state = stdin().lock().unwrap();
    if close {
        target.closed.store(true, Ordering::Relaxed);
    }
    let empty = {
        let mut route = state.route.lock().unwrap();
        let previous = route.holders.len();
        route.holders.retain(|held| !Arc::ptr_eq(held, target));
        if route.holders.len() != previous
            && route.ended
            && let Some(target) = route.holders.last()
        {
            target.sender.post(Message::InputEnded);
        }
        route.holders.is_empty()
    };
    if !empty {
        return Ok(());
    }
    // Join before restoring canonical mode: otherwise a ready read could wait
    // for a full line. The reader never takes the lifecycle or restore lock.
    let stop = state.stop_reader();
    let restore = if let Some(saved) = &state.saved {
        sys::restore_mode(saved, emergency)
    } else {
        Ok(())
    };
    if restore.is_ok() {
        state.saved = None;
    }
    stop.and(restore)
}

impl Stdin {
    fn start_reader(&mut self) -> io::Result<()> {
        // Hold route until Started is queued, before the reader can send EOF.
        let mut routing = self.route.lock().unwrap();
        let reader = sys::Reader::start(self.route.clone())?;
        routing.ended = false;
        for target in &routing.holders {
            target.sender.post(Message::InputStarted);
        }
        self.reader = Some(reader);
        Ok(())
    }

    fn stop_reader(&mut self) -> io::Result<()> {
        if let Some(reader) = self.reader.take() {
            reader.stop()?;
        }
        Ok(())
    }
}

pub(super) fn input(route: &Mutex<Route>, bytes: &[u8]) {
    let routing = route.lock().unwrap();
    if let Some(target) = routing.holders.last() {
        target.sender.post(Message::Input(bytes.to_vec()));
    }
}

pub(super) fn ended(route: &Mutex<Route>, error: Option<io::Error>) {
    let mut routing = route.lock().unwrap();
    routing.ended = true;
    if let Some(target) = routing.holders.last() {
        if let Some(error) = error {
            target.failure.lock().unwrap().record(error);
        }
        target.sender.post(Message::InputEnded);
    }
}
