# Task: make the tests and examples run on Windows, add CI and the #969 fixture

## Goal

ink-rs is gaining Windows support (Windows 10 1809+, conhost and Windows Terminal). Another worker is writing the Windows driver in `src/driver/sys/windows.rs` at the same time; you do not see its code. Your part: make `tests/` and `examples/` build and pass on Windows as well as on Linux and macOS, add a GitHub Actions workflow that runs fmt, clippy and the full test suite on all three, and add a PTY fixture for Ink #969. Your work is checked on Windows CI after both parts are merged, so write the Windows code against the real APIs and keep it simple enough to review.

## What the Windows driver will do

Rely on this behavior; it mirrors Node/libuv:

- stdout/stdin are a TTY when `GetConsoleMode` succeeds on them (and VT output could be enabled). Output goes through `WriteConsoleW` with VT processing on.
- Raw mode sets the console input mode to `ENABLE_VIRTUAL_TERMINAL_INPUT` alone; Ctrl+C arrives as byte 0x03 input. Restoring sets the saved mode back.
- Resize comes from a 100 ms poll of the console size.
- Windows has no job control: `suspend_on_ctrl_z` has no effect and Ctrl+Z (0x1A) reaches `use_input`.
- The crate gets `[target.'cfg(windows)'.dependencies] windows-sys = "0.61"` with at least `Win32_Foundation` and `Win32_System_Console`; `rustix`, `filedescriptor` and `signal-hook` move to `[target.'cfg(unix)'.dependencies]`. Integration tests and examples may use these crate dependencies on their platform. Do not edit `Cargo.toml`; if you need another windows-sys feature, say so in the report.

## How PTY tests behave on Windows

`portable-pty` uses ConPTY on Windows (read `~/.cargo/registry/src/*/portable-pty-0.9.0/src/win/`). Differences from a Unix PTY that the harness must handle:

1. **The output is conhost's re-render**, not the child's bytes. conhost parses what the child writes into its buffer and paints the result as its own VT stream: it can split text runs, reorder or drop cursor-visibility sequences, drop private modes it does not know (`?2026`), and add its own sequences (`?9001h`, `?1004h`, `?25l`/`?25h` around paints). Assertions that look for raw ink-rs bytes (`\x1b[?25h`, `\x1b[?25l`, markers in the byte stream) hold on Unix; on Windows assert on `vt100` screen state (contents with scrollback, `hide_cursor()`) instead. Keep the Unix assertions as they are where they test ink-rs's own bytes.
2. **Cursor position request.** portable-pty creates the pseudoconsole with `PSEUDOCONSOLE_INHERIT_CURSOR`, so conhost writes `ESC[6n` at startup and waits for a cursor position report before it renders. `Session` must answer `ESC[6n` with `ESC[1;1R` on Windows, also when the request is split across reads.
3. **End of output.** The output pipe reaches EOF only after the pseudoconsole is closed (`ClosePseudoConsole`, which runs when both the master and the slave are dropped). After the child exits, close the pseudoconsole and drain the reader until it ends. A read on a pipe whose writer closed can fail with `ERROR_BROKEN_PIPE` (109) instead of returning 0; treat it as end of output on Windows, like the Linux EIO case.
4. **Resize**: `MasterPty::resize` calls `ResizePseudoConsole`; the driver sees it through its poll, not a signal. Fix the comment on `Session::resize`.
5. `Session::spawn` already removes `CI` and `CONTINUOUS_INTEGRATION`; GitHub Actions sets `CI=true`, which would make the app non-interactive.

## Changes

1. **`examples/suspend-terminal/app.rs`**: Ink's example runs `sh -c` on every platform; keep `sh` (Git for Windows provides it on the GitHub runner). Put the `ExitStatusExt`/signal-name branch under `cfg(unix)`; on Windows an exit status always has a code.
2. **`tests/pty.rs`**:
   - The raw-mode restoration check (`stdin_termios` / `same_termios`, marker `termios-restored`) gets a Windows implementation that compares the stdin console mode from `GetConsoleMode` before and after. Give the helper and marker platform-neutral names. The FFI call is the one place unsafe is allowed in tests (the user exempted Windows console FFI): one minimal block with a `// SAFETY:` comment.
   - `JobControl` and `Orphan` are Unix-only (`cfg(unix)` on the variants, the `CASES` entries and their code).
   - Handover, marker and overview assertions follow point 1 above on Windows.
   - Keep the case list, names and Ink references.
