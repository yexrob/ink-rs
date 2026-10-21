# ink-rs M3 phase 3b: `use_paste`, bracketed paste and kitty keyboard negotiation

You are working in a git worktree of ink-rs at `<scratchpad>/wt/m3-p3b`. Read `CLAUDE.md`, then `docs/design.md`: the sections 输入 (粘贴, kitty 键盘协议, raw mode, 分发), 恢复终端 (the restore snapshot), 退出, and 移植 Ink 测试的方法. Then read:

- `src/runtime/input.rs` (phase 3a: `InputState`, `Registration`, `Runtime::input`, `flush_input`, `emit_input`), `src/runtime/hooks.rs` (`Scope::use_input`), `src/runtime/state.rs` (`Store::input`);
- `src/input/mod.rs`, `src/input/parser.rs` (`InputEvent::Paste`), `src/input/keypress.rs`;
- `src/app.rs` (`Options`, `AppOutput::{restore_bytes, write, restore}`, `App::new`, `App::render_if_due` and its raw-mode check, `App::input`, `App::unmount`);
- `src/terminal.rs`, `src/testing.rs` (`TestTerminal`, `mount`, `TestApp`), `src/driver/mod.rs` and `src/driver/tests.rs` (the other `App::new` callers);
- `tests/hooks_use_input.rs`, `tests/input_buffered_ctrl_c.rs`, `tests/rerender_input.rs` (phase 3a ports; follow their style).

Phase 3a added `use_input`, Ctrl+C exit and raw-mode holders. This phase adds `use_paste`, bracketed paste mode, the `kitty_keyboard` option with its negotiation, and the matching restore bytes. Focus (phase 4) and the production reader thread, termios and the natural-end change (phase 5) are out of scope.

References, all read only:
- The Ink checkout at `<scratchpad>/ink-src/ink`. Read it; never run anything from it. Sources: `src/hooks/use-paste.ts`; `src/components/App.tsx` (`handleReadable` around line 320, `handleSetBracketedPasteMode` around line 453, the unmount effect around line 705); `src/ink.tsx` (`initKittyKeyboard`, `confirmKittySupport`, `enableKittyProtocol` around line 826, `finishUnmount` around line 1160); `src/kitty-keyboard.ts` (flags and `resolveFlags`).
- `<scratchpad>/m3-research/tests-input.md`: the rows PS-01..04, KN-01..05, KK-64..75, BC-04/05, RI-03, and §3 (the key-event pipeline).

## 1. What to build

### 1.1 Public API

```rust
// Scope
pub fn use_paste(&mut self, active: bool, handler: impl FnMut(&str) + 'static)

// Options
pub kitty_keyboard: KittyKeyboard   // default KittyKeyboard::Disabled

// crate root
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KittyKeyboard {
    #[default]
    Disabled,
    Auto(KittyFlags),
    Enabled(KittyFlags),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KittyFlags(u8);   // private field
impl KittyFlags {
    pub const DISAMBIGUATE_ESCAPE_CODES: Self;        // 1
    pub const REPORT_EVENT_TYPES: Self;               // 2
    pub const REPORT_ALTERNATE_KEYS: Self;            // 4
    pub const REPORT_ALL_KEYS_AS_ESCAPE_CODES: Self;  // 8
    pub const REPORT_ASSOCIATED_TEXT: Self;           // 16 | 8
}
impl Default for KittyFlags   // DISAMBIGUATE_ESCAPE_CODES
impl BitOr for KittyFlags
```

- `use_paste` is Ink's `usePaste(handler, {isActive})`. Rustdoc: an active hook enables raw mode and bracketed paste mode; pasted text arrives in one call, verbatim, and does not reach `use_input` while any `use_paste` is active; without one, `use_input` receives the paste as one input; the handler from the latest render is called, in registration order.
- `KittyKeyboard` is Ink's `kittyKeyboard` option. Ink's protocol is opt-in: an omitted option does nothing, so the default is `Disabled`. `Auto` and `Enabled` carry the flags, whose default is Ink's `['disambiguateEscapeCodes']`. Rustdoc on each variant gives its condition (1.4).
- `REPORT_ASSOCIATED_TEXT` includes the all-keys bit, the value 24. Ink's `resolveFlags` adds it because the protocol defines associated text only with all-key reporting. Put the bit in the constant, so there is no separate resolve step, and say why in its rustdoc.
- Put `KittyKeyboard`, `KittyFlags` and the query-response matcher (1.2) in a new `src/input/kitty.rs`, and re-export the two public types from the crate root.

