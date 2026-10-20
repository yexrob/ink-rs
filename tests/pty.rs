//! Ink test/exit.tsx, test/hooks.tsx and test/render.tsx @ 26d2c3f.
//! One executable serves as harness and fixture so Cargo builds both together.
use std::io::{self, Read, Write};
#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ink_rs::{AnimationOptions, Element, Error, Options, Scope, Text, component, render};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use rustix::termios::{Termios, tcgetattr};

const FIXTURE: &str = "INK_RS_PTY_FIXTURE";
const READY: &str = "__READY__";
const TIMEOUT: Duration = Duration::from_secs(5);

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, PartialEq)]
enum Case {
    Normal,
    Unmount,
    Finish,
    Exit,
    Error,
    Value,
    Object,
    Raw,
    RawError,
    RawUnmount,
    Alive,
    CtrlC,
    Pipe,
    AnimationPipe,
    AnimationNonInteractive,
}

const CASES: &[(Case, &str, &str)] = &[
    (
        Case::Normal,
        "normal",
        "exit normally without unmount() or exit()",
    ),
    (Case::Unmount, "unmount", "exit on unmount()"),
    (Case::Finish, "finish", "exit when app finishes execution"),
    (Case::Exit, "exit", "exit on exit()"),
    (Case::Error, "error", "exit on exit() with error"),
    (Case::Value, "value", "exit on exit() with result value"),
    (Case::Object, "object", "exit on exit() with object result"),
    (Case::Raw, "raw", "exit on exit() with raw mode"),
    (
        Case::RawError,
        "raw-error",
        "exit on exit() with raw mode with error",
    ),
    (
        Case::RawUnmount,
        "raw-unmount",
        "exit on unmount() with raw mode",
    ),
    (Case::Alive, "alive", "don’t exit while raw mode is active"),
    (
        Case::CtrlC,
        "ctrl-c",
        "useInput - handle Ctrl+C when `exitOnCtrlC` is `false`",
    ),
    (
        Case::Pipe,
        "pipe",
        "#725: non-TTY child process output is flushed",
    ),
    (
        Case::AnimationPipe,
        "use-animation-non-interactive-exit",
        "useAnimation can drive non-interactive process exit",
    ),
    (
        Case::AnimationNonInteractive,
        "use-animation-interactive-false-exit",
        "useAnimation can drive explicitly non-interactive process exit",
    ),
];

fn animation_exit(cx: &mut Scope, _: &()) -> Element {
    let animation = cx.use_animation(AnimationOptions {
        interval: Duration::from_millis(8),
        ..AnimationOptions::default()
    });
    let handle = cx.use_app();
    cx.use_effect(animation.frame, move || {
        if animation.frame >= 3 {
            handle.exit();
        }
    });
    Text::new(animation.frame.to_string()).into()
}

struct Value {
    message: &'static str,
}

fn example(cx: &mut Scope, case: &Case) -> Element {
    let case = *case;
    let count = cx.use_state(|| 0);
    let active = cx.use_state(|| true);
    let quit = cx.use_state(|| false);
    let raw = matches!(
        case,
        Case::Raw | Case::RawError | Case::RawUnmount | Case::Alive | Case::CtrlC
    );
    // No captured AppHandle: E13 must be kept alive by raw mode, not a handle.
    cx.use_input(raw && active.get(), move |input, key| {
        if case == Case::CtrlC {
            assert!(input == "c" && key.ctrl, "unexpected input");
            quit.set(true);
        } else if input == "q" {
            quit.set(true);
        }
    });
    let exit = cx.use_app();
    let quitting = quit.get();
    cx.use_effect(quitting, move || {
        if quitting {
            exit.exit();
        }
    });
    let handle = cx.use_app();
    let writer = cx.use_stdout();
    cx.use_effect((), move || {
        if case == Case::Alive {
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(500));
                handle.update(move || {
                    active.set(false);
                    active.set(true);
                    writer.write(READY);
                });
            });
        } else if matches!(
            case,
            Case::Unmount | Case::Finish | Case::Exit | Case::Error
        ) {
            thread::spawn(move || {
                let delay = if case == Case::Finish { 20 } else { 100 };
                for n in 1..=5 {
                    thread::sleep(Duration::from_millis(delay));
                    handle.update(move || count.set(n));
                }
                match case {
                    Case::Exit => handle.exit(),
                    Case::Error => handle.exit_with_error("errored"),
                    _ => {}
                }
            });
        } else if matches!(
            case,
            Case::Raw | Case::RawError | Case::Value | Case::Object
        ) {
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(500));
                match case {
                    Case::RawError => handle.exit_with_error("errored"),
                    Case::Value => handle.exit_with("hello from ink"),
                    Case::Object => handle.exit_with(Value {
                        message: "hello from ink object",
                    }),
                    _ => handle.exit(),
                }
            });
        }
    });
    Text::new(format!("Counter: {}", count.get())).into()
}