3. **`tests/common/pty.rs`**: points 2–4 above. Keep the Unix behavior unchanged.
4. **`tests/examples_pty.rs`**: make it build and pass on Windows. `check_jest_frames` splits frames on `ESC[?2026l`, which conhost drops; decide whether the check still means something on Windows and make it hold or restrict it, and say which in the report.
5. **Other tests**: scan `tests/` and `examples/` for Unix assumptions and fix them. Known one: `tests/errors.rs` expects `" tests/errors.rs:16:9"`; on Windows `file!()` may use `\`. Build the expected line from `file!()` instead of hard-coding the separator. Also look for `/dev/`, `std::os::unix`, signal use, LF-only assumptions and shell commands.
6. **Line endings.** The GitHub Windows runner checks out with `core.autocrlf=true`, which turns source and fixture files (`tests/fixtures/error source.txt`, the source lines the error overview prints) into CRLF. Add a root `.gitattributes` with `* text=auto eol=lf` so every checkout keeps LF.
7. **#969 fixture**, a new case in `tests/pty.rs` that runs on every platform. Ink 8.0.0 `test/render.tsx` `#969: full-height rerenders on Windows should clear terminal between frames` with `test/fixtures/issue-969-windows-full-height-rerender.tsx` and `issue-450-fixture-helpers.tsx` (read them in the Ink clone). Ink falls back to a full clear between full-height frames on win32; ink-rs does not copy that, and this fixture is the experiment that decides whether it must. Port the fixture: a column `View` whose height equals the terminal rows (from `use_window_size`), with `#450 top`, a `flex_grow` row holding `frame N`, and a bottom row; it rerenders every 100 ms up to `frame 8`, then exits. Make the bottom row fill the full width, so each frame writes the bottom-right cell, the case Ink's comment names ("Windows consoles scroll when the bottom-right cell is written"). Run it in a 6-row PTY (Ink's default) and a width of your choice. Assert on the final `vt100` screen including scrollback: `frame 8` appears once, no earlier `frame N` remains, `#450 top` and the bottom row appear once each. Name it after the Ink test with a description that states what ink-rs asserts.
8. **`.github/workflows/ci.yml`**, new:
   - triggers: `push` and `pull_request`;
   - `cargo fmt --all --check` on one OS;
   - on `ubuntu-latest`, `macos-latest` and `windows-latest`: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, each with default features and with `--no-default-features`;
   - stable toolchain through `dtolnay/rust-toolchain`, `Swatinem/rust-cache@v2`, `actions/checkout@v4`; `fail-fast: false` so one platform's failure does not hide another's;
   - nothing else (no release, coverage or MSRV jobs).

## Tests

No new tests beyond the #969 case: it is needed because Ink has a Windows-only output branch for it and ink-rs replaces that branch with this measurement. Do not weaken an existing assertion on Unix to make Windows pass.

## Leave unchanged

`src/`, `Cargo.toml`, `README.md`, `docs/`. The other worker owns them. Do not update `docs/ink-tests.md`; I do that after the CI result.

## Commands

You cannot run Windows binaries, and the Windows driver is not in your tree, so the Windows build of the tests cannot be checked here. Check what you can:

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo test --offline --test pty --test examples_pty --test errors`
- `cargo test --offline --no-default-features --test pty --test examples_pty`
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`

If you have a YAML linter on the machine, validate the workflow; otherwise read it twice. Do not run the full `cargo test`; I run it at integration.

## Project context

ink-rs is a Rust port of Ink (React for CLIs). Read `CLAUDE.md`, `AGENTS.md` and the 平台 part of `docs/design.md`. A read-only clone of Ink with its `node_modules` is at `<scratchpad>/ink-src/ink`; read and grep it, never run anything from it. Dependency sources are in `~/.cargo/registry/src/*/` (`portable-pty-0.9.0`, `vt100-0.16.*`, `windows-sys-0.61.2`).

## Rules

- unsafe only in the one Windows console-mode helper described above. No new dependencies. Code reads like the surrounding code: same naming, comment density and idiom. No code written only to satisfy a test.
- Stay inside `tests/`, `examples/`, `.github/` and `.gitattributes`.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report, in Chinese, no adverbs:
- the files you changed, and what changed in each;
- the unsafe block, with its SAFETY argument;
- design choices this brief left open, and what you chose (including the `check_jest_frames` decision);
- every deviation from this brief, and why;
- what could not be verified without Windows, as a checklist for CI;
- the exact commands you ran and their results.
