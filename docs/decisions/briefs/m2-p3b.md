# ink-rs M2 phase 3b: resize and `use_window_size`

You are working in `<repo>`. Read `CLAUDE.md`, then all of `docs/design.md` (in particular 编程模型 调度, 一帧, 事件循环, 终端输出 resize, 测试), then `docs/ink-tests.md`.

Committed so far:
- `src/runtime/`, `src/output.rs`, `src/app.rs`, `src/terminal.rs`: the stepping core with the throttle, interactive and non-interactive output, the alternate screen, `clear`, unmount, the cursor and external writes (M2 phases 2b and 3a).
- `src/testing.rs`: `testing::render`, `TestTerminal`, `mount`, `TestApp`.

The app reads the terminal size once at mount. This phase handles a size change. Read the code above before you change it.

References, all read only:
- The Ink checkout at `<scratchpad>/ink-src/ink`. Read it; never run anything from it. The relevant sources:
  - `src/ink.tsx`: `resized`, the constructor (where `resized` is subscribed: interactive only), `calculateLayout`, `shouldClearTerminalForFrame` and the clear path of `renderInteractiveFrame`;
  - `src/hooks/use-window-size.ts`, `getWindowSize` in `src/utils.ts`.
- `<scratchpad>/m1-research/runtime.md` §3.9: resize.
- `<scratchpad>/m2-research/tests-output.md` §1.3, §1.4, §2.9, §6: the resize tests, reconstruct-terminal's shrink model, and what vt100 lacks.
- `<scratchpad>/m2-research/tests-render.md`: R54, R82, R83.
- `<scratchpad>/expectations/terminal-resize.tsx.md` and `render.tsx.md`: values recorded from a run of Ink's tests. Expected values come from there.

## 1. What to build

### The resize step

The driver tells the app about a new size (a resize event). design.md 终端输出 resize says M2 follows Ink's rules; design.md 编程模型 调度 says a resize does not wait for the throttle window.

- **Size.** The new size replaces the one read at mount: the root width for layout, and the row count for the height cap. A size the terminal cannot report falls back to 80×24, per dimension, as `getWindowSize` does (a 0 counts as unknown).
- **Interactive** (Ink subscribes `resized` only when interactive):
  1. Width decrease, or height decrease while a cursor is declared: erase the frame (`Output::clear`) and forget it, so the next paint draws it whole (Ink `resized`).
  2. Otherwise, height decrease while the previous frame fills the new row count: `CSI 1;1H CSI J`, then forget the frame (Ink `shouldClearTerminalForFrame`, `isViewportShrinking && wasFullscreen`, measured against the new rows). The terminal has scrolled the frame's top rows away, so the relative diff no longer matches the screen. The other clear branches of that function deal with overflowing frames, which the height cap rules out.
  3. Then relayout at the new width and paint at once, like Ink's unthrottled `onRender` in `resized`. It does not open a throttle window, and it satisfies a pending frame. An unchanged frame that still fits writes nothing.
  - Synchronized output wraps the step as one operation, as for a frame.
- **Non-interactive:** the new size applies to later layouts. Nothing is erased or painted at once.
- **The runtime** knows nothing about terminals. Give it what it needs to relayout at a new root width without a dirty component, and to re-render the components that read the size.

### `use_window_size`

