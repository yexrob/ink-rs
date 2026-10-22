# Task: terminal graphics primitives in ink-rs

## Goal

eink-rs, a downstream crate, will show images in its Markdown component with the kitty graphics protocol's Unicode placeholders. The image data goes to the terminal once; the image appears where text cells hold `U+10EEEE` plus row and column diacritics, with the image id in the foreground colour. ink-rs can already paint those cells as ordinary `Text`. It lacks three primitives. Add them:

1. Input parsing of terminal string replies (APC, DCS, OSC, SOS, PM).
2. `Scope::use_graphics()`, a lazy probe for kitty graphics and the cell size in pixels.
3. `OutputWriter::write_passthrough(bytes)`, which wraps bytes for tmux.

Image encoding, image ids and placeholders stay in eink-rs. ink-rs knows nothing about images.

## Read first

- `<repo>/CLAUDE.md`.
- `<repo>/docs/decisions/069-terminal-graphics.md`: the plan and the user's choices.
- `<repo>/docs/design.md`: 「终端输出」 and 「输入」.
- `<repo>/src/input/parser.rs` and its tests; `<repo>/src/input/kitty.rs` (`is_query_response`).
- `<repo>/src/runtime/input.rs`: `Runtime::input`, and how query replies and unmapped terminal reports are filtered today.
- `<repo>/src/app.rs`:
  - `App::new`: the kitty keyboard query (`kitty_query`), the model for sending a query and waiting for a reply with a deadline.
  - `render_if_due`: raw mode, the control flush, the returned wake time.
  - `input`, `resize`, and the suspend paths.
- `<repo>/src/runtime/hooks.rs` (`use_window_size`) and `<repo>/src/runtime/mod.rs` (`invalidate_window_size`, `resize`): the model for a hook whose readers are scheduled when a runtime value changes.
- `<repo>/src/runtime/output.rs`: `OutputWriter::write_control`, `WriteDestination`.
- `<repo>/src/terminal.rs`; `<repo>/src/driver/mod.rs` (`mount` and its `env` closure); `<repo>/src/driver/environment.rs`; `size()` in `<repo>/src/driver/sys/unix.rs` and `<repo>/src/driver/sys/windows.rs`; `<repo>/src/driver/process.rs`.
- `<repo>/src/testing.rs` (`TestTerminal`, `mount`, `TestApp::input`, `TestApp::advance`) and `<repo>/tests/kitty_negotiation.rs`: how a query and its reply are tested.

## Protocol facts

