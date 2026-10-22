# Task: mouse input in the alternate screen

## Goal

bingo wants clicks, drags, the wheel and hover in its full-screen TUI. The requirement from bingo is `<repo>/docs/decisions/briefs/bingo-mouse.md`. Research settled the design below; the user chose the alternate screen only, with a hover level.

## Read first

- `<repo>/CLAUDE.md`.
- `<repo>/docs/decisions/075-mouse.md`: the plan and the user's choices.
- `<repo>/docs/design.md`: 「输入」 and the terminal restore.
- `<repo>/src/input/parser.rs` and its tests: `escape_sequence_len`, `control_sequence`, `has_pending_escape`, `flush_pending_escape`, `InputEvent`.
- `<repo>/src/input/mod.rs`: the public input types.
- `<repo>/src/runtime/input.rs`: `Runtime::input`, the order of Ctrl+C, Ctrl+Z, focus and handlers, how unmapped sequences are dropped, and the 20 ms escape deadline.
- `<repo>/src/runtime/hooks.rs`: `use_input`, `use_paste`, and how a hook becomes a raw-mode holder.
- `<repo>/src/runtime/state.rs`: `ElementMetrics`, `measure_element`.
- `<repo>/src/app.rs`:
  - the paste mode: `paste_mode`, `set_paste_mode`, `restore_modes`, `restore_bytes`;
  - `suspend` and `resume`;
  - the end of `step`, where modes follow the holder counts;
  - `paint` and the alternate-screen merge of `static_rows` into the live rows;
  - `input`.
- `<repo>/src/runtime/output.rs` (`Output::render`, how rows above the screen are cut) and `<repo>/src/testing.rs` (`TestTerminal`, `mount`).
- `<repo>/src/driver/sys/windows.rs`: `enter_raw_mode` and the reader thread's record filter.

## Parsing

- A new `InputEvent` kind for mouse reports. A sequence that starts `ESC [ <` and matches `<digits;digits;digits` ending in `M` or `m` becomes a mouse event; anything else keeps today's path.
- SGR's button byte `b`: the low two bits are the button (0 left, 1 middle, 2 right, 3 none), +4 shift, +8 meta, +16 ctrl, +32 motion, +64 wheel (64 up, 65 down, 66 left, 67 right). `M` is press or motion, `m` is release. Coordinates are one-based.
- A report split across reads waits in the existing pending buffer.
- When the 20 ms escape timeout fires on a fragment that starts `ESC [ <`, the fragment is dropped; it never reaches `use_input` as keys.
- X10 (`ESC [ M` plus three bytes) and 1015 encodings are not requested and not parsed. Say in the doc comment that ink-rs asks for SGR only.

## Public types

```rust
pub enum MouseButton { Left, Middle, Right }

pub enum MouseKind {
    Press(MouseButton),
    Release(MouseButton),
    Drag(MouseButton),
    /// Pointer motion with no button held; needs `MouseTracking::Motion`.
    Move,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

pub struct MouseEvent {
    pub kind: MouseKind,
    /// Cells from the live frame's top-left, the origin of `ElementMetrics`.
    /// `y` is negative over Static rows replayed above the live frame.
    pub x: i32,
    pub y: i32,
    pub ctrl: bool,
    pub shift: bool,
    pub meta: bool,
}

/// How much the terminal reports. Each level includes the ones before it.
pub enum MouseTracking {
    /// Press, release and the wheel (mode 1000).
    Click,
    /// Adds motion while a button is held (mode 1002).
    Drag,
    /// Adds motion with no button held, for hover (mode 1003).
    Motion,
}
```

Derive what the existing input types derive. `SGR` release reports do not name the button on some terminals (`b` low bits 3); map that case and say how.

## Hook and hit testing

```rust
impl Scope {
    /// Receives mouse events while `tracking` is `Some`. Mouse reporting is on
    /// while any app hook is active, at the highest level asked for.
    pub fn use_mouse(&mut self, tracking: Option<MouseTracking>, handler: impl FnMut(MouseEvent) + 'static);
}

impl ElementMetrics {
    /// Whether the cell `(x, y)` lies inside this box.
    pub fn contains(&self, x: i32, y: i32) -> bool;
}
```

