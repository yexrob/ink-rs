# M2 remaining phases: design notes (main agent)

Order: P2a (running) → P2b (brief written) → P3a → P3b → P2c (needs user decisions).

## P3a: use_cursor + external writes
- `Scope::use_cursor(Option<pos>)`, declared during render. Ink use-cursor.ts: insertion effect without deps,
  so the position applies at each commit in which the component renders; unmount → undefined.
  Memo'd component keeps its position (CU2). Integer cells; Codex picks the type and explains.
- Cursor change is part of the frame: throttled like it; unchanged frame + cursor → no bytes (R117).
- `use_stdout()` / `use_stderr()` → Copy handles (like State, via the registry) with `write(&str)`.
  - Write during render panics in debug, as state writes do.
  - Queue drained at the end of the step (bytes order equals Ink: frame N, then data + redraw N).
  - Interactive stdout: one write: [bsu] Output::clear + data + redraw last frame (with cursor) + [esu].
    `\n` → `\r\n` (crossterm raw mode clears OPOST; libuv keeps ONLCR); add `\r\n` when data lacks a trailing newline
    (Output assumes column 0 after forget). Deviation from Ink: record in design.md.
  - Interactive stderr: stdout [bsu]+clear, stderr data, stdout redraw+[esu] (Ink writeToStderr).
  - Non-interactive: data as-is to its stream.
  - testing::render (Ink debug): stdout write → frame entry `data + static + live`; stderr → stderr list + entry `static + live`.
  - Writes after unmount are dropped.
  - Terminal trait gains stderr; TestTerminal records it.
- Ports: cursor.tsx CU2, CU10-CU17, CU19 (height-cap adaptation); CU3/CU18/CU20 dup-incr; CU1, CU4-7, CU9 M3; CU8 不适用 (Suspense).
  cursor-exit-position CX1, CX2 (Output::done twice, src/output/tests), CX3; CX4-6 dup-incr.
  cursor-helpers.tsx (17) and line-update.ts (29): 不适用 (Ink internals; cell diff).
- Update `Scope::use_effect` doc already done in P2b.

## P3b: resize + use_window_size
- `App::resize(columns, rows)`; TestApp::resize. Ink `resized` (ink.tsx:262-285) + design.md 终端输出 resize:
  width decrease → Output::clear + redraw; height decrease with a full previous frame → `CSI 1;1H CSI J` or
  cursor-relative erase (ink.tsx:771-781); else diff. Resize bypasses the throttle. Root width relayout.
- `use_window_size()` → (columns, rows), re-renders on change.
- Ports: terminal-resize TR1-TR19 except S* (TR11-14, TR16, TR17: vt100 has no row-shrink model → 手动验收 list),
  TR20-28 dup-incr; render.tsx R83 (adapt listener count), R54 / R82 不适用.
- design.md resize paragraph: say what was built.

## P2c: sync driver
User decisions (2026-10-20): 1 render + wait_until_exit; 2 option a (end when idle);
3 State Send + AppHandle::update closures; 4 crossterm + signal-hook, own color detection following chalk's supports-color.
Questions as asked:
1. Entry: `ink_rs::render(element, Options) -> Instance` mounts and writes the first frame;
   `Instance::wait_until_exit(self)` runs the loop on this thread; `rerender`, `clear`, `unmount`.
2. Natural end of the loop (Node exits when its loop is empty; Ink C12 relies on it):
   a. return when no work, no live AppHandle, no input subscription (M3), no timer (M4) (recommended);
   b. only on exit() or a signal.
3. Cross-thread: State becomes Send (ids only; use off the UI thread panics, design.md 编程模型);
   `AppHandle::update(move || count.set(..))` runs Send closures on the UI thread (recommended);
   alternative `AppHandle<M>` messages.
4. Dependencies: crossterm (size, TTY, M3 input), signal-hook (iterator API, no unsafe), color detection:
   own function following supports-color, or the `supports-color` crate.
- Panic hook installed once, chains the previous hook, writes restore bytes first (show cursor, leave alt screen),
  shared "restored" flag with Drop. Signals: unmount on the UI thread, restore, emulate the default handler.
- CI: is-in-ci (`CI` or `CONTINUOUS_INTEGRATION` present and not "0"/"false").
- Ports: components C12, C13.
