# ink-rs M2 phase 3a: the cursor and external writes

You are working in `<repo>`. Read `CLAUDE.md`, then all of `docs/design.md` (in particular 跨层约定, 编程模型, 一帧, 终端输出 光标 and 外部输出, 测试), then `docs/ink-tests.md`.

Committed so far:
- `src/runtime/`: components, hooks, effects in React's order, Static across frames.
- `src/output.rs`: the cell diff. `Output::render` already takes a cursor position; it hides, places and shows the cursor as Ink's log-update does, and `Output::done` returns the cursor below the frame.
- `src/app.rs` and `src/terminal.rs` (M2 phase 2b): the stepping core with the throttle, the interactive and non-interactive paths, the alternate screen, `clear` and unmount.
- `src/testing.rs`: `testing::render` (Ink's debug mode), and `TestTerminal`, `mount` and `TestApp` (phase 2b).

No component can declare a cursor yet, and nothing outside the frame can reach the terminal. This phase adds `use_cursor`, `use_stdout` and `use_stderr`. Read the code above before you change it.

References, all read only:
- The Ink checkout at `<scratchpad>/ink-src/ink`. Read it; never run anything from it. The relevant sources:
  - `src/hooks/use-cursor.ts`, `src/components/CursorContext.ts`;
  - `src/hooks/use-stdout.ts`, `src/hooks/use-stderr.ts`, `src/components/StdoutContext.ts`, `src/components/StderrContext.ts`, `src/components/App.tsx` (where the contexts get their `write`);
  - `src/ink.tsx`: `setCursorPosition`, `restoreLastOutput`, `writeToStdout`, `writeToStderr`, `renderInteractiveFrame` (the cursor checks), `unmount` (where `isUnmounted` is set);
  - `src/render-to-string.ts` (the default contexts: `write` does nothing).
- `<scratchpad>/m2-research/tests-output.md` §1, §2.6-2.8, §5-7: the cursor tests, what each observes, and their conflicts with design.md.
- `<scratchpad>/m2-research/tests-render.md`: R117.
- `<scratchpad>/expectations/`: values recorded from a run of Ink's tests (`cursor.tsx.md`, `cursor-exit-position.tsx.md`, `hooks.tsx.md`, `render.tsx.md`). Expected values come from there. Fixtures are under `test/fixtures/` of the checkout.

## 1. What to build

### `use_cursor`

design.md 终端输出 光标: a component declares where the terminal cursor goes, for IME candidate windows.

- **API.** `Scope::use_cursor(position)`, called during render with an optional position. `None` hides the cursor. Choose the public position type (cells, relative to the origin below) and say why in the report.
- **When it applies.** Ink's `useCursor` stores the position during render and propagates it in a `useInsertionEffect` without deps, whose cleanup sets the position to undefined. So:
  - the position takes effect at each commit in which the component renders;
  - a component that skips rendering (props equal) keeps the position it last committed;
  - unmounting a component that called `use_cursor` sets the position to `None`, whoever set it;
  - when several components call it in one commit, the last insertion effect in React's commit order wins (`commitMutationEffectsOnFiber` in React: deletions, then children, then the fiber's own insertion effects).
  - Build it as an internal insertion-effect kind of the runtime. Do not add a public `use_insertion_effect`.
- **Where it applies.** The position is relative to the top-left of the live frame, below Static output, as Ink's log-update sees it. Where phase 2b hands `Output` replayed Static rows above the live rows (the alternate screen), offset the position by those rows.
- **Throttle.** The cursor belongs to the frame: a commit that changes it makes a pending frame, painted under the same throttle rules. An unchanged frame with an unchanged cursor writes nothing (Ink `renderInteractiveFrame`, `isCursorDirty`); a cursor-only change writes only the cursor motion.
- **Other paths.** Non-interactive output, `testing::render` and `render_to_string` ignore the cursor, as Ink's non-interactive and debug paths do.

### `use_stdout` and `use_stderr`

design.md 终端输出 外部输出 and 编程模型 调度 ("外部写不等窗口"). The user chose to offer only `write`; there is no stream object.

- **API.** `Scope::use_stdout()` and `Scope::use_stderr()` return handles with `write(&str)`. They are `Copy` and resolve the runtime as `State` does, so closures and effects can hold them.
  - A write during render panics in debug builds, as a state write does.
  - Writes from the moment unmount begins its tree teardown (phase 2b's step 2) are dropped (Ink sets `isUnmounted` before React's cleanup).
  - A write of an empty string does nothing.
- **Timing.** A write reaches the terminal when it is made, as in Ink, not at the end of the step. The driver installs the destination when it mounts the runtime. The app must not hold a borrow of its output state while user code runs (effects, cleanups). Say in the report how this is guaranteed and what happens on a write from a layout effect, before the commit's frame is painted.
- **Interactive** (Ink `writeToStdout`, `writeToStderr`):
  - stdout: one terminal write holding `[bsu]`, the erase of the frame (`Output::clear`), the data, the redraw of the last frame with its cursor, `[esu]`;
  - stderr: a stdout write with `[bsu]` and the erase, a stderr write with the data, a stdout write with the redraw and `[esu]`;
  - `bsu`/`esu` as phase 2b decides synchronized output;
  - the redraw writes the last painted frame. A frame that is pending under the throttle stays pending and is painted when its window ends.
  - **Line endings.** The data is written with each `\n` that is not preceded by `\r` written as `\r\n`, and with `\r\n` appended when it does not end in `\n`. Ink writes the data unchanged. The reasons, to record in design.md 外部输出 as a difference from Ink: crossterm's raw mode (M3) clears `OPOST`, so a bare `\n` would not return to column 0 (Node's raw mode keeps `ONLCR`); and the redraw starts at column 0, where the cell diff assumes it is, so data without a final newline would be overwritten.
