# ink-rs M3 phase 3a: `use_input`, Ctrl+C exit and raw-mode holders

You are working in a git worktree of ink-rs at `<scratchpad>/wt/m3-p3a`. Read `CLAUDE.md`, then `docs/design.md`: the sections 事件循环, 输入 (the rules this phase implements, except paste, kitty negotiation, the reader thread, termios and multiple apps), 退出, and 移植 Ink 测试的方法. Then read:

- `src/input/mod.rs`, `src/input/parser.rs`, `src/input/keypress.rs` (the parser and key mapping, ported and tested in phase 1);
- `src/runtime/mod.rs` (`Runtime`, `commit`, `passive_effects`, `after_write`, `Drop`), `src/runtime/hooks.rs` (how `use_cursor` builds on an insertion effect), `src/runtime/state.rs` (`Store`), `src/runtime/app_handle.rs` (`UpdateQueue`, the exit request from phase 2);
- `src/app.rs` (`App::render_if_due`, `App::unmount`), `src/terminal.rs`, `src/testing.rs` (`TestTerminal`, `TestApp`), `src/driver/process.rs` (`ProcessTerminal`).

M3 adds input, paste, the kitty keyboard protocol, focus and exit. Phase 2 added the exit API. This phase wires input into the runtime and App: `use_input`, Ctrl+C exit, raw-mode holders and the test driver's input. Later phases add `use_paste` and kitty negotiation (3b), focus (4), and the production reader thread, termios and the natural-end change (5). Do not start those.

References, all read only:
- The Ink checkout at `<scratchpad>/ink-src/ink`. Read it; never run anything from it. Sources: `src/hooks/use-input.ts`, `src/components/App.tsx:220-460` (`clearInputState`, `handleExit`, `handleInput`, `emitInput`, `schedulePendingInputFlush`, `handleReadable`, `handleSetRawMode`).
- `<scratchpad>/m3-research/tests-input.md` §1.1–§1.3, §1.9–§1.11, §2, §3 (test inventory and the key-event pipeline), and `tests-focus-exit.md` §6 and §7.

## 1. What to build

### 1.1 Public API

```rust
// Scope
pub fn use_input(&mut self, active: bool, handler: impl FnMut(&str, KeyEvent) + 'static)

// Options
pub exit_on_ctrl_c: bool   // default true

// crate root
pub use input::{KeyEvent, KeyEventType};
```

- `use_input` is Ink's `useInput(handler, {isActive})`. Rustdoc: what the handler receives (text runs stay one call; see `From<ParsedKey> for (String, KeyEvent)`), that an active hook enables raw mode, that the handler from the latest render is called (Ink wraps it in `useEffectEvent`), and the Ctrl+C rule below.
- `Options::exit_on_ctrl_c`: Ink's `exitOnCtrlC`. Rustdoc: a Ctrl+C press or repeat exits the app with `Ok(None)` and does not reach `use_input`; a release does.
- Remove the module-level `expect(dead_code)` on `mod input` in `src/lib.rs`. If an item stays unused until phase 3b, put a narrow `#[expect(dead_code, reason = "...")]` on that item and report it.
- Update the rustdoc in `src/lib.rs` if it lists hooks or test driver operations.

### 1.2 Runtime: parsing, dispatch, holders

Runtime owns the parser and the dispatch, so `testing::render` can feed input in phase 4 without App (design.md 分发). Keep the state in the runtime's store or beside it, `pub(crate)` or narrower.