- **kitty graphics query**: `ESC _ G i=31,s=1,v=1,a=q,t=d,f=24;AAAA ESC \`.
  - A terminal with kitty graphics replies `ESC _ G i=31;OK ESC \`, or `ESC _ G i=31;<CODE>:<message> ESC \` on an error.
  - Terminals without it ignore the query.
- **`CSI 16 t`** (XTWINOPS): the reply is `CSI 6 ; <height> ; <width> t`, one cell in pixels.
- **DA1 `CSI c`**: every terminal answers `CSI ? <params> c`. Terminals answer queries in order, so DA1, sent last, marks the end of the replies to the earlier queries. Replies can arrive split across reads.
- **tmux passthrough**: `ESC P tmux;`, then the bytes with each `ESC` doubled, then `ESC \`.
  - It needs `allow-passthrough on`; otherwise tmux drops it.
  - tmux answers DA1 and `CSI 16 t` itself. DA1 must be wrapped to reach the outer terminal. `CSI 16 t` stays unwrapped: tmux knows its cell size.
  - tmux sets `TMUX` in the environment of its panes.
- **herdr**, another multiplexer, needs no special code:
  - it answers the kitty query, DA1 and `CSI 16 t` itself;
  - it fills the `TIOCGWINSZ` pixel fields;
  - it drops tmux passthrough and removes `TMUX` from its panes.

## Change 1: string sequences in the input parser

- The parser recognises these string sequences, terminated by ST (`ESC \`) or BEL (0x07):
  - `ESC P` (DCS), `ESC ]` (OSC), `ESC _` (APC), `ESC X` (SOS), `ESC ^` (PM).
- A complete string sequence becomes its own event kind, not `InputEvent::Text`. It never reaches key decoding, Ctrl+C handling, focus, paste handlers or `use_input`.
- An unterminated one waits for more bytes, like an incomplete CSI.
- Alt+P, Alt+], Alt+_, Alt+X and Alt+^ send these same introducers, and keys must keep working:
  - When the existing 20 ms escape timeout fires on an unterminated string sequence, the parser emits what it emits today: `ESC` plus the introducer as one event, then the rest parsed as ordinary input.
  - Choose a payload rule that keeps a stray introducer from swallowing input. Reply payloads are printable: kitty's `Gi=31;OK`, DECRQSS, OSC colour replies, OSC 52 base64. A byte that cannot appear in a reply payload, such as a C0 control other than the terminators, ends the attempt, and the input parses as it does today.
  - Say in the report which rule you chose.
- Bracketed paste does not change: paste content stays raw.
- Outside a probe, complete string sequences are dropped. CSI replies (DA1, `CSI 6;…t`) keep today's handling outside a probe: Ink drops unmapped terminal reports.

## Change 2: `Scope::use_graphics()`

The public API. Better names are welcome; say why in the report.

```rust
/// What the terminal can draw beyond text, from [`Scope::use_graphics`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Graphics {
    pub support: GraphicsSupport,
    /// One cell's size in pixels, width then height, when known.
    pub cell_pixels: Option<(u16, u16)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsSupport {
    /// The probe has not finished.
    Probing,
    /// The kitty graphics protocol with Unicode placeholders.
    Kitty,
    /// No supported image protocol.
    Unsupported,
}
```

**Starting the probe**

- The first `use_graphics` call in an app starts the probe. Later calls, in any component, return the current value. The probe runs once per app.
- A component that called the hook is scheduled when the value changes, as `use_window_size` readers are on resize.
- Hooks cannot write during render (`assert_not_rendering`). The hook records the request. The app sends the queries in the same step, after the commit, through the terminal as it sends the kitty keyboard query, and outside the synchronized frame.

**Apps that cannot probe** return `Unsupported` from the first call and send nothing:

- non-interactive apps, and apps whose stdout or stdin is not a TTY;
- `testing::render`, `testing::render_with` and `render_to_string`;
- terminals that answer the kitty query but lack placeholders (record 069): `TERM_PROGRAM=WezTerm`, or `KONSOLE_VERSION` set.

Read the environment where the driver reads it today (the `env` closure in `driver::mount`), not inside the runtime. `testing::mount` must not read the process environment: tests run inside tmux on this machine.

**Queries**

- Send, in this order: the kitty query, `CSI 16 t`, DA1.
- Inside tmux, wrap the kitty query and DA1 with Change 3's wrapping. `CSI 16 t` stays unwrapped.

**Raw mode and input**

- Today `raw_mode` is `stdin_tty && holders > 0`. While probing, raw mode is on, and input is read and parsed with or without input hooks. After the probe, raw mode follows the input hooks again.
- Input during the probe that is not a reply goes to the handlers as it does today. Say in the report what happens to it when there is no handler.
- Replies during the probe never reach `use_input`.

**Replies**

| Reply | Effect |
|---|---|
| kitty reply with `i=31` and payload `OK` | kitty seen |
| `CSI 6 ; h ; w t` | cell size, stored as `(w, h)` |
| DA1 reply | the probe ends |

- The probe ends as `Kitty` if it saw the `OK`, otherwise as `Unsupported`.
- Timeout: 1 s after the queries go out, the probe ends with what it has seen. The app loop must wake for the deadline; see how `render_if_due` returns the next wake time.
- A terminal handover during the probe (`suspend_terminal`, Ctrl+Z, SIGTSTP) ends it as `Unsupported`.
- Kitty keyboard negotiation (`KittyKeyboard::Auto`) and this probe can overlap. Both must work.

**Cell size**

- The Unix driver reads `ws_xpixel` and `ws_ypixel` from the `tcgetwinsize` call that `size()` makes, and divides them by the columns and rows. Zero means unknown.
- Windows and `TestTerminal` report it as unknown. Add no FFI.
- A size the driver knows wins over `CSI 16 t`. On resize the driver reads it again, and readers are scheduled when it changed. When the driver does not know it, the `CSI 16 t` reply is used.
- `cell_pixels` does not depend on `support`: it can be `Some` with `Unsupported`.

## Change 3: `OutputWriter::write_passthrough`

- `write_passthrough(bytes)` queues bytes meant for the outer terminal, with `write_control`'s timing and rules.
- When `TMUX` was set at mount, each call is wrapped as one passthrough sequence: `ESC P tmux;`, the bytes with each `ESC` doubled, `ESC \`. Elsewhere it behaves as `write_control`.
- ink-rs does not chunk. Callers send one command per call; eink-rs splits kitty data into chunks of 4096 bytes. The doc comment says so and names the `allow-passthrough on` requirement.
- The probe uses the same wrapping function.

## Tests

Add tests for the new behaviour, and give the reason for each in the report. The coverage needed:

- **Parser**:
  - APC, DCS and OSC, each with ST and with BEL;
  - a reply split across reads;
  - Alt+P and Alt+_ alone, and each followed by a letter in a later read after the escape timeout;
  - a string sequence inside a bracketed paste.
- **Probe**, through `testing::mount` and `TestTerminal`:
  - the queries in `writes()`;
  - `OK`, the cell size and DA1 → `Kitty` with the cell size, and the reader re-rendered;
  - DA1 alone → `Unsupported`;
  - the timeout, through `advance`;
  - raw mode on during the probe and off after it with no input hook (`raw_mode_calls`);
  - replies never reach `use_input`;
  - a non-interactive app → `Unsupported` with no writes.
- **tmux wrapping**: a unit test of the wrapping function, including `ESC` doubling.

Add nothing beyond what these behaviours need.

## Out of scope

- Image encoding, image ids, placeholders, loaders: eink-rs.
- Other protocols (Sixel, iTerm2) and half blocks.
- File, temp-file and shared-memory transmission; video.
- Probing again, and a public API for arbitrary queries.
- README, `docs/`, examples, CHANGELOG, the version number.

## Rules

- `CLAUDE.md` holds. No `unsafe`. The Windows console FFI exemption covers the existing code in `sys/windows.rs`.
- Behaviour outside these features does not change; the existing tests are the check.
- Code reads like the existing code: naming, comment density, idiom. Comments that cite Ink stay accurate.
- New public items get doc comments in the style of `use_window_size` and `write_control`.
- **The sandbox cannot write `.git`.** Do not commit. Do not write report files into the repository.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`
- `cargo test --offline --lib`
- `cargo test --offline --test kitty_negotiation --test kitty_keyboard --test hooks_use_input --test hooks_use_input_kitty --test hooks_use_paste --test input_buffered_ctrl_c --test exit_keyboard --test rerender_input --test suspension_input_disable --test suspend_terminal --test terminal_resize --test runtime --test hooks --test focus`, plus any test file you add.
- Run only these; the main agent runs the full suite and the manual tests in tmux and herdr.

## Report

Your final message is the report, in Chinese, without adverbs. It covers:

- the files you changed, and what each change does;
- the event kind for string sequences, the payload rule, and the timeout fallback;
- where the probe state lives, how a request reaches the app, and how readers are scheduled;
- the raw mode rule, and what happens to non-reply input during the probe;
- how the driver reports the cell size, and how a resize updates it;
- every deviation from this brief, and why;
- the tests you added, and why each is needed;
- the exact commands you ran and their results.
