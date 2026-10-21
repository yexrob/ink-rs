# Task: implement the Windows driver in `src/driver/sys/windows.rs`

## Goal

`ink_rs::render`, `Instance` and `ink_rs::tokio` work on Windows 10 1809 (build 17763) and later, in conhost and Windows Terminal. The shared driver already calls a small `sys` interface; `src/driver/sys/unix.rs` implements it today. Add `sys/windows.rs` with the same items and wire it up. Node/libuv is the reference for every behavior below.

## Unsafe exemption

The project forbids unsafe code. The user granted one exemption: Windows console FFI. Rules:

- unsafe appears only in `src/driver/sys/windows.rs`.
- `src/lib.rs`: replace `#![forbid(unsafe_code)]` with `#![cfg_attr(not(windows), forbid(unsafe_code))]` and `#![cfg_attr(windows, deny(unsafe_code))]`; put `#[allow(unsafe_code)]` on the `mod windows;` item in `src/driver/sys/mod.rs`, nowhere else.
- Each unsafe block is as small as the call it wraps and carries a `// SAFETY:` comment naming the invariant (valid handle, buffer length, initialized out-parameter).
- Use `windows-sys` 0.61 (already in the offline registry as 0.61.2). Enable only the features you use.

## Interface to implement

Read `src/driver/sys/unix.rs`, `src/driver/process.rs` and `src/driver/stdin.rs` first. Provide the same items with these Windows behaviors:

1. **`Output`** writes to a duplicated stdout or stderr handle (`try_clone_to_owned`).
   - A console handle (`GetConsoleMode` succeeds): convert the bytes to UTF-16 and write them with `WriteConsoleW`, looping until every unit is written. One `write` call converts and writes its whole buffer, so a synchronized frame stays one buffer, as the Unix `File` path does. Invalid UTF-8 is replaced with U+FFFD. Do not change the console code page: `SetConsoleOutputCP` is process-global state.
   - Otherwise (pipe or file): write the bytes unchanged, as a `File`.
   - Keep the comment about bypassing std's `LineWriter`.