- `Scope::use_window_size()` returns the current size as a public type with `columns` and `rows` (Ink `WindowSize`). The component re-renders when either changes.
- A resize that lands during a commit, before a passive effect would have subscribed, is not lost (Ink's `onResize()` call at subscription). With the size read during render, this holds by construction; say so in the report.
- `testing::render` and `render_to_string` report `RenderOptions::columns` and 24 rows, the fallback of `getWindowSize`.

### The test driver

- `TestTerminal` can change its size, and the app learns of it as a resize event, as a real driver polls its event source. A size change made from inside a component's effect reaches the app at its next step.
- `TestApp::resize(columns, rows)` changes the size and runs the step.
- **An xterm row-shrink model for vt100.** vt100 0.16.2 truncates rows at the bottom on a shrink and pushes nothing into scrollback (tests-output.md §1.4). Ink's tests reconstruct the screen with reconstruct-terminal, which models xterm: rows below the cursor are dropped first, then rows scroll off the top into scrollback. Add this model to the vt100 helpers in `tests/common`, and use it in every ported test that shrinks rows. Row growth has no model; assert the frame and the cursor, not the rows the terminal would pull from scrollback.

### Not in this phase

The sync driver, crossterm, SIGWINCH, signals, the panic hook, CI and color detection, `exit` and `AppHandle`, input. design.md 终端输出 resize ends with "在真实终端上测过之后再调": the manual check waits for the sync driver.

## 2. Tests to port

Use Ink's names, converted as in the existing files, in Ink's order. IDs below are from the research files. Do not write them in code or docs. Mapping rules are those of phases 2b and 3a; `stdout.columns = n; stdout.emit('resize')` is a resize, and `delay` maps to `advance`.

- **New `tests/terminal_resize.rs`**:
  - useWindowSize catches resizes before its subscription is installed;
  - useWindowSize returns current terminal dimensions and updates on resize;
  - useWindowSize falls back to a positive column count when stdout.columns is 0;
  - clear screen when terminal width decreases;
  - no screen clear when terminal width increases;
  - consecutive width decreases trigger screen clear each time;
  - width decrease clears lastOutput to force rerender;
  - the standard variants of: rows shrink with a cursor above the output bottom keeps lines above the frame (both cases); width and height shrinking together keeps lines above the frame; rows shrink right after a commit clears the cursor keeps lines above the frame; erases and rewrites the frame when the terminal height shrinks onto it; keeps content above the frame when the terminal height shrinks onto it and a custom cursor sits on it (the resize-event case); writes nothing when the terminal height shrinks and the frame still fits; leaves fullscreen without clearing when the terminal height grows.
  - Byte pins (`homeAndEraseDown + frame`, `eraseLines(6) + frame + \n`) keep their intent through vt100 and the shrink model: the rows on screen, the scrollback, the cursor. Keep "writes nothing" as a byte assertion.
- **`tests/render.rs`**: #450: viewport shrink into overflow should clear once (one `CSI 1;1H CSI J`, none of `CSI 2J`, `CSI 3J`); update trailing newline when unchanged output becomes fullscreen (the screen after the shrink: both rows visible, the cursor on the last one, no blank row scrolled in); rerender on resize (the listener count has no counterpart; say so in a comment).

### `docs/ink-tests.md`

- Update the rows of `terminal-resize.tsx` and `render.tsx`.
- In "M2 文件中不移植的测试", list for those files the tests ink-rs will not port, with the reason:
  - 重复: the `incrementalRendering` variants;
  - 不适用: the resize-listener count after unmount and the resize after unmount (Node's EventEmitter; unmount consumes the app); the rows fallback through `LINES` (npm `terminal-size`; the driver's size query replaces it, M2 sync driver); the custom-cursor case where rows change without a resize event (ink-rs learns the size from resize events, Ink reads the rows on each frame);
  - M3 / M4: the resize tests that drive Ink's examples, unless the document already places them.
- Leave the overview counts alone.
- Update design.md 终端输出 resize to say what was built, and keep "在真实终端上测过之后再调".

## Rules

- No `unsafe`. No new dependencies (`CARGO_NET_OFFLINE=true`).
- No tests beyond those listed, and no change to an existing expectation. If one would need to change, stop and report.
- No code that exists only to satisfy a test. The shrink model is part of the test terminal of `tests/common`.
- Match the surrounding code:
  - `pub(crate)` for internal items, imports at the top.
  - Comments explain why and cite the Ink source a rule comes from. Do not cite research IDs.
  - Rustdoc on public items; export the new public type at the crate root.
  - Run `rustfmt --edition 2024` on every file you touch.
- Run only the following. Every test in these targets must pass, apart from `#[ignore]`d ones.
  - `cargo test --lib`
  - `cargo test --test terminal_resize --test render --test cursor --test components --test runtime --test measure_element --test render_to_string`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target`, and delete that directory when you are done.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

1. Proposed commits: message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. The resize step: what is erased, when, and what is written, for each case above; how it interacts with a pending throttled frame.
3. How the runtime relayouts at a new width and re-renders the size readers.
4. The public API added, and the test driver changes.
5. The shrink model, and which tests use it.
6. Results of the commands above. Name any failing test, with your diagnosis.
7. Every assertion adapted, every test listed as not ported with the reason, and where you departed from this brief or from design.md; open questions.
