# ink-rs M2 phase 2b: the app layer

You are working in `<repo>`. Read `CLAUDE.md`, then all of `docs/design.md` (in particular 分层, 跨层约定, 编程模型 调度 and effect, 一帧, 事件循环, 终端输出, 测试), then `docs/ink-tests.md`.

Committed so far:
- `src/runtime/` mounts components, keeps a tree mounted across frames and runs effects in React's order. Since M2 phase 2a, each Static instance renders only its unwritten items and unmounts them after the write (`Instance::pending_static`, `finish_static`), and `Frame::static_changed` reports that the first Static differs from the one at the previous present.
- `src/output.rs` turns frames into terminal bytes: cell diff, height cap, Static rows above the frame, cursor, `clear`, `done`.
- `src/testing.rs`: `testing::render` stands in for Ink's debug mode. It is unthrottled and records one string per frame.

Nothing connects the runtime to a terminal yet. This phase adds that layer, the stepping core of design.md 事件循环, with an in-memory terminal. Read the code above before you change it.

References, all read only:
- The Ink checkout at `<scratchpad>/ink-src/ink`. Read it; never run anything from it. The relevant sources:
  - `src/ink.tsx`: the constructor (throttles), `onRender`, `renderFrame`, `renderInteractiveFrame`, `setAlternateScreen`, `resolveInteractiveOption`, `resolveAlternateScreenOption`, `shouldSync`, `unmount`, `finishUnmount`, `clear`.
  - `src/render.ts`: options and `cleanup`.
  - `src/write-synchronized.ts`.
- `<scratchpad>/m1-research/runtime.md` §3.2-3.10: render loop, throttle, unmount.
- `<scratchpad>/m2-research/tests-render.md`: every test of `render.tsx`, `clear-rerender.tsx`, `issue-973-static-commit.tsx`, `static-runtime-blank-lines.tsx`, `write-synchronized.tsx`, `render-callback.tsx`, the M2 tests of `components.tsx` and `measure-element.tsx`, with the Ink facts they rely on and their conflicts with design.md.
- `<scratchpad>/m2-research/tests-output.md` §2.2 (LC8) and §2.4 (LB4).
- `<scratchpad>/expectations/`: values recorded from a run of Ink's tests, one file per Ink test file. Expected values come from there. Ink's test fixtures are under `test/fixtures/` of the checkout.

## 1. What to build

### `Terminal` (new `src/terminal.rs`)

The bottom layer of design.md 分层. A `pub(crate)` trait:
- write bytes to stdout;
- the size as `(columns, rows)`, or `None` when unknown;
- whether stdout is a TTY.

Writes are infallible at this layer; the crossterm implementation in a later phase decides what a failed write means. Stderr, raw mode and resize events come in later phases. Do not add them.

### `Options` (public, exported at the crate root)

The options of a live app. Field meanings follow Ink's `render` options (`src/render.ts`, `src/ink.tsx`):
- `max_fps: u32`, default 30. The throttle window is `ceil(1000 / max_fps)` ms, 34 ms at 30. Ink treats a non-positive value as a 0 ms window; do the same for 0, and say so in the rustdoc.
- `interactive: Option<bool>`, default `None`. `None` resolves to "stdout is a TTY and not in CI".
- `alternate_screen: bool`, default false. It takes effect only when the app is interactive and stdout is a TTY (`resolveAlternateScreenOption`).
- `color_level: Option<ColorLevel>`, default `None`. `None` means detected from the environment.

Detection of CI and of the color level reads the process environment. It belongs to the driver of a later phase, not to this one. The core receives resolved values: put the resolution of `interactive` in a function that takes the option, an `in_ci` flag and the TTY flag.

### The app (new `src/app.rs`)

A `pub(crate)` type that owns the `Runtime`, the `Output`, a `Box<dyn Terminal>`, the throttle state and the resolved modes. It is the core that design.md 事件循环 describes: `render_if_due(now) -> Option<Instant>`, plus `rerender`, `clear` and `unmount`.

Size: read the terminal size once at mount, with 80×24 when it is unknown (design.md 终端输出, Ink `getWindowSize`). The root width is the column count. Resize comes in a later phase.

#### Steps and the throttle