2. **VT output.** Once per process, add `ENABLE_VIRTUAL_TERMINAL_PROCESSING` to the console output modes of stdout and stderr (keep `ENABLE_PROCESSED_OUTPUT`; do not set `DISABLE_NEWLINE_AUTO_RETURN`, so LF keeps moving to column 0 like ONLCR). Do not reset it at unmount; libuv `uv__determine_vterm_state` does the same, and shells reset output modes.
3. **`stdout_is_tty`**: stdout is a console handle and VT processing is enabled on it. A console without VT support (legacy console) counts as not a TTY, so the app runs non-interactive and, with the colour rule below, without escape sequences. **`stdin_is_tty`**: `GetConsoleMode` succeeds on stdin. A msys/mintty pipe is not a TTY, as in libuv, even though `std::io::IsTerminal` reports it as one: do not use `IsTerminal` here.
4. **`size`**: `GetConsoleScreenBufferInfo` on stdout; columns = `dwSize.X`, rows = `srWindow.Bottom - srWindow.Top + 1` (libuv `uv__tty_console_signal_resize`). `None` when the call fails.
5. **Raw mode.** `SavedMode` is the console input mode (`u32`). Entering raw mode saves it and sets `ENABLE_VIRTUAL_TERMINAL_INPUT` alone: no line input, echo or processed input, so Ctrl+C arrives as byte 0x03, like Unix with ISIG cleared (libuv `uv_tty_set_mode`, `UV_TTY_MODE_RAW_VT`, which Node's `setRawMode(true)` uses). Do not set `ENABLE_EXTENDED_FLAGS`; quick-edit stays as the user set it. Restoring sets the saved mode; `emergency` makes no difference on Windows.
6. **Reader.**
   - `start`: duplicate the stdin handle, create a manual-reset event, spawn the `ink-stdin` thread.
   - Thread loop: `WaitForMultipleObjects` on the stdin handle and the event. Event signalled → return. Stdin signalled → read the available records with `ReadConsoleInputW` (bounded buffer), keep `KEY_EVENT` records that are key-down with a non-zero `UnicodeChar`, plus libuv's exception (key-up of `VK_MENU` with a non-zero char: the result of Alt+numpad composition), decode UTF-16 to UTF-8 keeping a pending high surrogate across reads, and pass the bytes to `stdin::input`. Ignore other record types. A failed wait or read passes the error to `stdin::ended` and returns. A console has no end of input.
   - In VT input mode the console delivers escape sequences as one record per character; do not add libuv's ESC prefix for Alt (that applies to its non-VT mode).
   - Keep the decoding in a small function separate from the FFI calls, so it can be tested without a console.
   - `stop`: set the event, join the thread, report a panic as Unix does.
7. **Process events** (`install_events`, once per process):
   - `SetConsoleCtrlHandler` with a handler. When `default_signals` is set (no live app), return FALSE so the system's default handling runs, the analog of `register_conditional_default`. Otherwise call `process::signal` with the ctrl type, then block the handler thread forever: on `CTRL_CLOSE_EVENT` (and LOGOFF/SHUTDOWN) the system ends the process as soon as the handler returns, so returning would skip cleanups. The process exits from `terminate` on the UI thread (last registration) or inside `process::signal` when no loop can run cleanups.
   - The handler cannot capture; store `default_signals` in a static.
   - Resize: a `ink-resize` thread queries `size()` every 100 ms and calls `process::resize()` when it changes. This replaces SIGWINCH. Do not use `SetWinEventHook` (user32, a message loop, and ntdll's undocumented `NtQueryInformationProcess` to find conhost); write that reason in a comment.
   - There is no stop/continue source.
8. **`Termination`** is the ctrl type (`u32`). **`terminate`** exits with `STATUS_CONTROL_C_EXIT` (0xC000013A), the exit code of a console program ended by Ctrl+C; `std::process::exit` suffices, no FFI.
9. **Job control.** Windows has none. Add `pub(in crate::driver) const JOB_CONTROL: bool` to both sys modules (Unix `true`). In `mount` (`src/driver/mod.rs`), clear `options.suspend_on_ctrl_z` when `!sys::JOB_CONTROL`, so Ctrl+Z (0x1A) reaches `use_input` as in Node. Windows `stop_process` is then unreachable; make it an empty function with a comment, or remove it from the interface if that reads better. Update the doc comment of `Options::suspend_on_ctrl_z` in `src/app.rs`.

## Colour detection

`src/driver/environment.rs`, `detect_color`: chalk's vendored supports-color has a Windows branch right after the `TERM=dumb` check (read it in the Ink clone's `node_modules`): Windows 10 build ≥ 14931 gives level 3, ≥ 10586 gives 2, older gives 1. With the 1809 minimum the branch gives 3. Port it as a platform input: `color_level` receives whether the platform is Windows (the driver passes `cfg!(windows)`), and the existing tests pass `false`. Add the platform argument without reading the Windows version.

## Wiring

- `src/lib.rs`: remove `#[cfg(unix)]` from `mod driver` and from the `render` / `Instance` / `tokio` re-exports. Apply the unsafe attributes above.
- `src/driver/sys/mod.rs`: add the Windows module and re-export, mirroring Unix.
- `Cargo.toml`: move `rustix`, `filedescriptor` and `signal-hook` to `[target.'cfg(unix)'.dependencies]`; add `windows-sys` under `[target.'cfg(windows)'.dependencies]`, and include the `Win32_System_Console` feature (tests will call `GetConsoleMode`). Add `x86_64-pc-windows-msvc` to `package.metadata.docs.rs.targets`.

## Tests

One Windows-only unit test in `sys/windows.rs` for the record decoding: a surrogate pair split across two reads, key-up records dropped, the Alt-composition key-up kept. It is needed because ConPTY cannot produce split reads on demand and the decoding keeps state across reads. No other new tests. Another worker is making `tests/` and `examples/` build on Windows; do not touch them.

## Docs

In Chinese, in the style of `docs/design.md` (plain statements, no adverbs):
- `docs/design.md` 平台 section: replace the "不支持 Windows" paragraph with the Windows design: minimum version, the `sys` split (shared logic vs `sys/unix.rs` / `sys/windows.rs`), the unsafe exemption and where it applies, and the rules above (VT input/output, `WriteConsoleW`, TTY rule, size, polling resize and why, ctrl handler and why it blocks, exit code, no job control).
- Update other passages your change makes false: the **resize** paragraph's first sentences (SIGWINCH thread → per-platform source), the 非 TTY / CI colour sentence ("不移植 Windows 分支"), the **Ctrl+Z** paragraph (no effect on Windows), and any other mention of Unix-only support.
- `README.md` "Platform support": Linux, macOS and Windows 10 1809+, in English.

## Commands

You cannot run Windows binaries here. Check the Windows build with cross-target `check`/`clippy`, which need no linker:

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo clippy --offline --target x86_64-pc-windows-msvc --lib -- -D warnings`
- `cargo clippy --offline --target x86_64-pc-windows-msvc --lib --no-default-features -- -D warnings`
- `cargo check --offline --target x86_64-pc-windows-msvc --lib --profile test` (the unit tests compile)
- `cargo test --offline --lib`
- `cargo test --offline --test pty --test examples_pty`
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`

Do not run the full `cargo test`; I run it at integration, and Windows tests run on CI.

## Project context

ink-rs is a Rust port of Ink (React for CLIs). Read `CLAUDE.md`, `AGENTS.md` and the 平台, 终端输出 and 退出 parts of `docs/design.md`. A read-only clone of Ink with its `node_modules` is at `<scratchpad>/ink-src/ink`; read and grep it, never run anything from it. libuv's Windows tty source is at `<scratchpad>/win-research/libuv-tty.c`. `windows-sys` source is in `~/.cargo/registry/src/*/windows-sys-0.61.2/`.

## Rules

- unsafe only as described above. No dependencies beyond `windows-sys`. Code reads like the surrounding code: same naming, comment density and idiom. No code written only to satisfy a test.
- Stay inside `src/`, `Cargo.toml`, `README.md` and `docs/design.md`.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report, in Chinese, no adverbs:
- the files you changed, and what changed in each;
- every unsafe block: the call and its SAFETY argument, one line each;
- design choices this brief left open, and what you chose;
- every deviation from this brief, and why;
- what could not be verified without Windows, as a checklist for CI;
- the exact commands you ran and their results.