- Handlers run in registration order, as `use_input` handlers do. Mouse events skip Ctrl+C, Ctrl+Z and focus handling, and never reach `use_input`.
- An active mouse hook counts as a raw-mode holder.
- Hit testing is `measure_element(node).contains(ev.x, ev.y)` in the handler. The library does not route by node or move focus on click; the doc comment shows the pattern. It ignores overflow clipping and overlap; say so.
- When stdin is not a TTY, the hook does nothing and reports no error, as `use_focus` does. Find where `use_input` reports `Unsupported` and exempt mouse holders the way focus holders are exempt.

## Modes

- `AppOutput` gets a mouse mode beside the paste mode. At the end of `step`, the mode is the highest level among active hooks, on only when raw mode is on, the app uses the alternate screen, and stdout is a TTY.
- Turning it on writes `ESC[?1006h` and the level's mode (`?1000h`, `?1002h` or `?1003h`). A level change turns the old level off before the new one goes on. Turning it off writes the level off and `ESC[?1006l`.
- The restore snapshot (`restore_modes`, `restore_bytes`) carries the off bytes, so exit, unmount, panic and termination turn reporting off, as they do for bracketed paste.
- `suspend` turns it off and clears the state; the next step after `resume` turns it on again, as the paste path does.
- Inline mode never turns reporting on, and `use_mouse` receives nothing there. The doc comment says why: the live frame's screen row moves with Static output, scrolling, external writes and resize, and ink-rs cannot know it.
- The doc comment also says that reporting takes the terminal's text selection away (most terminals bypass with Shift+drag, iTerm2 with Option+drag) and that it is off until a hook asks for it.

## Coordinates

- In the alternate screen, frame row 0 is screen row 0. `paint` places up to one screen of Static history above the live rows, and the output cuts rows that do not fit. Record, at each paint, the screen row where the live frame starts: the replayed Static rows minus the rows cut from the top.
- An event's `y` is the screen row minus that origin; `x` is the screen column. Both are zero-based. Between paints, use the origin of the last paint.
- `App::input` gives the origin to the runtime before it dispatches.

## Windows

`ENABLE_VIRTUAL_TERMINAL_INPUT` is on, so after `?1000h` and `?1006h` Windows Terminal sends mouse reports as VT sequences in key records, which the reader thread passes on. Do not change `windows.rs`. The doc comment says Windows Terminal is supported and older conhost is unverified.

## Tests

Each covers new logic:

- **Parser**: press, release, drag, wheel and modifiers, one case each (the button-byte mapping); a report split across two reads gives one event (the pending path); a fragment dropped after the escape timeout never reaches `use_input` (no leaked keys).
- **Mount**, alternate screen, through `testing::mount` and `TestTerminal`:
  - a click inside a box's rect reaches the handler with live-frame coordinates, and `contains` is true; a click outside gives false;
  - with Static history replayed above, coordinates shift by the origin;
  - mode bytes: on writes `?1006h` and the level, raising the level to `Motion` switches modes, unmount and suspend write the off bytes;
  - inline mode and a non-TTY stdout write no mode bytes.

Add nothing beyond these.

## Out of scope

`docs/`, README, CHANGELOG, examples, the version number. Focus on click, routing by node, drag-and-drop helpers.

## Rules

- `CLAUDE.md` holds. No `unsafe`.
- Behaviour outside mouse input does not change; the existing tests are the check.
- Code reads like the existing code: naming, comment density, idiom. Comments that cite Ink stay accurate.
- New public items get doc comments in the style of `use_paste` and `use_input`.
- No new dependencies.
- **The sandbox cannot write `.git`.** Do not commit. Do not write report files into the repository.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings`
- `cargo test --offline --lib`
- `cargo test --offline --test hooks_use_input --test hooks_use_paste --test input_buffered_ctrl_c --test suspend_terminal --test suspension_input_disable --test terminal_resize --test runtime --test hooks --test focus`, plus any test file you add.
- Run only these; the main agent runs the full suite and the manual checks.

## Report

Your final message is the report, in Chinese, without adverbs. It covers:

- the files you changed, and what each change does;
- the event kind, the button-byte mapping, the release case, and the timeout rule;
- how events reach handlers, and the raw-mode and non-TTY rules;
- the mode state, the level rule, and the restore and suspend paths;
- the origin: where it is computed and how the runtime gets it;
- every deviation from this brief, and why;
- the tests you added, and why each is needed;
- the exact commands you ran and their results.