Rustdoc on every public item you add or change.

### 1.2 Runtime

- **Paste holders.** An active `use_paste` registers in a passive effect keyed on `active`, built the same way as `use_input` (latest-render handler, registration in effect order, removal in the cleanup). Keep one registration mechanism for both channels; do not copy `Registration`.
- **Raw-mode holders** are now input registrations plus paste registrations. The parser and the escape deadline reset when the sum reaches zero (Ink `clearInputState` runs on the raw-mode count, which `usePaste` also holds). Add an accessor for the paste count; App uses it for bracketed paste mode.
- **Paste events.** `InputEvent::Paste(text)`: when at least one paste handler is registered, call each from a snapshot with the text, with no Ctrl+C check and no drop filter (Ink emits `paste` on its own channel). Otherwise keep the phase 3a fallback through `emit_input`.
- **Kitty query responses.** A `Text` event that matches `^\x1b\[\?\d+u$` is consumed before the drop filter, whatever the kitty option, and `Runtime::input` reports that one arrived (Ink App.tsx `onKittyQueryResponse`). A paste never matches: a response inside a paste is input (KN-05). Write the matcher by hand; no regex dependency.
- The exit rule from phase 3a holds for every event kind: once an exit is pending, the rest of the read is dropped.

### 1.3 Bracketed paste mode (App)

At the end of `render_if_due`, where phase 3a compares raw mode, compare `raw mode is on && paste holders > 0` with the paste mode App last set, and switch:
- writing `\x1b[?2004h` or `\x1b[?2004l` through `Terminal::write`, with the restore bytes for the new state, and without synchronized-update wrapping;
- only when stdout is a TTY (Ink `handleSetBracketedPasteMode` returns early otherwise). When it is not, the mode stays off.