- **Holders.** An active `use_input` registers in a passive effect keyed on `active` and unregisters in its cleanup (Ink's two `useEffect`s with `[isActive, …]` deps). Handlers are called in registration order, which is passive-effect order; a hook that turns inactive and active again moves to the end, as with an `EventEmitter`. Build it from the existing hook machinery, as `use_cursor` builds on an insertion effect; no new effect phase. The number of registered hooks is the raw-mode holder count. Phases 3b and 4 add `use_paste` and `use_focus` holders to the same count.
- **Count reaches zero.** Reset the parser and clear the escape deadline at once, inside the cleanup (Ink `clearInputState`). The terminal mode switch waits for the end of the step (1.3), so a swap within one commit keeps raw mode on and still resets the parser (Ink test `clear pending input parser state when swapping components that use useInput`).
- **`input(bytes, now)`.** Ignore the bytes while the holder count is zero: the production reader reads only in raw mode. Otherwise clear the escape deadline, push the bytes through `InputParser`, and handle each event in order:
  - `Text(text)`: if `is_complete_control_sequence(text)` and `parse_keypress(text)` is not kitty and has an empty name, drop it (Ink App.tsx readable handler). Otherwise emit it.
  - `Paste(text)`: there is no `use_paste` yet, so emit the paste text as one input (Ink's fallback when no paste listener exists). It goes through the same emit as text, so a `\x03` inside a longer paste does not exit.
  - After the events, if `has_pending_escape()`, set the escape deadline to `now + 20ms`.
- **Emit** (Ink `emitInput` → `handleInput` → the `useInput` listener):
  1. `parse_keypress(text)`.
  2. If `exit_on_ctrl_c` and the key is `c` with ctrl and its event type is not `Release`: request exit with `Ok(None)` through the update queue and return. The key does not reach handlers.
  3. Map with `From<ParsedKey> for (String, KeyEvent)` and call each registered handler with `(&input, key)`. Call from a snapshot of the registrations, as `EventEmitter.emit` does, and hold no `RefCell` borrow of shared runtime state across a handler call.
- **Exit stops the read.** Once an exit request is pending (from Ctrl+C or from a handler through `AppHandle`), drop the rest of the events of this read. Ink tears the app down inside the same read; the tests do not observe the remainder.
- **Escape flush.** Expose the deadline so the step can wake for it, and a flush that runs when the deadline has passed: take `flush_pending_escape()` and emit it without the drop filter (Ink `schedulePendingInputFlush`).
- **`exit_on_ctrl_c`** reaches the runtime from `Options` through App. The runtime default is `true`, Ink's default, which `testing::render` will use in phase 4.
- `UpdateQueue::request_exit` becomes `pub(crate)` for the Ctrl+C exit and for 1.3.

Handlers run outside the commit, like `AppHandle::update` callbacks. They may write `State`, call `AppHandle` methods, and write through `use_stdout`. The commit for a read happens in the next step, so one read's updates commit together (design.md 分发).

### 1.3 App and Terminal

`Terminal` gains two methods:

```rust
fn stdin_is_tty(&self) -> bool;
fn set_raw_mode(&mut self, enabled: bool); // failures are retained for take_error, like write
```

- `App::input(bytes, now)` forwards to the runtime; ignore it after unmount.
- `App::render_if_due(now)`:
  - At the start, run the escape flush when its deadline has passed.
  - At the end, after passive effects and `after_write`, and before the exit check: compare `holders > 0` with the raw mode App last set. On a change to on: if `stdin_is_tty()` is false, request exit with `Err(Error::Io(io::Error::new(io::ErrorKind::Unsupported, "raw mode is not supported on stdin")))` and leave raw mode off (Ink `setRawMode` throws); otherwise call `set_raw_mode(true)`. On a change to off, call `set_raw_mode(false)`. Comparing at the end of the step gives Ink's deferred disable: a swap within one commit does not switch the mode.
  - The returned wake time is the earliest of the current result and the escape deadline.
- `App::unmount`: if raw mode is on, call `set_raw_mode(false)` first, before the pending frame, in both the normal and the panicking branch (design.md 退出 teardown order).
- `ProcessTerminal`: `stdin_is_tty` returns `io::stdin().is_terminal()`. `set_raw_mode` does nothing yet; a one-line comment says the production input driver adds termios and the reader. Do not change `natural_end`.

### 1.4 Test driver

- `TestTerminal::with_stdin_tty(self, tty: bool) -> Self`. The default is `true`, as Ink's `createStdin` reports `isTTY`.
- `TestTerminal::raw_mode_calls(&self) -> Vec<bool>`: every `set_raw_mode` argument in call order, shared by clones, like the write log.
- `TestApp::input(&mut self, bytes: impl AsRef<[u8]>)`: one stdin read at the current clock time, then a step. Rustdoc: input reaches active `use_input` hooks; a lone Esc or an incomplete sequence waits 20 ms, so call `advance`.

Rustdoc on every public item you add or change.

## 2. Tests

Port, names and order as in Ink (design.md 移植 Ink 测试的方法), through `testing::mount` and `TestTerminal` (interactive TTY, 100 columns, unless the Ink test says otherwise). New files get the usual header, `//! Ported from ink test/<file> @ 26d2c3f.`

Ink fixtures exit on a match and throw on anything else. Port a fixture as a Rust component with the same branches: call `exit()` on a `use_app` handle on a match and `panic!` otherwise. Assert `exit_result()` is `Some(Ok(None))`; this replaces the PTY's `exited` text and exit code. For a lone Esc or an incomplete sequence, `advance(Duration::from_millis(20))` after the input.

1. `test/hooks-use-input.tsx` → `tests/hooks_use_input.rs`, 32 tests, fixture `test/fixtures/use-input.tsx`. Not ported: `discrete priority keeps states in sync with useTransition during rapid input` (React transition) and `handle remove (delete)` (same bytes and fixture condition as `handle delete`). The parameterised `drops unmapped control sequence` test becomes 8 tests, one per case; it records `(input, escape)` and asserts the full list after the sequence and `q` in two reads.
2. `test/hooks-use-input-navigation.tsx` → `tests/hooks_use_input_navigation.rs`, 18 tests, same fixture. `handles rapid arrows and enter in one chunk` sends the four keys in one read.
3. `test/hooks-use-input-kitty.tsx` → `tests/hooks_use_input_kitty.rs`, 24 tests, fixture `test/fixtures/use-input-kitty.tsx`. The fixture disables kitty negotiation; ink-rs has no negotiation yet, and decoding does not depend on it.
4. `test/input-keypad-enter.tsx` → `tests/input_keypad_enter.rs`, 2 tests (the loop's two cases).
5. `test/input-buffered-ctrl-c.tsx` → `tests/input_buffered_ctrl_c.rs`: the three cases of `Ctrl+C exits when buffered with text` (`"\x03"`, `"ab\x03"`, `"\x03ab"`). The `setImmediate` wait becomes the step after `input`. The second Ink test registers `usePaste` and comes in phase 3b.
6. `test/rerender-input.tsx` → `tests/rerender_input.rs`: `rerender() keeps a pending Escape keypress` and `rerender() keeps an escape sequence that spans two reads`. The paste case comes in phase 3b; the listener-churn case is not applicable.
7. `test/exit-keyboard.tsx` → `tests/exit_keyboard.rs`: the 7 cases of `automatic exit handles ${name}`. `isExited` is `exit_result().is_some()`. The `undefined` exit option case uses `Options::default()`.
8. `test/cursor.tsx`: the six input tests (lines 96, 174, 197, 220, 259, 361) → `tests/cursor.rs` at Ink's position. Follow the file's existing adaptation: vt100 cursor position and visibility (`common::Screen`) in place of the hide/show and `cursorTo` byte checks; keep a byte check where the screen cannot show the point, and say which.
9. `test/hooks.tsx`: `useInput - ignore input if not active` (fixture `use-input-multiple.tsx`) and ``useInput - handle Ctrl+C via kitty protocol when `exitOnCtrlC` is `false` `` (fixture `use-input-ctrl-c.tsx`) → `tests/hooks.rs` at Ink's position. The first fixture exits from a 100 ms timer; ink-rs has no timers yet, so the test exits through a retained handle after the input. The legacy `\x03` case needs a PTY (ISIG) and comes in phase 5.
10. `test/components.tsx`: lines 1136, 1212, 1256, 1308, 1384 → `tests/components.rs` at Ink's position. These Ink tests call `useStdin().setRawMode` or use `debug: true` with a spied stdin. ink-rs has no `use_stdin`; an active `use_input` is the raw-mode holder, and `raw_mode_calls()` replaces the `setRawMode` spy. `ref`/`unref` and `listenerCount` have no counterpart; drop them and say so. The microtask wait disappears: the switch happens at the end of the step. `setRawMode() should throw if raw mode is not supported` becomes: non-TTY stdin, an active `use_input`, `exit_result()` is `Err(Error::Io(e))` with `ErrorKind::Unsupported`, and `raw_mode_calls()` is empty. `render different component based on whether stdin is a TTY or not` needs `isRawModeSupported` and is not ported.

No other tests. The parser and keypress unit tests exist; do not add more.

## Rules

- No `unsafe`. No new dependencies.
- No code that exists only to satisfy a test.
- Match the surrounding code: `pub(crate)` for internal items, imports at the top, comments that say why and cite the Ink source where the code ports it. Run `rustfmt --edition 2024` on every file you touch.
- Do not edit `docs/`. Report the documentation changes instead (below).
- Run only the following. Every test in these targets must pass.
  - `cargo test --test hooks_use_input --test hooks_use_input_navigation --test hooks_use_input_kitty --test input_keypad_enter --test input_buffered_ctrl_c --test rerender_input --test exit_keyboard --test cursor --test hooks --test components`
  - `cargo test --lib runtime`
  - `cargo test --lib input`
  - `cargo test --lib driver`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

1. Proposed commits: message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. The public API added, with its rustdoc; the internal changes (where the input state lives, the hook, the emit path, App's raw-mode check, the wake time).
3. Where the behaviour departs from Ink, with the reason.
4. The ported tests: names, files, and every assertion adapted, with the reason.
5. Results of the commands above. Name any failing test, with your diagnosis.
6. The rows to change in `docs/ink-tests.md` (Ink file, ported count, Rust file), the entries to move out of the M3 lists, and anything design.md should record. Open questions.
