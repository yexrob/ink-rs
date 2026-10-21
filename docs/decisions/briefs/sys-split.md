# Task: split the driver's OS calls into `src/driver/sys/unix.rs`

## Goal

ink-rs is about to gain Windows support. Before that, separate the driver's platform-neutral logic from its Unix system calls, so that a later `sys/windows.rs` implements the same small interface and nothing else in the driver changes. This task is a pure refactor: behavior on macOS and Linux stays byte-for-byte the same, and no Windows code is written yet.

## Current layout

- `src/driver/process.rs`: `RestoreState`, `Failure`, `ProcessTerminal` (the `Terminal` impl), `Entry`, `Registry`, `Registration`, the panic hook, the signal thread (`install_signals`) and `terminate`.
- `src/driver/stdin.rs`: raw-mode holders (`Target`, `Route`), the lifecycle lock (`Stdin`), `acquire` / `release` / `restore`, termios save and raw setup, the reader thread (`read`, using `filedescriptor::{socketpair, poll}`), and two unit tests.
- `src/driver/mod.rs`, `tokio.rs`, `environment.rs`: platform-neutral already; leave them alone except for imports.

## Target layout

```
src/driver/
  process.rs     shared: RestoreState, Failure, ProcessTerminal, Entry, Registry,
                 Registration, panic hook, and the fan-out that reacts to OS events
  stdin.rs       shared: Target, Route, Stdin lifecycle, acquire/release/restore,
                 and the routing that delivers bytes / end-of-input to the last holder
  sys/mod.rs     #[cfg(unix)] mod unix; #[cfg(unix)] pub(super) use unix::*;
  sys/unix.rs    every rustix, signal_hook, filedescriptor and std::os::fd use
```

After the change, `rustix`, `signal_hook`, `filedescriptor` and `std::os::fd` / `std::os::unix` appear only in `sys/unix.rs` (and in tests that stay Unix-only).

## The interface `sys` provides

Use these responsibilities; pick the exact names and signatures so the shared code reads well, and keep the set minimal. The Windows module will provide the same items.

1. **Output handles.** Duplicated stdout and stderr handles that implement `Write` (today: `File` from `try_clone_to_owned`). Keep the comment about bypassing std's `LineWriter`. On Windows this becomes a type that writes UTF-16 to a console, so expose it as a `sys` type or alias, not as `File` in the shared code.
2. **TTY checks** for stdout and stdin.
3. **Window size**: `Option<(columns, rows)>`.
4. **Raw mode**: entering raw mode returns a saved-mode value; restoring takes that value and the `emergency` flag (TCSANOW vs TCSADRAIN today). Keep the libuv comments with the termios setup. `Stdin::saved` holds the `sys` saved-mode type.
5. **Reader thread**: start it with the shared `Route`; stop it (today: write to the socketpair, then join). The thread body in `sys` only obtains bytes or end-of-input from the OS; the routing it does today (post `Message::Input` to the last holder; on EOF or error mark `ended`, record the error, post `InputEnded`) moves into shared functions in `stdin.rs` that the reader calls. Keep "hold route until Started is queued" and the other ordering comments where the logic lives.
6. **Process events**: install the OS event source once (today: the signal-hook thread plus `register_conditional_default`). The thread body in `sys` maps OS events onto shared functions in `process.rs`:
   - resize → post `Message::Resize` to every entry;
   - stop → the Stop fan-out that waits for every in-loop acknowledgement (the session-leader check and `emulate_default_handler(SIGTSTP)` stay in `sys`, around the shared fan-out);
   - continue → post `Message::Continue` to every entry;
   - termination → the shared block that records the first termination in `Registry::termination`, restores and terminates when no loop can run cleanups, and otherwise posts `Message::Signal`.
   The `default_signals` flag stays in the shared `Registry`; `sys` reads it when installing conditional defaults.
7. **Termination value and exit**: a `sys` type for "which termination happened" (today the signal number, `i32`) stored in `Registry::termination`, and `terminate(value) -> !` (today `emulate_default_handler` then `abort`).
8. **Job control**: `stop_process()` (today `kill_current_process_group(TSTP)`). Windows will have no job control; do not add anything for that yet.

Lock order and the long comments in `process.rs` / `stdin.rs` describe shared invariants: keep them next to the shared code, and keep each OS-specific comment next to the OS call it explains.

## Existing tests

- The two unit tests in `stdin.rs` build a reader on `socketpair` and `/dev/null`. Keep them, moved to wherever the code they exercise ends up; mark them Unix-only if they stay tied to socketpair. Their assertions do not change.
- `src/driver/tests.rs` stays as is.
- No new tests: this is a refactor and the existing PTY tests cover the behavior.

## Leave unchanged

- `#[cfg(unix)] mod driver;` and the `cfg(unix)` re-exports in `src/lib.rs`: the driver is still Unix-only after this task.
- `Cargo.toml`, `tests/`, `examples/`, `docs/`.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo test --offline --lib`
- `cargo test --offline --test pty --test examples_pty --test suspend_terminal --test terminal_resize`
- `cargo test --offline --no-default-features --lib`
- `cargo check --offline --target x86_64-pc-windows-msvc --lib` (must still pass: the driver is excluded there)
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`

Do not run the full `cargo test`; I run it at integration.

## Project context

ink-rs is a Rust port of Ink (React for CLIs). Read `CLAUDE.md`, `AGENTS.md` and the 平台 / 终端输出 / 退出 parts of `docs/design.md` before you start.

## Rules

- No unsafe code. No new dependencies. Code reads like the surrounding code: same naming, comment density and idiom. No code written only to satisfy a test.
- Stay inside `src/driver/`. Other workers may change other parts of the tree.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report, in Chinese, no adverbs:
- the files you changed, and what moved where;
- the `sys` interface as you defined it (item names and signatures);
- design choices this brief left open, and what you chose;
- every deviation from this brief, and why;
- the exact commands you ran and their results.
