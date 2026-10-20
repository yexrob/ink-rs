//! Ink 8.0.0 test/exit.tsx, test/hooks.tsx, test/render.tsx and
//! test/suspend-terminal.tsx, commit 26d2c3f.
//! One executable serves as harness and fixture so Cargo builds both together.
use std::ffi::OsStr;
use std::io::{self, Write};
#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ink_rs::{
    AnimationOptions, Element, Error, FlexDirection, Options, Scope, Styled, Text, View, component,
    render,
};
#[path = "common/pty.rs"]
mod session;
#[cfg(unix)]
use rustix::termios::{Termios as InputMode, tcgetattr};
use session::{Session, collect, contents_with_scrollback, printed};
#[cfg(windows)]
use windows_sys::Win32::System::Console::{CONSOLE_MODE as InputMode, GetConsoleMode};

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
    Thrown,
    ThrownCoreFirst,
    NestedPanic,
    Alive,
    CtrlC,
    Pipe,
    AnimationPipe,
    AnimationNonInteractive,
    Suspend,
    #[cfg(unix)]
    JobControl,
    #[cfg(unix)]
    Orphan,
    FullHeightRerender,
    #[cfg(feature = "tokio")]
    TokioFinish,
    #[cfg(feature = "tokio")]
    TokioSuspend,
}

const CASES: &[(Case, &str, &str)] = &[
    #[cfg(feature = "tokio")]
    (
        Case::TokioFinish,
        "tokio-finish",
        "tokio driver ends after its last use_future task",
    ),
    #[cfg(feature = "tokio")]
    (
        Case::TokioSuspend,
        "tokio-suspend",
        "tokio driver hands the terminal over while tasks keep running",
    ),
    (
        Case::Suspend,
        "suspend",
        "suspendTerminal hands the terminal to a child process, then redraws (PTY)",
    ),
    #[cfg(unix)]
    (
        Case::JobControl,
        "job-control",
        "Ctrl+Z hands the terminal to the shell and fg redraws",
    ),
    #[cfg(unix)]
    (
        Case::Orphan,
        "orphan",
        "Ctrl+Z ignores a session-leader process group",
    ),
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
    (
        Case::Thrown,
        "exit-with-thrown-error",
        "exit with thrown error",
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
    (
        Case::ThrownCoreFirst,
        "panic-core-hook-first",
        "caught panic with core hook installed first",
    ),
    (
        Case::NestedPanic,
        "panic-during-unwind",
        "destructor panic restores before abort",
    ),
    (
        Case::FullHeightRerender,
        "issue-969-windows-full-height-rerender",
        "#969: full-height rerenders on Windows should clear terminal between frames; ink-rs keeps one final frame and no stale frames in screen or scrollback",
    ),
];

#[cfg(feature = "tokio")]
fn future_finish(cx: &mut Scope, _: &()) -> Element {
    let label = cx.use_state(|| "waiting");
    cx.use_future((), move || async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        label.set("task done");
    });
    Text::new(label.get()).into()
}

#[cfg(feature = "tokio")]
fn future_suspend(cx: &mut Scope, _: &()) -> Element {
    cx.use_input(true, |_, _| {});
    let handle = cx.use_app();
    cx.use_future((), move || async move {
        let (send, receive) = mpsc::channel();
        let (returned, done) = tokio::sync::oneshot::channel();
        let exit = handle.clone();
        tokio::task::spawn_local(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            send.send(()).unwrap();
            done.await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
            exit.exit();
        });
        handle
            .suspend_terminal(move || {
                print!("CHILD_OUTPUT");
                io::stdout().flush().unwrap();
                receive.recv().unwrap();
                returned.send(()).unwrap();
            })
            .unwrap();
    });
    Text::new("Ink frame").into()
}

