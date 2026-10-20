use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, AsRawFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};

use filedescriptor::{FileDescriptor, POLLIN, poll, pollfd, socketpair};
use rustix::termios::{
    ControlModes, InputModes, LocalModes, OptionalActions, OutputModes, SpecialCodeIndex, Termios,
    tcgetattr, tcsetattr,
};

use super::process::Failure;
use super::{Message, Post};

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
struct Route {
    holders: Vec<Arc<Target>>,
    ended: bool,
}

struct Reader {
    wake: FileDescriptor,
    thread: JoinHandle<()>,
}

#[derive(Default)]
struct Stdin {
    saved: Option<Termios>,
    reader: Option<Reader>,
    route: Arc<Mutex<Route>>,
}

fn stdin() -> &'static Mutex<Stdin> {
    static STDIN: OnceLock<Mutex<Stdin>> = OnceLock::new();
    STDIN.get_or_init(Mutex::default)
}

fn fd_error(error: filedescriptor::Error) -> io::Error {
    match error {
        filedescriptor::Error::Io(error) | filedescriptor::Error::Poll(error) => error,
        error => io::Error::other(error),
    }
}

fn set_mode(mode: &Termios, action: OptionalActions) -> io::Result<()> {
    loop {
        match tcsetattr(io::stdin(), action, mode) {
            Err(rustix::io::Errno::INTR) => continue,
            result => return result.map_err(Into::into),
        }
    }
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
        let saved = tcgetattr(io::stdin())?;
        let mut raw = saved.clone();
        // libuv v1.52.1 src/unix/tty.c, uv_tty_set_mode(UV_TTY_MODE_RAW): do not
        // use cfmakeraw, which clears OPOST and changes output newlines.
        raw.input_modes.remove(
            InputModes::BRKINT
                | InputModes::ICRNL
                | InputModes::INPCK
                | InputModes::ISTRIP
                | InputModes::IXON,
        );
        raw.local_modes
            .remove(LocalModes::ECHO | LocalModes::ICANON | LocalModes::IEXTEN | LocalModes::ISIG);
        raw.control_modes.insert(ControlModes::CS8);
        raw.output_modes.insert(OutputModes::ONLCR);
        raw.special_codes[SpecialCodeIndex::VMIN] = 1;
        raw.special_codes[SpecialCodeIndex::VTIME] = 0;
        // uv_tty_set_mode uses TCSADRAIN; uv_tty_reset_mode uses TCSANOW.
        set_mode(&raw, OptionalActions::Drain)?;
        state.saved = Some(saved);
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
                if set_mode(saved, OptionalActions::Now).is_ok() {
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
        set_mode(
            saved,
            if emergency {
                OptionalActions::Now
            } else {
                OptionalActions::Drain
            },
        )
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
        let input = File::from(io::stdin().as_fd().try_clone_to_owned()?);
        let (wake, wake_read) = socketpair().map_err(fd_error)?;
        let route = self.route.clone();
        // Hold route until Started is queued, before the reader can send EOF.
        let mut routing = self.route.lock().unwrap();
        let thread = thread::Builder::new()
            .name("ink-stdin".into())
            .spawn(move || read(input, wake_read, route))?;
        routing.ended = false;
        for target in &routing.holders {
            target.sender.post(Message::InputStarted);
        }
        self.reader = Some(Reader { wake, thread });
        Ok(())
    }

    fn stop_reader(&mut self) -> io::Result<()> {
        if let Some(mut reader) = self.reader.take() {
            // A reader that reached EOF/error has closed its socket already.
            let wake = reader.wake.write_all(&[1]);
            drop(reader.wake);
            let joined = reader.thread.join();
            if joined.is_err() {
                return Err(io::Error::other("stdin reader panicked"));
            }
            if let Err(error) = wake
                && error.kind() != io::ErrorKind::BrokenPipe
            {
                return Err(error);
            }
        }
        Ok(())
    }
}

fn read(mut input: File, wake: FileDescriptor, route: Arc<Mutex<Route>>) {
    let mut fds = [
        pollfd {
            fd: input.as_raw_fd(),
            events: POLLIN,
            revents: 0,
        },
        pollfd {
            fd: wake.as_raw_fd(),
            events: POLLIN,
            revents: 0,
        },
    ];
    let mut bytes = [0; 4096];
    loop {
        let result = poll(&mut fds, None).map_err(fd_error).and_then(|_| {
            if fds[1].revents != 0 {
                return Ok(None);
            }
            input.read(&mut bytes).map(Some)
        });
        match result {
            Ok(None) => return,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Ok(Some(n)) if n > 0 => {
                let routing = route.lock().unwrap();
                if let Some(target) = routing.holders.last() {
                    target.sender.post(Message::Input(bytes[..n].to_vec()));
                }
            }
            result => {
                let mut routing = route.lock().unwrap();
                routing.ended = true;
                if let Some(target) = routing.holders.last() {
                    if let Err(error) = result {
                        target.failure.lock().unwrap().record(error);
                    }
                    target.sender.post(Message::InputEnded);
                }
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn target() -> (Arc<Target>, mpsc::Receiver<Message>) {
        let (sender, receiver) = mpsc::channel();
        (
            Arc::new(Target::new(Arc::new(sender), Arc::default())),
            receiver,
        )
    }

    // Native rules absent from Ink's PTY cases: last-acquired routing,
    // fallback to the previous holder, and EOF delivery without an I/O error.
    #[test]
    fn reader_routes_to_last_holder_and_reports_eof() {
        let (first, first_rx) = target();
        let (second, second_rx) = target();
        let route = Arc::new(Mutex::new(Route {
            holders: vec![first.clone(), second],
            ended: false,
        }));
        let (mut input, source) = socketpair().unwrap();
        let (_wake, wake_read) = socketpair().unwrap();
        let source = source.as_file().unwrap();
        let routing = route.clone();
        let reader = thread::spawn(move || read(source, wake_read, routing));
        input.write_all(b"second").unwrap();
        assert!(
            matches!(second_rx.recv_timeout(Duration::from_secs(2)).unwrap(), Message::Input(bytes) if bytes == b"second")
        );
        assert!(first_rx.try_recv().is_err());
        route.lock().unwrap().holders.pop();
        input.write_all(b"first").unwrap();
        assert!(
            matches!(first_rx.recv_timeout(Duration::from_secs(2)).unwrap(), Message::Input(bytes) if bytes == b"first")
        );
        drop(input);
        assert!(matches!(
            first_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            Message::InputEnded
        ));
        reader.join().unwrap();
        assert!(route.lock().unwrap().ended);
        assert!(first.failure.lock().unwrap().take().is_none());
    }

    // Native rule: a failed read reaches the same first-error slot as writes.
    #[test]
    fn reader_records_io_error_and_keeps_first_failure() {
        let (target, receiver) = target();
        let route = Arc::new(Mutex::new(Route {
            holders: vec![target.clone()],
            ended: false,
        }));
        let (_wake, wake_read) = socketpair().unwrap();
        // A write-only descriptor rejects reads on both supported platforms.
        let source = File::options().write(true).open("/dev/null").unwrap();
        let reader = thread::spawn(move || read(source, wake_read, route));
        assert!(matches!(
            receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
            Message::InputEnded
        ));
        reader.join().unwrap();
        let mut failure = target.failure.lock().unwrap();
        let first = failure.take().expect("read error");
        assert!(first.raw_os_error().is_some());
        failure.record(io::Error::other("later write"));
        assert!(failure.take().is_none());
    }
}
