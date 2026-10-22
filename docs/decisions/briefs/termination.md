# Task: a process-level setting that leaves termination signals to the application

## Goal

bingo, a downstream coding agent, runs ink-rs inside a tokio process that owns SIGINT, SIGTERM and SIGHUP. On SIGTERM or SIGHUP it ends the running turn, runs its hooks, flushes its session store and unmounts the app before it exits. Today ink-rs installs process-wide handlers on the first render. On a termination signal it restores the terminal and ends the process with the default action, so the host's cleanup never runs. When no app is live, the conditional defaults stay installed and the host's own `tokio::signal` handlers lose to them.

Add a process-level setting that leaves these signals to the application. The requirement from bingo is `<repo>/docs/decisions/briefs/bingo-signal-ownership.md`; this brief changes its shape (a process-level function, not an `Options` field) for the reason below.

## Read first

- `<repo>/CLAUDE.md`.
- `<repo>/docs/decisions/072-termination.md`: the decision and the reason for a process-level function.
- `<repo>/docs/design.md`: the parts on signals, suspend and terminal restore.
- `<repo>/src/driver/process.rs`: `register`, `Registry`, `Registration` and its `Drop`, `signal`, `registry().default_signals`, `install_panic_hook`.
- `<repo>/src/driver/sys/unix.rs`: `install_events` and `terminate`.
- `<repo>/src/driver/sys/windows.rs`: `control_handler`, `install_events` and `terminate`.
- `<repo>/src/driver/mod.rs` (`mount`, `Instance::unmount`, `wait_until_exit`) and `<repo>/src/driver/tokio.rs`.
- `<repo>/src/lib.rs`: the public exports and `Error`.
- `<repo>/tests/pty.rs`: the custom `main` that runs each case as a child process in a PTY.

## Why a process-level function

Signal handlers belong to the process and cannot be removed. One run can mount several apps: a picker, then the main screen; or a library that calls `render` itself. With a per-`Options` value, a library mount with the default installs the handlers first, and the host's setting comes too late. The host calls the function once at the top of `main`, and every mount in the process follows it.

## Change

The API below; better names are welcome, say why in the report.

```rust
/// Who handles the termination signals: SIGINT, SIGTERM and SIGHUP on Unix,
/// the console control events on Windows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Termination {
    /// ink-rs restores the terminal and ends the process. Today's behaviour.
    #[default]
    Handled,
    /// ink-rs installs nothing for these signals; the application handles them
    /// and ends its apps with `Instance::unmount` or `AppHandle::exit`.
    Application,
}

pub fn set_termination(termination: Termination) -> Result<(), …>;
```

**The rule**

- The setting is read when ink-rs installs its process handlers (the first render through the driver). From then on it is fixed.
- A call before that sets it. A later call with the same value returns `Ok`; a later call with a different value returns an error and changes nothing.
- Pick the error type. If `Error` cannot take a new variant without breaking users, use a small dedicated type. Say which in the report.
- `testing::mount` and `testing::render` install no handlers; the setting does not affect them, and they do not fix it.

**`Application` on Unix**

- ink-rs registers neither conditional defaults nor `Signals` handlers for SIGINT, SIGTERM and SIGHUP. The host's handlers, installed before or after the mount, see these signals, and the process does not end on them.
- SIGWINCH, SIGTSTP and SIGCONT keep today's handling.

**`Application` on Windows**

- ink-rs leaves the console control events to the application. Check what `control_handler` does today. If it handles nothing but termination, `Application` skips `SetConsoleCtrlHandler`; otherwise keep the non-termination part. Say in the report which events count as termination.
- Add no new FFI; the existing unsafe blocks keep their `// SAFETY:` comments.

**Unchanged**

- The panic hook and the terminal restore on panic.
- `exit_on_ctrl_c`: it is about the Ctrl+C key in raw mode, not SIGINT.
- `Instance::unmount` and `AppHandle::exit` restore the terminal (raw mode, cursor, bracketed paste, kitty keyboard, alternate screen) as a termination signal does today. Check this holds; if it does not, fix it and say so.

## Tests

Add one PTY case in `tests/pty.rs`, Unix only:

- The child calls `set_termination(Termination::Application)`, registers its own SIGTERM flag with signal-hook, mounts an app, and raises SIGTERM.
- The child sees the flag set, renders a frame that shows it, checks that `set_termination(Termination::Handled)` now returns an error and `set_termination(Termination::Application)` returns `Ok`, unmounts, and exits with code 0.
- The parent checks the exit code, the frame rendered after the signal, and the restore bytes in the PTY output.

It needs a child process because the setting and the handlers are process-wide, and only a real signal shows that ink-rs left it alone. The existing tests cover the default.

## Out of scope

`docs/`, README, CHANGELOG, examples, the version number.

## Rules

- `CLAUDE.md` holds. No `unsafe` outside the Windows console FFI that exists today.
- Behaviour with the default setting does not change; the existing tests are the check.
- Code reads like the existing code: naming, comment density, idiom.
- New public items get doc comments that state the rule above, in plain declarative sentences.
- No new dependencies. signal-hook is a dependency already; the test may use it.
- **The sandbox cannot write `.git`.** Do not commit. Do not write report files into the repository.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings`
- `cargo test --offline --lib`
- `cargo test --offline --test pty`
- Run only these; the main agent runs the full suite.

## Report

Your final message is the report, in Chinese, without adverbs. It covers:

- the files you changed, and what each change does;
- the API, its names and the error type;
- where the setting lives and how `install_events` reads it on each platform;
- the Windows events that count as termination;
- whether unmount and exit restore the terminal as a signal does;
- every deviation from this brief, and why;
- the test you added;
- the exact commands you ran and their results.