fn suspended_child(cx: &mut Scope, _: &()) -> Element {
    cx.use_input(true, |_, _| {});
    let handle = cx.use_app();
    cx.use_effect((), move || {
        let exit = handle.clone();
        handle
            .suspend_terminal(move || {
                print!("CHILD_OUTPUT");
                io::stdout().flush().unwrap();
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(100));
                    exit.exit();
                });
            })
            .unwrap();
    });
    Text::new("Ink frame").into()
}

#[cfg(unix)]
fn job_control(cx: &mut Scope, _: &()) -> Element {
    let quit = cx.use_state(|| false);
    cx.use_input(true, move |input, _| {
        if input == "q" {
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
        // Deliver readiness after the mounting step has acquired stdin.
        handle.update(move || writer.write(READY));
    });
    Text::new("job frame").into()
}

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

// Ink issue-969-windows-full-height-rerender / issue-450-fixture-helpers.
fn full_height_rerender(cx: &mut Scope, _: &()) -> Element {
    let size = cx.use_window_size();
    let frame = cx.use_state(|| 0);
    let count = frame.get();
    let handle = cx.use_app();
    cx.use_effect(count, move || {
        if count == 8 {
            handle.exit();
        } else {
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                handle.update(move || frame.set(count + 1));
            });
        }
    });
    View::new()
        .height(size.rows)
        .flex_direction(FlexDirection::Column)
        .child(Text::new("#450 top"))
        .child(
            View::new()
                .flex_grow(1.0)
                .child(Text::new(format!("frame {count}"))),
        )
        // Change the padding each frame so cell diffs write the bottom-right cell.
        .child(Text::new(format!(
            "#450 bottom{}",
            count
                .to_string()
                .repeat(usize::from(size.columns).saturating_sub("#450 bottom".len()))
        )))
        .into()
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
#[cfg(unix)]
fn same_input_mode(a: &InputMode, b: &InputMode) -> bool {
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

#[cfg(unix)]
fn stdin_mode() -> Result<InputMode> {
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

#[cfg(windows)]
fn same_input_mode(a: &InputMode, b: &InputMode) -> bool {
    a == b
}

#[cfg(windows)]
fn stdin_mode() -> Result<InputMode> {
    let stdin = io::stdin();
    let mut mode = 0;
    // SAFETY: stdin keeps its borrowed handle alive for the call, and mode is
    // a valid writable CONSOLE_MODE. GetConsoleMode does not retain either.
    if unsafe { GetConsoleMode(stdin.as_raw_handle(), &mut mode) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(mode)
}

fn nested_panic(cx: &mut Scope, fail: &bool) -> Element {
    struct BrokenDrop;
    impl Drop for BrokenDrop {
        fn drop(&mut self) {
            panic!("nested destructor failure");
        }
    }
    cx.use_input(true, |_, _| {});
    if *fail {
        let _cleanup = BrokenDrop;
        panic!("outer component failure");
    }
    Text::new("raw mode active").into()
}

fn fixture(case: Case) -> Result {
    #[cfg(feature = "tokio")]
    if matches!(case, Case::TokioFinish | Case::TokioSuspend) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build_local(Default::default())?;
        runtime.block_on(async {
            let element = if case == Case::TokioFinish {
                component(future_finish, ())
            } else {
                component(future_suspend, ())
            };
            ink_rs::tokio::render(element, Options::default())
                .wait_until_exit()
                .await
        })?;
        println!("exited");
        return Ok(());
    }
    if case == Case::ThrownCoreFirst {
        let terminal = ink_rs::testing::TestTerminal::new(100, 24);
        ink_rs::testing::mount(
            Text::new("install core hook"),
            Options::default(),
            &terminal,
        )
        .unmount();
    }
    let before = matches!(case, Case::Raw | Case::Thrown | Case::ThrownCoreFirst)
        .then(stdin_mode)
        .transpose()?;
    let element = match case {
        Case::Normal => Text::new("Hello World").into(),
        Case::Thrown | Case::ThrownCoreFirst => component(|_, _: &()| panic!("errored"), ()),
        Case::NestedPanic => component(nested_panic, false),
        Case::Pipe => Text::new("ready-stdin-not-tty").into(),
        Case::AnimationPipe | Case::AnimationNonInteractive => component(animation_exit, ()),
        Case::Suspend => component(suspended_child, ()),
        #[cfg(unix)]
        Case::JobControl | Case::Orphan => component(job_control, ()),
        Case::FullHeightRerender => component(full_height_rerender, ()),
        _ => component(example, case),
    };
    let mut app = render(
        element,
        Options {
            exit_on_ctrl_c: case != Case::CtrlC,
            interactive: (case == Case::AnimationNonInteractive).then_some(false),
            ..Options::default()
        },
    );
    if case == Case::NestedPanic {
        app.rerender(component(nested_panic, true));
        return Err("nested panic did not abort".into());
    }
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
        Err(Error::Panic(error)) if matches!(case, Case::Thrown | Case::ThrownCoreFirst) => {
            println!("{error}")
        }
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
        let after = stdin_mode()?;
        if !same_input_mode(&before, &after) {
            return Err(
                format!("input mode was not restored: before={before:?} after={after:?}").into(),
            );
        }
        println!("input-mode-restored");
    }
    println!("exited");
    Ok(())
}

// printed is what the child wrote: the bytes of a pipe or a Unix PTY, or
// conhost's screen and history under ConPTY (session::printed).
fn assert_output(case: Case, output: &[u8], printed: &str) -> Result {
    let mut parser = vt100::Parser::new(24, 100, 1000);
    parser.process(output);
    let output = String::from_utf8_lossy(output);
    let handover = case == Case::Suspend;
    #[cfg(feature = "tokio")]
    let handover = handover || case == Case::TokioSuspend;
    if handover && cfg!(windows) {
        // conhost repaints the screen and does not forward ink-rs's cursor sequences.
        if !printed.contains("CHILD_OUTPUT")
            || !printed.contains("Ink frame")
            || parser.screen().hide_cursor()
        {
            return Err(
                format!("missing handover, redraw or cursor restoration: {output:?}").into(),
            );
        }
    } else if handover {
        let Some((_, after)) = output.rsplit_once("CHILD_OUTPUT") else {
            return Err(format!("missing child output: {output:?}").into());
        };
        let mut screen = vt100::Parser::new(24, 100, 0);
        screen.process(after.as_bytes());
        if !output.contains("\x1b[?25h")
            || !after.contains("\x1b[?25l")
            || !screen.screen().contents().contains("Ink frame")
        {
            return Err(format!("missing handover or redraw: {output:?}").into());
        }
    }
    let marker = match case {
        #[cfg(feature = "tokio")]
        Case::TokioFinish => "task done",
        Case::Error | Case::RawError | Case::Thrown | Case::ThrownCoreFirst => "errored",
        Case::Value => "result:hello from ink",
        Case::Object => "result:hello from ink object",
        Case::Pipe => "ready-stdin-not-tty",
        _ => "exited",
    };
    if !printed.contains(marker) || !printed.contains("exited") {
        return Err(format!("missing {marker:?} or exited in {output:?}").into());
    }
    if matches!(case, Case::Raw | Case::Thrown | Case::ThrownCoreFirst)
        && !printed.contains("input-mode-restored")
    {
        return Err("missing input mode restoration marker".into());
    }
    if matches!(case, Case::Thrown | Case::ThrownCoreFirst) {
        let shown = if cfg!(windows) {
            printed.to_owned()
        } else {
            parser.screen().contents()
        };
        if !shown.contains("ERROR") || parser.screen().hide_cursor() {
            return Err(format!("missing overview or cursor restoration: {output:?}").into());
        }
        if printed.contains("panicked at") {
            return Err("std panic output escaped the boundary".into());
        }
    }
    Ok(())
}

fn run_pty(case: Case, id: &str) -> Result {
    #[cfg(unix)]
    if case == Case::JobControl
        && !std::env::var_os("PATH")
            .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join("bash").is_file()))
    {
        println!("skip - job control: bash is not on PATH");
        return Ok(());
    }
    let executable = std::env::current_exe()?;
    #[cfg(unix)]
    let args = [
        OsStr::new("--norc"),
        OsStr::new("--noprofile"),
        OsStr::new("-i"),
        OsStr::new("-c"),
        OsStr::new("\"$0\"; fg"),
        executable.as_os_str(),
    ];
    #[cfg(unix)]
    let (program, args): (&OsStr, &[&OsStr]) = if case == Case::JobControl {
        (OsStr::new("bash"), &args)
    } else {
        (executable.as_os_str(), &[])
    };
    #[cfg(windows)]
    let (program, args): (&OsStr, &[&OsStr]) = (executable.as_os_str(), &[]);
    let (rows, cols) = if case == Case::FullHeightRerender {
        (6, 40)
    } else {
        (24, 100)
    };
    let mut session = Session::spawn(program, args, &[(FIXTURE, id)], rows, cols)?;
    #[cfg(unix)]
    if matches!(case, Case::JobControl | Case::Orphan) {
        session.wait_for(|output| String::from_utf8_lossy(output).contains(READY))?;
        let start = session.output().len();
        session.send(b"\x1a")?;
        if case == Case::JobControl {
            session.wait_for(|output| {
                let output = String::from_utf8_lossy(&output[start..]);
                output.split_once("\x1b[?25h").is_some_and(|(_, resumed)| {
                    resumed.contains("job frame") && resumed.contains("\x1b[?25l")
                })
            })?;
        } else {
            thread::sleep(Duration::from_millis(300));
        }
        session.send(b"q")?;
    }
    if matches!(case, Case::Alive | Case::CtrlC) {
        let mut ready_at = None;
        session.wait_for(|output| {
            if ready_at.is_none() && printed(output, rows, cols).contains(READY) {
                ready_at = Some(Instant::now());
            }
            ready_at.is_some_and(|ready| {
                case == Case::CtrlC || ready.elapsed() >= Duration::from_millis(500)
            })
        })?;
        session.send(if case == Case::CtrlC { b"\x03" } else { b"q" })?;
    }
    let status = session.wait_exit()?;
    let output = session.output();
    let printed = printed(output, rows, cols);
    if case == Case::NestedPanic {
        let mut parser = vt100::Parser::new(24, 100, 0);
        parser.process(output);
        if status.success()
            || !printed.contains("nested destructor failure")
            || parser.screen().hide_cursor()
        {
            return Err(format!(
                "nested panic failed to restore and abort: {:?}",
                String::from_utf8_lossy(output)
            )
            .into());
        }
        return Ok(());
    }
    if !status.success() {
        return Err(format!(
            "child status {status:?}; output: {:?}",
            String::from_utf8_lossy(output)
        )
        .into());
    }
    if case == Case::FullHeightRerender {
        let mut parser = vt100::Parser::new(rows, cols, 1000);
        parser.process(output);
        let contents = contents_with_scrollback(&mut parser);
        let bottom = format!(
            "#450 bottom{}",
            "8".repeat(usize::from(cols) - "#450 bottom".len())
        );
        if contents.matches("frame 8").count() != 1
            || contents.matches("#450 top").count() != 1
            || contents.matches("#450 bottom").count() != 1
            || contents.matches(&bottom).count() != 1
            || (0..8).any(|frame| contents.contains(&format!("frame {frame}")))
        {
            return Err(format!("stale or missing full-height frame: {contents:?}").into());
        }
        return Ok(());
    }
    assert_output(case, output, &printed)
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
                assert_output(case, &output, &String::from_utf8_lossy(&output))?;
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
