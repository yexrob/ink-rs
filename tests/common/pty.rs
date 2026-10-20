use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{Child, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system};

// Keep one timeout across readiness, input and exit, as in the PTY harness.
const TIMEOUT: Duration = Duration::from_secs(5);

pub fn collect(mut reader: impl Read + Send + 'static) -> mpsc::Receiver<io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if sender.send(Ok(buffer[..n].to_vec())).is_err() {
                        break;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    let _ = sender.send(Err(error));
                    break;
                }
            }
        }
    });
    receiver
}

pub struct Session {
    master: Option<Box<dyn MasterPty + Send>>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    receiver: mpsc::Receiver<io::Result<Vec<u8>>>,
    output: Vec<u8>,
    status: Option<ExitStatus>,
    #[cfg(windows)]
    cursor_reported: bool,
    closed: bool,
    deadline: Instant,
}

impl Session {
    pub fn spawn(
        program: &OsStr,
        args: &[&OsStr],
        env: &[(&str, &str)],
        rows: u16,
        cols: u16,
    ) -> io::Result<Self> {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(io::Error::other)?;
        let mut command = CommandBuilder::new(program);
        command.args(args);
        command.env("TERM", "xterm-256color");
        command.env("RUST_LIB_BACKTRACE", "0");
        command.env_remove("CI");
        command.env_remove("CONTINUOUS_INTEGRATION");
        for &(key, value) in env {
            command.env(key, value);
        }
        let reader = pair.master.try_clone_reader().map_err(io::Error::other)?;
        let writer = pair.master.take_writer().map_err(io::Error::other)?;
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(io::Error::other)?;
        drop(pair.slave);
        Ok(Self {
            master: Some(pair.master),
            writer,
            child,
            receiver: collect(reader),
            output: Vec::new(),
            status: None,
            #[cfg(windows)]
            cursor_reported: false,
            closed: false,
            deadline: Instant::now() + TIMEOUT,
        })
    }

    pub fn output(&self) -> &[u8] {
        &self.output
    }

    pub fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    // Unix sends SIGWINCH; Windows resizes ConPTY and the driver polls its size.
    #[allow(dead_code)]
    pub fn resize(&self, rows: u16, cols: u16) -> io::Result<()> {
        self.master
            .as_ref()
            .ok_or_else(|| io::Error::other("PTY is closed"))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(io::Error::other)
    }

    pub fn is_running(&mut self) -> io::Result<bool> {
        if self.status.is_none() {
            self.status = self.child.try_wait()?;
        }
        // ConPTY's output pipe stays open until ClosePseudoConsole. The reader
        // thread keeps draining while dropping the last console owner blocks.
        // Finish the cursor handshake even if the child has exited.
        #[cfg(windows)]
        if self.status.is_some() && self.cursor_reported {
            self.master.take();
        }
        Ok(self.status.is_none())
    }

    fn read(&mut self) -> Result<(), String> {
        if Instant::now() >= self.deadline {
            return Err(format!(
                "timed out; output: {:?}",
                String::from_utf8_lossy(&self.output)
            ));
        }
        match self.receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(Ok(bytes)) => {
                #[cfg(windows)]
                let start = self.output.len().saturating_sub(3);
                self.output.extend(bytes);
                #[cfg(windows)]
                {
                    // Include the previous suffix so a split ESC[6n is answered.
                    let requests = self.output[start..]
                        .windows(4)
                        .filter(|bytes| *bytes == b"\x1b[6n")
                        .count();
                    for _ in 0..requests {
                        self.send(b"\x1b[1;1R").map_err(|error| error.to_string())?;
                        self.cursor_reported = true;
                    }
                }
            }
            // Linux PTY masters report EIO after their last slave closes.
            Ok(Err(error)) if cfg!(target_os = "linux") && error.raw_os_error() == Some(5) => {
                self.closed = true;
            }
            // Windows pipes can report ERROR_BROKEN_PIPE instead of EOF.
            #[cfg(windows)]
            Ok(Err(error)) if error.raw_os_error() == Some(109) => {
                self.closed = true;
            }
            Ok(Err(error)) => return Err(error.to_string()),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                self.closed = true;
                thread::sleep(Duration::from_millis(10));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        self.is_running().map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn wait_for(&mut self, mut done: impl FnMut(&[u8]) -> bool) -> Result<(), String> {
        loop {
            if self.status.is_some() {
                return Err("child exited before input".into());
            }
            if done(&self.output) {
                return Ok(());
            }
            self.read()?;
        }
    }

    pub fn wait_exit(&mut self) -> Result<ExitStatus, String> {
        loop {
            if self.closed
                && let Some(status) = &self.status
            {
                return Ok(status.clone());
            }
            self.read()?;
        }
    }
}

// What a PTY child printed. A Unix PTY carries the child's bytes; ConPTY
// carries conhost's repaint, which can split text runs, so read its screen
// and history instead.
#[allow(dead_code)]
pub fn printed(output: &[u8], rows: u16, cols: u16) -> String {
    if cfg!(windows) {
        let mut parser = vt100::Parser::new(rows, cols, 1000);
        parser.process(output);
        contents_with_scrollback(&mut parser)
    } else {
        String::from_utf8_lossy(output).into_owned()
    }
}

// Collect each history row once, then the visible screen, without overlapping
// scrollback viewports duplicating markers.
#[allow(dead_code)]
pub fn contents_with_scrollback(parser: &mut vt100::Parser) -> String {
    parser.screen_mut().set_scrollback(usize::MAX);
    let history = parser.screen().scrollback();
    let (_, cols) = parser.screen().size();
    let mut rows = Vec::new();
    for offset in (1..=history).rev() {
        parser.screen_mut().set_scrollback(offset);
        rows.push(parser.screen().rows(0, cols).next().unwrap());
    }
    parser.screen_mut().set_scrollback(0);
    rows.extend(parser.screen().rows(0, cols));
    rows.join("\n")
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.status.is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