design.md 编程模型 调度 is the rule: commits are not throttled; paint and the write are. The first paint in a window writes at once; later changes in the window are painted at the end of the window.
- **Commit.** A step commits when the runtime has work. A committed but unpainted frame is pending.
- **Throttled paint.** A pending frame is painted at once when no throttled paint happened in the last window, and otherwise at the end of the window that the last throttled paint opened. A throttled paint opens a new window.
- **Paint at once.** A commit that renders new Static items or changes the Static identity paints in the same step, whatever the window (Ink `onImmediateRender`). It satisfies a pending frame. It does not open a window: Ink's immediate render bypasses the throttle without touching it. The runtime has to report this from the commit; build it on what phase 2a tracks. Phase 2a compares the Static identity in `present`; move the comparison so that the commit can report it.
- **Effects.** Passive effects follow the commit (design.md 编程模型 effect): after the write when the frame is painted in the same step, right after the commit when the frame is deferred. Phase 2a's rule stays: paint, write, the commit's passive phase, then the Static item unmount.
- **Return value.** `Some(now)` when the runtime still has work; otherwise the end of the window when a frame is pending; otherwise `None`. With nothing to do, a step renders nothing and writes nothing.

Split `Runtime::render_if_due` into the steps the app needs (commit, paint, the after-write phase, a work query). Keep `Runtime::render_if_due` as their unthrottled composition, or replace it, as long as `render_to_string` and `testing::render` keep their behavior. The runtime knows nothing about time or throttling.

#### Interactive output

- Frames go through `Output`.
- **The height cap.** It applies on a TTY. When stdout is not a TTY, the frame is not capped and never counts as full: Ink's `renderInteractiveFrame` treats piped output as never fullscreen.
- **Synchronized output** (design.md 终端输出 同步输出, Ink `shouldSynchronize`). It is used when the app is interactive and stdout is a TTY.
  - Each operation is one terminal write with the bsu/esu pair inside the same buffer. An operation is a frame, a `clear()`, or the unmount sequence.
  - An operation that produces no bytes writes nothing, the pair included.
  - Move the pair out of `Output` into the app, so that `Output::new` loses its flag. The output tests that run with the flag on keep their assertions.
- **The alternate screen.**
  - Enter: `CSI ?1049h` followed by `CSI H`, so the frame starts at the first row (design.md 终端输出 全屏模式). Write it before the first frame.
  - Leave at unmount: `CSI ?1049l` then `CSI ?25h`, with nothing after them (Ink `finishUnmount`).
  - Static rows have no scrollback to go to. The app keeps every Static row written so far, as Ink's `fullStaticOutput`, and resets them when the Static identity changes. Each frame hands `Output` the newest Static rows that can be visible, at most the row count, followed by the live rows, with no new Static rows. The height cap then shows the tail.
  - The result is a replay without a full clear: write only what changed. Do not copy all Static rows on each frame.
- **`clear()`** erases the frame (`Output::clear`) and does nothing when the app is not interactive (Ink `clear`). The next paint redraws the whole frame. A pending frame stays pending.

#### Non-interactive output

design.md 终端输出 非 TTY / CI and Ink `renderFrame`:
- No frames are drawn, and there is no throttle.
- A commit that renders new Static items paints and writes those rows at once, each row followed by `\n`.
- Unmount writes the final frame, its rows joined by `\n`, then `\n`.
- Rows have the content of `paint::serialize_row`: styled, no trailing blanks.
- Paint only when there is something to write; a commit without new Static items costs no paint.

#### Unmount

`unmount` and the app's `Drop` share one sequence; a second call does nothing.
1. Finish the current state, ignoring the window: commit pending work, then paint and write a pending frame (Ink `settleThrottle`). Write nothing when nothing is pending, so `clear()` followed by unmount leaves the terminal clean (Ink `onRender`, "Keep it while unmounting").
2. Unmount the tree with its cleanups. Nothing is written after this point except step 3.
3. Interactive on the primary screen: `Output::done`. On the alternate screen: leave it. Not interactive: the final frame as above.

`Drop` while the thread is panicking skips step 1 and paints nothing: component code may panic again. It still restores the terminal in step 3.

### `testing`

- **`TestTerminal`**, the in-memory `Terminal`. `Clone`, with shared contents, so the test keeps a handle while the app owns another.
  - `new(columns, rows)` is a TTY; offer a way to make a non-TTY one.
  - It records each write as one entry, readable as strings.