Order within the step: turning on, raw mode first, then paste (Ink's effect calls `setRawMode(true)`, then `setBracketedPasteMode(true)`); turning off, paste first, then raw mode (Ink writes `?2004l` at once and defers the raw-mode disable). When stdin is not a TTY, raw mode fails as in phase 3a, so paste mode stays off.

Ink switches paste mode inside the effect; comparing at the end of the step means a `use_paste` swap within one commit writes nothing. Say so in a comment.

### 1.4 Kitty keyboard negotiation (App)

`App::new` takes the current time, `now: Instant`; update `testing::mount`, `driver::render` and `src/driver/tests.rs`. After the alternate-screen write (Ink calls `setAlternateScreen` before `initKittyKeyboard`), and before the first frame:
- `Enabled(flags)`: when stdin and stdout are TTYs, whatever the interactive mode, write `\x1b[>{flags}u` and mark the protocol on.
- `Auto(flags)`: when interactive and stdin and stdout are TTYs, write `\x1b[?u` and remember the flags with the deadline `now + 200ms`.
- `Disabled`: nothing.

`App::input(bytes, now)`: when the runtime reports a query response, a query is pending, and `now` is before its deadline, write `\x1b[>{flags}u` and mark the protocol on. Clear the pending query on a response or once its deadline has passed; a later response does nothing (Ink's 200 ms timer calls `finish(false)`). Check the deadline when the response arrives; do not add it to the wake time. The response is read only while raw mode is on, as in Ink, where stdin is read only in raw mode. After unmount, `App::input` already does nothing (KK-68).

Write each mode sequence (`\x1b[>{flags}u`, `\x1b[?u`) as its own `Terminal::write`, with the restore bytes for the new state.

### 1.5 Restore

`AppOutput::restore_bytes` and `AppOutput::restore` start with `\x1b[?2004l` when paste mode is on, then `\x1b[<u` when the protocol is on, then the bytes they write today. This holds for the alternate screen, the main screen and non-interactive mode, and for the panicking branch of `unmount`. Ink's order is: the App unmount effect writes `?2004l`, then `finishUnmount` writes `<u`, then `log.done()` or the alternate-screen exit, or the non-interactive final frame. Keep the mode state where `restore_bytes` can read it.

## 2. Tests

Port, names and order as in Ink, through `testing::mount` and `TestTerminal` (interactive TTY, 100 columns, unless the Ink test says otherwise). New files get the header `//! Ported from ink test/<file> @ 26d2c3f.`. Append to an existing file at Ink's position. Byte checks use `TestTerminal::writes()`; where Ink checks that one write equals a sequence and the ink-rs write carries more bytes (the restore), check that a write contains it and say so in a comment.

1. `test/hooks-use-paste.tsx` → `tests/hooks_use_paste.rs`, 4 tests, fixture `test/fixtures/use-paste.tsx`. Port the fixture as a Rust component with the same branches: the paste handler exits on a match and does nothing otherwise; the `noUseInput` `use_input` panics. Assert `exit_result()` is `Some(Ok(None))`. `receives bracketed paste as single text blob` also checks the writes contain `\x1b[?2004h` and, after the exit, `\x1b[?2004l`.
2. `test/kitty-negotiation.tsx` → `tests/kitty_negotiation.rs`:
   - `associated text requests include all-key reporting`: there is no public resolve function, so mount with `Enabled(KittyFlags::REPORT_ASSOCIATED_TEXT)` and check `\x1b[>24u`, and with `Enabled(DISAMBIGUATE_ESCAPE_CODES | REPORT_ASSOCIATED_TEXT)` and check `\x1b[>25u`.
   - `auto detection consumes responses without duplicating input`.
   - `pasted query responses remain input and do not enable the protocol`.
   - Not ported: `suspension cancels pending keyboard negotiation` and `unmount while suspended does not pop the primary keyboard stack` need suspend (M4).
3. `test/kitty-keyboard.tsx` → `tests/kitty_keyboard.rs`, the app-level tests from line 589 to 775 except `auto detection handles Uint8Array query response` (Rust input is bytes in every case): 11 tests.
   - The `createFakeStdout`/`createFakeStdin` tests use `Enabled(KittyFlags::default())` and a `Text`; stdin and stdout TTY flags come from `TestTerminal::with_stdin_tty` and `with_tty`.
   - `createDetectionApp` becomes a helper: `Auto(KittyFlags::default())`, interactive, a `use_input` that records inputs.
   - `auto detection does not enable protocol after unmount`: `TestApp::unmount` consumes the app, so end the app with an exit request from a handle the test retains (carry it out through the props, as `tests/hooks.rs` does with a thread-local), then feed the late response with `input`; check that no write contains `\x1b[>1u`.
   - `auto detection handles synchronous query response`: Ink pushes the response before App attaches its reader; ink-rs reads it once raw mode is on, so feed it after `mount`. Say so in a comment.
   - Ink's `delay(n)` becomes `advance(Duration::from_millis(n))`.
4. `test/input-buffered-ctrl-c.tsx` → `tests/input_buffered_ctrl_c.rs`: the two cases of `buffered Ctrl+C respects ${…}` after the existing three. `exitOnCtrlC: isBracketedPaste` maps to `Options::exit_on_ctrl_c`; `isExited` is `exit_result().is_some()`.
5. `test/rerender-input.tsx` → `tests/rerender_input.rs`: `rerender() keeps a bracketed paste that spans two reads`, between the existing two tests and Ink's last test, which is not ported.

No other tests. Existing tests must keep passing unchanged, apart from `App::new` call sites.

## Rules

- No `unsafe`. No new dependencies.
- No code that exists only to satisfy a test.
- Match the surrounding code: `pub(crate)` or narrower for internal items, imports at the top, comments that say why and cite the Ink source where the code ports it. Run `rustfmt --edition 2024` on every file you touch.
- Do not edit `docs/`. Report the documentation changes instead (below).
- Run only the following. Every test in these targets must pass.
  - `cargo test --test hooks_use_paste --test kitty_negotiation --test kitty_keyboard --test input_buffered_ctrl_c --test rerender_input --test hooks_use_input --test exit_keyboard --test components --test cursor_exit_position`
  - `cargo test --lib runtime`
  - `cargo test --lib input`
  - `cargo test --lib driver`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

1. Proposed commits: message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. The public API added, with its rustdoc; the internal changes (the shared registration, the holder count, the paste route, the response matcher, App's paste-mode and kitty state, the restore bytes).
3. Where the behaviour departs from Ink, with the reason.
4. The ported tests: names, files, and every assertion adapted, with the reason.
5. Results of the commands above. Name any failing test, with your diagnosis.
6. The rows to change in `docs/ink-tests.md` (Ink file, ported count, Rust file), the entries to move out of the M3 lists, and anything design.md should record. Open questions.