- **Non-interactive:** the data, unchanged, to its stream.
- **`testing::render`** (Ink's debug branch):
  - a stdout write records one entry in `frames()`: the data, then the accumulated Static output, then the last live output;
  - a stderr write records the data in a new `stderr()` list of `TestInstance`, then one entry in `frames()` with the accumulated Static output and the last live output;
  - keep the signatures `last_frame(&self) -> &str` and `frames(&self) -> &[String]`, and keep entries in the order they were written;
  - update the rustdoc of both.
- **`render_to_string`** drops writes, as Ink's default contexts do.
- **`TestTerminal`** records stderr writes as well, in one sequence with stdout writes so a test can replay both streams into one screen (a PTY shares them), and can read each stream alone.

### Not in this phase

Resize and `use_window_size`, the sync driver, crossterm, signals, the panic hook, CI and color detection, `exit` and `AppHandle`, input.

## 2. Tests to port

Use Ink's names, converted as in the existing files, in Ink's order. IDs below are from the research files. Do not write them in code or docs.

Mapping rules are those of phase 2b (see the header of `tests/render.rs` and `tests/common`):
- `createStdout()` with `isTTY` is a TTY `TestTerminal` with 24 rows unless the test sets `rows`; `createStdout()` used as stderr is the same terminal's stderr.
- `delay`, `waitForCondition` and `waitUntilRenderFlush` map to running the due steps (`advance`).
- PTY fixtures (`term`) become in-process tests that rebuild the fixture's tree; stdout and stderr replay into one vt100 screen.
- Byte assertions on hide/show order (`lastIndexOf(?25h) > lastIndexOf(?25l)`) and on cursor motion (`cursorUp(n) + cursorTo(x) + ?25h`) keep their intent through vt100: the cursor's row, column and visibility after the writes. Keep a byte assertion where it names a property the cell diff shares with Ink (for example, no `?25h` after the cursor's owner unmounts).
- The height cap: as in phase 2b, assert what design.md's rule produces, say why in a comment, and add the test to the existing 差异 entry of `docs/ink-tests.md`.
- Any other expectation that fails means stop and report. Do not change it.

The tests:
- **New `tests/cursor.rs`** (`cursor.tsx`):
  - memoized cursor survives sibling updates (incremental: false) — Ink's `React.memo` is the default props bail-out;
  - cursor remains visible after useStdout().write();
  - cursor remains visible after useStderr().write();
  - debug mode: useStdout().write() replays latest frame;
  - debug mode: useStdout().write() does not leak into stderr;
  - debug mode: useStderr().write() replays latest frame without empty writes;
  - debug mode: useStdout().write() replays rerendered frame;
  - debug mode: useStderr().write() replays rerendered frame;
  - standard rendering - fullscreen: cursor lands on the requested row across rerender and cursor-only update;
  - standard rendering - fullscreen: cursor lands on the requested row on the sync path (the frame overflows: height cap).
- **New `tests/cursor_exit_position.rs`**: unmount returns a positioned cursor below the frame (incremental: false); alternate-screen exit does not reposition the primary cursor (incremental: false).
- **New `src/output/tests/cursor_exit_position.rs`**: done returns the cursor once without erasing output (incremental: false), on `Output` directly as the other log-update ports do.
- **New `tests/hooks.rs`** (`hooks.tsx`): useStdout - write to stdout; useStderr - write to stderr. The fixtures print `exited` after the app exits; write it to the screen after unmount.
- **`tests/render.rs`**: no bsu/esu when output and cursor are unchanged, next to its sibling from phase 2b.

### `docs/ink-tests.md`

- Update the rows of `cursor.tsx`, `cursor-exit-position.tsx`, `hooks.tsx` and `render.tsx`.
- In "M2 文件中不移植的测试", list for those files the tests ink-rs will not port, with the reason:
  - 重复 (`incrementalRendering` variants);
  - M3: the cursor tests that send input or call `useInput`; the `useInput` tests of `hooks.tsx`;
  - 不适用: the Suspense/concurrent cursor test;
  - `cursor-helpers.tsx` (all 17) and `line-update.ts` (all 29) test Ink's internal functions; the cell diff has no counterpart. Mark them 不适用 unless the document already places them.
- Leave the overview counts alone.

## Rules

- No `unsafe`. No new dependencies (`CARGO_NET_OFFLINE=true`).
- No tests beyond those listed, and no change to an existing expectation, except as section 2 allows for the height cap.
- No code that exists only to satisfy a test.
- Match the surrounding code:
  - `pub(crate)` for internal items, imports at the top.
  - Comments explain why and cite the Ink or React source a rule comes from. Do not cite research IDs.
  - Rustdoc on public items; export the new public types at the crate root.
  - Run `rustfmt --edition 2024` on every file you touch.
- Update `docs/design.md` where what you build differs from it or decides something it leaves open (the line endings above, the cursor origin), and list each change in the report.
- Run only the following. Every test in these targets must pass, apart from `#[ignore]`d ones.
  - `cargo test --lib`
  - `cargo test --test cursor --test cursor_exit_position --test hooks --test render --test components --test runtime --test render_to_string --test static_runtime_blank_lines`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target`, and delete that directory when you are done.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

1. Proposed commits: message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. How the cursor is stored, ordered across components and cleared, and how it reaches `Output` on each path.
3. How writes reach the terminal: the destination the driver installs, the borrow guarantee, the byte sequence for stdout and stderr, and the layout-effect case.
4. The public API added.
5. Results of the commands above. Name any failing test, with your diagnosis.
6. Every assertion adapted (vt100 for bytes, height cap), and every test listed as not ported, with the reason.
7. Where you departed from this brief or from design.md, and open questions.