- **`mount(element, Options, &TestTerminal) -> TestApp`** runs the first step. It resolves `interactive` with `in_ci = false`: Ink's own test run sets `CI=false`. `color_level: None` resolves to `ColorLevel::Basic`, as Ink's test helpers run with `FORCE_COLOR`.
- **`TestApp`** holds a fake clock that starts at mount:
  - `rerender(element)` runs a step at the current time, like Ink's synchronous `rerender`;
  - `advance(Duration)` runs every step that falls due up to the new time, in order, like sinon's `clock.tick`. It begins with a step at the current time, so state written from the test body is picked up;
  - `clear()` and `unmount(self)`. Dropping it unmounts.
- Keep `testing::render` as it is.
- Put the vt100 helpers the new tests share in `tests/common`: a screen with scrollback fed from a `TestTerminal`.

Update the rustdoc of `Scope::use_effect`: it runs after the commit, and after the frame is written when that frame is not throttled. Mention `Options` and `testing::mount` in the crate docs in `src/lib.rs`.

### Not in this phase

Do not build any of these:
- resize, `use_window_size`, `use_cursor`;
- `use_stdout` and `use_stderr`;
- the sync driver, crossterm, signals, the panic hook;
- CI and color detection;
- `exit` and `AppHandle`.

## 2. Tests to port

Use Ink's names, converted as in the existing files, in Ink's order. IDs below are from `tests-render.md` and `tests-output.md`. Do not write them in code or docs.

### Mapping Ink's tests

- **Streams.**
  - `createStdout(columns)` with `isTTY` true is a `TestTerminal` TTY. When the test does not set `rows`, use 24, the fallback of Ink's `getWindowSize`.
  - `isTTY: false` is a non-TTY terminal.
  - A PTY fixture (`term`, `run`) becomes an in-process test: rebuild the fixture's component tree, with the rows Ink saw (see Artifact in `tests-render.md`).
- **Timers.** A fixture `setInterval` or `setTimeout` that updates state becomes a `State` handle kept in a `thread_local!` (as `tests/runtime.rs` does), written from the test body, followed by `advance`. Fake timers (`clock.tick`) map to `advance`.
- **Observations.**
  - `stdout.write` counts and contents map to `TestTerminal` writes.
  - `reconstructTerminalLines` and screen checks map to vt100: scrollback plus visible rows.
  - Ink's `cleanup()` maps to dropping the app.
- **Ink's clear sequences.** Assertions that count `homeAndEraseDown` or other clears and expect zero keep their intent: assert that none of `CSI 2J`, `CSI 3J` and `CSI 1;1H CSI J` is written.
- **The height cap.** Some screen assertions may differ only because design.md caps the frame at the screen height, so the top rows of an overflowing frame never reach scrollback. For those:
  - assert the screen and scrollback that design.md's rule produces;
  - say why in a comment;
  - add one entry to the 差异 section of `docs/ink-tests.md` that names the tests.
- **Any other expectation that fails** means stop and report. Do not change it.

### The tests

- **New `tests/render.rs`**, the standard variants only:
  - do not erase screen;
  - do not erase screen where <Static> is taller than viewport;
  - last line of <Static> survives a full-clear accounting frame (the screen part);
  - erase screen where state changes (the screen intent: A, B, C are gone);
  - fullscreen mode should not add extra newline at the bottom (the screen part);
  - #442: full terminal-size box should not add an extra scroll line (the screen part);
  - #450: initial overflowing frame should not clear terminal;
  - #450: initial full-height frame should not clear terminal;
  - #450 control: rows - 1 rerenders should avoid a full clear (the clear count);
  - #935: overflowing rerenders must not erase terminal scrollback (the screen part);
  - #450: full-height rerenders should not clear before unmount;
  - #450: grow from rows - 1 to full-height should not clear before unmount;
  - #450: non-TTY full-height rerenders should never clear terminal;
  - #450: non-TTY overflow transitions should never clear terminal;
  - #450: non-TTY grow-to-overflow rerender should not clear terminal;
  - clear output;
  - preserve terminal history when rerendering after clear - standard;
  - full clears do not replay <Static> output into scrollback - standard (the screen part);
  - growing into an overflowing frame keeps <Static> rows that are still in the viewport - standard;
  - <Static> items added while the frame overflows are written once - standard (the screen part);
  - alternate screen replays <Static> output when the frame shrinks after overflowing - standard;
  - alternate screen replays <Static> items added while the frame overflows once - standard;
  - throttle renders to maxFps;
  - no throttled renders after unmount (unmount consumes the app, so the tick after it has no counterpart; say so in a comment);
  - unmount forces pending throttled render;
  - no bsu/esu when output is unchanged;
  - bsu/esu wraps throttledLog trailing call.
