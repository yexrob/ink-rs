# Task: let the application own termination signals

## Goal

bingo, a downstream coding agent, embeds ink-rs in a tokio process that owns SIGINT, SIGTERM and SIGHUP. On SIGTERM or SIGHUP it ends the running turn, runs its `session.end` hooks, flushes its session store and unmounts the app before it exits.

Today ink-rs registers process-wide handlers on the first render. On a termination signal it restores the terminal and ends the process with the default action, so `wait_until_exit` does not return and the host's cleanup does not run. When no app is live the conditional defaults stay installed, so the host's own `tokio::signal` handlers lose to them. `Options` has no way out.

Add an option that leaves SIGINT, SIGTERM and SIGHUP to the application. Resize (SIGWINCH), suspend and resume (SIGTSTP, SIGCONT), the terminal restore on panic and the Ctrl+C key handling keep today's behaviour.

## Read first

- `<repo>/CLAUDE.md`; `<repo>/docs/design.md`, the parts on signals, suspend and terminal restore.
- `<repo>/src/app.rs`: `Options` and its defaults.
- `<repo>/src/driver/process.rs`: `Registration` and its `Drop`, `signal`, `registry().default_signals`, `termination`, `install_panic_hook`.
- `<repo>/src/driver/sys/unix.rs`: `install_events` (about lines 165-207) and `terminate`.
- `<repo>/src/driver/sys/windows.rs`: `install_events` and `control_handler` (`SetConsoleCtrlHandler`).
- `<repo>/src/driver/mod.rs` and `<repo>/src/driver/tokio.rs`: `Instance::unmount`, `wait_until_exit`.
- The existing tests on signals and suspend.

## Change

- A new field on `Options`, for example `termination: Termination` with `Handled` (the default, today's behaviour) and `Application`. Better names are welcome; say why in the report.
- With `Application`:
  - On Unix, ink-rs registers neither handlers nor conditional defaults for SIGINT, SIGTERM and SIGHUP.
  - On Windows, the console control handler leaves the termination events to the application. Say which events count as termination.
  - SIGWINCH, SIGTSTP and SIGCONT keep today's handling.
  - The application ends the app with `Instance::unmount` or `AppHandle::exit`. Both restore the terminal (raw mode, cursor, bracketed paste, kitty keyboard, alternate screen) the way a termination signal does today.
  - `exit_on_ctrl_c` keeps its meaning: it is about the Ctrl+C key in raw mode, not SIGINT.
- The registration is process-wide and happens on the first render (`OnceLock`). Decide the rule when apps with different settings mount in one process, and document it on the field.
- Keep the panic hook.

## Tests

- A test registers its own SIGTERM handler (signal-hook's flag) before the mount, mounts with `Application`, raises SIGTERM, and sees that the flag is set, the app keeps rendering, and `unmount` restores the terminal (bytes written to the `TestTerminal`).
- The default keeps today's tests green.

## Docs

A decision record for the option, and the `Options` docs.

## Report

The option's name and shape, the rule for several apps, the Windows mapping, and the tests.
