use std::fs::File;
use std::io::{self, IsTerminal, Read, Write};
use std::os::fd::{AsFd, AsRawFd};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};

use filedescriptor::{FileDescriptor, POLLIN, poll, pollfd, socketpair};
use rustix::termios::{
    ControlModes, InputModes, LocalModes, OptionalActions, OutputModes, SpecialCodeIndex, Termios,
    tcgetattr, tcsetattr,
};
use signal_hook::consts::{SIGCONT, SIGHUP, SIGINT, SIGTERM, SIGTSTP, SIGWINCH};
use signal_hook::iterator::Signals;

use super::super::stdin::Route;
use super::super::{process, stdin};

pub(in crate::driver) type Output = File;
pub(in crate::driver) type SavedMode = Termios;
pub(in crate::driver) type Termination = i32;
pub(in crate::driver) const JOB_CONTROL: bool = true;

// Duplicated fds bypass std::io::Stdout's LineWriter: a synchronized
// operation containing newlines reaches File::write_all as one buffer.
pub(in crate::driver) fn stdout() -> io::Result<Output> {
    io::stdout().as_fd().try_clone_to_owned().map(File::from)
}

pub(in crate::driver) fn stderr() -> io::Result<Output> {
    io::stderr().as_fd().try_clone_to_owned().map(File::from)
}

pub(in crate::driver) fn stdout_is_tty() -> bool {
    io::stdout().is_terminal()
}

pub(in crate::driver) fn stdin_is_tty() -> bool {
    io::stdin().is_terminal()
}

pub(in crate::driver) fn size() -> Option<crate::terminal::TerminalSize> {
    rustix::termios::tcgetwinsize(io::stdout())
        .ok()
        .map(|size| crate::terminal::TerminalSize {
            cells: (size.ws_col, size.ws_row),
            cell_pixels: size
                .ws_xpixel
                .checked_div(size.ws_col)
                .zip(size.ws_ypixel.checked_div(size.ws_row))
                .filter(|&(width, height)| width > 0 && height > 0),
        })
}

pub(in crate::driver) fn enter_raw_mode() -> io::Result<SavedMode> {
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
    Ok(saved)
}

pub(in crate::driver) fn restore_mode(saved: &SavedMode, emergency: bool) -> io::Result<()> {
    set_mode(
        saved,
        if emergency {
            OptionalActions::Now
        } else {
            OptionalActions::Drain
        },
    )
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

pub(in crate::driver) struct Reader {
    wake: FileDescriptor,
    thread: JoinHandle<()>,
}

impl Reader {
    pub(in crate::driver) fn start(route: Arc<Mutex<Route>>) -> io::Result<Self> {
        let input = File::from(io::stdin().as_fd().try_clone_to_owned()?);
        let (wake, wake_read) = socketpair().map_err(fd_error)?;
        let thread = thread::Builder::new()
            .name("ink-stdin".into())
            .spawn(move || read(input, wake_read, route))?;
        Ok(Self { wake, thread })
    }

    pub(in crate::driver) fn stop(mut self) -> io::Result<()> {
        // A reader that reached EOF/error has closed its socket already.
        let wake = self.wake.write_all(&[1]);
        drop(self.wake);
        let joined = self.thread.join();
        if joined.is_err() {
            return Err(io::Error::other("stdin reader panicked"));
        }
        if let Err(error) = wake
            && error.kind() != io::ErrorKind::BrokenPipe
        {
            return Err(error);
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
                stdin::input(&route, &bytes[..n]);
            }
            result => {
                stdin::ended(&route, result.err());
                return;
            }
        }
    }
}

pub(in crate::driver) fn install_events(
    default_signals: Arc<AtomicBool>,
) -> &'static io::Result<()> {
    static SIGNALS: OnceLock<io::Result<()>> = OnceLock::new();
    SIGNALS.get_or_init(|| {
        // signal-hook retains its OS handler after unregister. Leave conditional
        // defaults installed, gated by live apps (Ink signal-exit unsubscribes).
        for signal in [SIGINT, SIGTERM, SIGHUP] {
            signal_hook::flag::register_conditional_default(signal, default_signals.clone())?;
        }
        let mut signals = Signals::new([SIGINT, SIGTERM, SIGHUP, SIGWINCH, SIGTSTP, SIGCONT])?;
        thread::Builder::new()
            .name("ink-signals".into())
            .spawn(move || {
                for signal in signals.forever() {
                    match signal {
                        SIGTSTP => {
                            // Session-leader groups have no shell to continue them.
                            // Default TSTP ignores orphaned groups, but emulation uses
                            // SIGSTOP. A full orphan check needs other processes' parents.
                            if rustix::process::getsid(None).ok()
                                == Some(rustix::process::getpgrp())
                            {
                                continue;
                            }
                            process::stop();
                            let _ = signal_hook::low_level::emulate_default_handler(SIGTSTP);
                        }
                        SIGCONT => process::resume(),
                        SIGWINCH => process::resize(),
                        signal => process::signal(signal),
                    }
                }
            })?;
        Ok(())
    })
}

pub(in crate::driver) fn terminate(signal: Termination) -> ! {
    let _ = signal_hook::low_level::emulate_default_handler(signal);
    // The supported termination signals do not return. Do not resume UI work if
    // the OS rejects the emulation.
    signal_hook::low_level::abort();
}

pub(in crate::driver) fn stop_process() {
    let _ = rustix::process::kill_current_process_group(rustix::process::Signal::TSTP);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::Message;
    use crate::driver::stdin::Target;
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