// rustix Termios does not implement PartialEq; compare its semantic fields,
// not struct padding. SpecialCodes exposes every slot through Debug.
// These are the fields exposed on Linux and macOS.
fn same_termios(a: &Termios, b: &Termios) -> bool {
    #[cfg(target_os = "linux")]
    if a.line_discipline != b.line_discipline {
        return false;
    }
    a.input_modes == b.input_modes
        && a.output_modes == b.output_modes
        && a.control_modes == b.control_modes
        && a.local_modes == b.local_modes
        && format!("{:?}", a.special_codes) == format!("{:?}", b.special_codes)
        && a.input_speed() == b.input_speed()
        && a.output_speed() == b.output_speed()
}

fn stdin_termios() -> Result<Termios> {
    // XNU bsd/kern/tty.c: ttioctl_locked sets PENDIN on an ICANON
    // transition and preserves it across tcsetattr. ttyselect -> ttnread
    // settles that pending reprocessing. Observe readiness without reading,
    // waiting, changing modes, or masking any flag in the equality check.
    #[cfg(target_os = "macos")]
    filedescriptor::poll(
        &mut [filedescriptor::pollfd {
            fd: io::stdin().as_raw_fd(),
            events: filedescriptor::POLLIN,
            revents: 0,
        }],
        Some(Duration::ZERO),
    )?;
    Ok(tcgetattr(io::stdin())?)
}

fn fixture(case: Case) -> Result {
    let before = (case == Case::Raw).then(stdin_termios).transpose()?;
    let element = match case {
        Case::Normal => Text::new("Hello World").into(),
        Case::Pipe => Text::new("ready-stdin-not-tty").into(),
        Case::AnimationPipe | Case::AnimationNonInteractive => component(animation_exit, ()),
        _ => component(example, case),
    };
    let app = render(
        element,
        Options {
            exit_on_ctrl_c: case != Case::CtrlC,
            interactive: (case == Case::AnimationNonInteractive).then_some(false),
            ..Options::default()
        },
    );
    if matches!(case, Case::Unmount | Case::RawUnmount) {
        let handle = app.handle();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(500));
            handle.exit();
        });
    }
    if case == Case::CtrlC {
        // render has completed the raw-mode acquire before input is allowed.
        print!("{READY}");
        io::stdout().flush()?;
    }
    match app.wait_until_exit() {
        Err(Error::App(error)) if matches!(case, Case::Error | Case::RawError) => {
            println!("{error}")
        }
        Err(error) => return Err(error.into()),
        Ok(Some(value)) if case == Case::Value => println!(
            "result:{}",
            value.downcast::<&str>().map_err(|_| "wrong value type")?
        ),
        Ok(Some(value)) if case == Case::Object => println!(
            "result:{}",
            value
                .downcast::<Value>()
                .map_err(|_| "wrong object type")?
                .message
        ),
        Ok(None)
            if !matches!(
                case,
                Case::Value | Case::Object | Case::Error | Case::RawError
            ) => {}
        _ => return Err("wrong exit result".into()),
    }
    if let Some(before) = before {
        let after = stdin_termios()?;
        if !same_termios(&before, &after) {
            return Err(
                format!("termios was not restored: before={before:?} after={after:?}").into(),
            );
        }
        println!("termios-restored");
    }
    println!("exited");
    Ok(())
}