- **New `tests/clear_rerender.rs`**, the three standard tests.
- **New `tests/issue_973_static_commit.rs`**, its one test.
- **`tests/components.rs`**:
  - render only last frame when stdout is not a TTY;
  - render all frames when interactive is explicitly true;
  - interactive option overrides TTY detection;
  - alternate screen - enters on mount and exits on unmount;
  - alternate screen - cleanup() exits the alternate screen;
  - alternate screen - ignored when non-interactive;
  - alternate screen - disabled by default;
  - alternate screen - content is rendered between enter and exit;
  - alternate screen - ignored when isTTY is false;
  - alternate screen - ignored when isTTY is false even if interactive is true;
  - static output is written immediately in non-interactive mode.
- **`tests/static_runtime_blank_lines.rs`**: the three non-interactive tests and `a blank Static append survives a later live update (incremental: false)`.
- **`tests/measure_element.rs`**: `calculate layout while rendering is throttled`.
- **`src/output/tests/log_update_blank_growth.rs`**: `growing a rendered Box with an empty row preserves terminal history`.
- **`src/output/tests/log_update_columns.rs`**: `side-by-side public components do not rewrite the unchanged left region`. Assert what the cell diff guarantees: the left region is not rewritten and the screen shows the update. Ink's `incrementalRendering` option has no counterpart; ink-rs has one diff.

### `docs/ink-tests.md`

- Update the table rows of every file you port into.
- For each of those files, list in "M2 文件中不移植的测试" the tests that ink-rs will not port, with the reason: 不适用, 重复 (`incrementalRendering` variants), or a later milestone (M3, M4, M5). Do not list tests that a later M2 phase ports: CI detection, resize, the cursor, `use_stdout` and `use_stderr`.
- **APIs this phase does not add.** Ink's `onRender` option (`render-callback.tsx`), `waitUntilRenderFlush`, and `patchConsole` go under 缺 API or 不适用, with the reason:
  - design.md 外部输出 rules out console capture;
  - Ink's `debug` option has `testing::render` as its counterpart and no runtime option.
- `write-synchronized.tsx` tests Ink's internal function and constants. Mark it 不适用 if the ported render tests cover the decision; say which.
- Leave the overview counts alone.

## Rules

- No `unsafe`. No new dependencies (`CARGO_NET_OFFLINE=true`).
- No tests beyond those listed, and no change to an existing expectation, except as section 2 allows for the height cap.
- No code that exists only to satisfy a test. The in-memory terminal and `mount` are the test driver of design.md, not test-only code.
- Match the surrounding code:
  - `pub(crate)` for internal items, imports at the top.
  - Comments explain why and cite the Ink or React source a rule comes from. Do not cite research IDs.
  - Rustdoc on public items.
  - Run `rustfmt --edition 2024` on every file you touch.
- Update `docs/design.md` only where what you build differs from it or decides something it leaves open (for example, non-TTY interactive output is not capped), and list each change in the report.
- Run only the following. Every test in these targets must pass, apart from `#[ignore]`d ones.
  - `cargo test --lib`
  - `cargo test --test render --test clear_rerender --test issue_973_static_commit --test components --test static_runtime_blank_lines --test measure_element --test runtime --test render_to_string --test static_string_replacement`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target`, and delete that directory when you are done.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

1. Proposed commits: message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. The runtime split and the app's step: what runs in which order for each of the three cases (paint at once, throttled paint, deferred frame), and the return value.
3. The public API added (`Options`, `testing`), and the `Terminal` trait.
4. How the alternate screen replays Static rows, and the cost per frame.
5. Results of the commands above. Name any failing test, with your diagnosis.
6. Every assertion adapted for the height cap, and every test listed as not ported, with the reason.
7. Where you departed from this brief or from design.md, and open questions.