fn collect(mut reader: impl Read + Send + 'static) -> mpsc::Receiver<io::Result<Vec<u8>>> {
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

fn assert_output(case: Case, output: &[u8]) -> Result {
    let output = String::from_utf8_lossy(output);
    let marker = match case {
        Case::Error | Case::RawError => "errored",
        Case::Value => "result:hello from ink",
        Case::Object => "result:hello from ink object",
        Case::Pipe => "ready-stdin-not-tty",
        _ => "exited",
    };
    if !output.contains(marker) || !output.contains("exited") {
        return Err(format!("missing {marker:?} or exited in {output:?}").into());
    }
    if case == Case::Raw && !output.contains("termios-restored") {
        return Err("missing termios restoration marker".into());
    }
    Ok(())
}

fn run_pty(case: Case, id: &str) -> Result {
    let pair = native_pty_system().openpty(PtySize {
        rows: 24,
        cols: 100,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    let mut command = CommandBuilder::new(std::env::current_exe()?);
    command.env(FIXTURE, id);
    command.env("TERM", "xterm-256color");
    command.env_remove("CI");
    command.env_remove("CONTINUOUS_INTEGRATION");
    let reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;
    let mut child = pair.slave.spawn_command(command)?;
    drop(pair.slave);
    let receiver = collect(reader);
    let result = (|| -> Result {
        let deadline = Instant::now() + TIMEOUT;
        let mut output = Vec::new();
        let mut ready_at = None;
        let mut sent = false;
        let mut status = None;
        let mut closed = false;
        loop {
            if Instant::now() >= deadline {
                return Err(
                    format!("timed out; output: {:?}", String::from_utf8_lossy(&output)).into(),
                );
            }
            match receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(Ok(bytes)) => output.extend(bytes),
                // Linux PTY masters report EIO after their last slave closes.
                Ok(Err(error)) if cfg!(target_os = "linux") && error.raw_os_error() == Some(5) => {
                    closed = true
                }
                Ok(Err(error)) => return Err(error.into()),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    closed = true;
                    thread::sleep(Duration::from_millis(10));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if status.is_none() {
                status = child.try_wait()?;
            }
            if matches!(case, Case::Alive | Case::CtrlC) && !sent {
                if ready_at.is_none() && String::from_utf8_lossy(&output).contains(READY) {
                    ready_at = Some(Instant::now());
                }
                if status.is_some() {
                    return Err("child exited before input".into());
                }
                if let Some(ready) = ready_at
                    && (case == Case::CtrlC || ready.elapsed() >= Duration::from_millis(500))
                {
                    writer.write_all(if case == Case::CtrlC { b"\x03" } else { b"q" })?;
                    writer.flush()?;
                    sent = true;
                }
            }
            if closed && let Some(status) = &status {
                if !status.success() {
                    return Err(format!(
                        "child status {status:?}; output: {:?}",
                        String::from_utf8_lossy(&output)
                    )
                    .into());
                }
                return assert_output(case, &output);
            }
        }
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn run_pipe(case: Case, id: &str) -> Result {
    let mut child = Command::new(std::env::current_exe()?)
        .env(FIXTURE, id)
        .env_remove("CI")
        .env_remove("CONTINUOUS_INTEGRATION")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = collect(child.stdout.take().unwrap());
    let stderr = collect(child.stderr.take().unwrap());
    let deadline = Instant::now() + TIMEOUT;
    let result = (|| -> Result {
        while child.try_wait()?.is_none() {
            if Instant::now() >= deadline {
                return Err("non-TTY child timed out".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
        let mut output = Vec::new();
        for (receiver, is_stdout) in [(&stdout, true), (&stderr, false)] {
            loop {
                match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                    Ok(bytes) => output.extend(bytes?),
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        return Err("non-TTY output timed out".into());
                    }
                }
            }
            if is_stdout {
                assert_output(case, &output)?;
            }
        }
        if !child.wait()?.success() {
            return Err(format!(
                "non-TTY child failed: {:?}",
                String::from_utf8_lossy(&output)
            )
            .into());
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn main() {
    if let Ok(id) = std::env::var(FIXTURE) {
        let result = CASES
            .iter()
            .find(|(_, key, _)| *key == id)
            .ok_or_else(|| "unknown fixture".into())
            .and_then(|(case, _, _)| fixture(*case));
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    let mut failed = 0;
    for &(case, id, name) in CASES {
        let result = if matches!(
            case,
            Case::Pipe | Case::AnimationPipe | Case::AnimationNonInteractive
        ) {
            run_pipe(case, id)
        } else {
            run_pty(case, id)
        };
        match result {
            Ok(()) => println!("ok - {name}"),
            Err(error) => {
                failed += 1;
                eprintln!("FAILED - {name}: {error}");
            }
        }
    }
    if failed > 0 {
        std::process::exit(1);
    }
}
