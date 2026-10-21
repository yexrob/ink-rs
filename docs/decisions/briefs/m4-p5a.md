# ink-rs M4 phase 5, batch 1: Ink examples with in-process tests

You are working in a git worktree of ink-rs at `<scratchpad>/wt/m4-p5a`.

Read first:
- `CLAUDE.md`.
- `docs/design.md`, these sections: 组件, 输入, 测试, 移植 Ink 测试的方法.
- `docs/ink-tests.md`: the M4 example list near line 270.
- In ink-rs:
  - `src/lib.rs` (the public API); `examples/counter.rs`;
  - `src/testing.rs` (`mount`, `TestApp`); `tests/common/mod.rs` (`Screen`);
  - `tests/render.rs` (header style, the `flush` helper); `tests/terminal_resize.rs`; `tests/use_animation.rs`; `tests/use_box_metrics.rs`;
  - `src/text/width.rs`, `src/text/measure.rs`, `src/text/styled.rs` (`parse`).

Design reference: `<scratchpad>/m4/research-examples-tokio.md`, sections §1–§3 and Q1, Q3–Q8, Q11. The user approved its recommendations. The decisions below restate them and take precedence.

Ink checkout, read only; never run anything from it: `<scratchpad>/ink-src/ink`.
- Sources: `examples/<name>/`.
- Tests:
  - `test/render.tsx`, the example tests between lines 271 and 760, and the `term` helper they use;
  - `test/alternate-screen-example.tsx`;
  - the fixtures `test/fixtures/aria-example.tsx` and `test/fixtures/alternate-screen-full-board-win.tsx`.

Another worker builds component panic handling in parallel. It changes `src/error.rs`, `src/app.rs`, `src/runtime/` and the error export in `src/lib.rs`. Do not touch those files, except the one `text_width` line in `src/lib.rs`.

## 1. Scope

Port these 20 Ink examples:
- alternate-screen (snake), aria, border-backgrounds, borders, box-backgrounds;
- chat, cursor-ime, incremental-rendering, justify-content, render-throttle;
- scroll, select-input, static, table, use-animation;
- use-focus, use-focus-with-id, use-input, use-stderr, use-stdout.

Left out of this batch:
- terminal-resize and suspend-terminal come in batch 2 (PTY tests).
- jest and subprocess-output come in batch 3 (tokio).
- counter is ported.
- concurrent-suspense, suspense, use-transition and router are not applicable (React-only).

## 2. Decisions

### 2.1 Layout

- An example with a test uses the Cargo multi-file layout:
  - `examples/<name>/main.rs` holds `mod app;` and `fn main() -> Result<(), ink_rs::Error>`;
  - `examples/<name>/app.rs` holds the components, the reducers and the line logic. It uses the public API only, because the tests include it by path.
- An example without a test is one file, `examples/<name>.rs`, as `counter.rs` is.
- Keep Ink's directory names, kebab-case, so `cargo run --example use-input` matches Ink's `npm run example examples/use-input`. Cargo needs no `[[example]]` entries.
- Each example starts with a comment naming the Ink example it ports, as `counter.rs` does.

### 2.2 `ink_rs::text_width`

Add `pub fn text_width(text: &str) -> usize` at the crate root.
- Contract: the columns the renderer gives the widest line of `text`. ANSI escape sequences count zero. Graphemes are measured as paint measures them (string-width 8.3 rules).
- Build it on the existing text code (`styled::parse` and `measure`). Add no new width logic.
- Document it for `use_cursor` users: a cursor column must agree with the renderer, and `unicode-width` disagrees on ZWJ emoji and Hangul jamo.
- The cursor-ime tests cover it; add no separate test.

### 2.3 Ticks

Periodic UI ticks use `use_animation`, so `TestApp::advance` drives them on the fake clock:
- snake: 150 ms;
- incremental-rendering: 16 ms and 1 s;
- use-stdout and use-stderr: 1 s;
- static: 100 ms; the animation is active while fewer than 10 items exist;
- render-throttle: 10 ms with `Options { max_fps: 10, .. }`.

How to consume the ticks:
- Where Ink's state is a tick count, derive it from `frame`.
- Otherwise, take one step per delivered frame in a `use_effect` keyed on `frame`.
- `frame` jumps after a late or throttled wake. Say in a comment which of the two rules each example uses.

`counter.rs` stays as the thread + `AppHandle` reference.

### 2.4 Translation rules

- `useReducer` becomes a `State<T>` plus `update(|s| reduce(s, action))` with a plain `fn reduce`.
- `useRef` becomes `use_memo((), || Rc::new(Cell::new(..)))`.
- One input chunk dispatches several keys against the closures of one render. Key handlers that change state per key use `State::update`, not `set(f(get()))`. This applies to chat, aria, scroll, use-input and snake.
- Render-phase state writes panic in ink-rs debug builds. incremental-rendering derives the clamped selection in render, and its handlers clamp before they move.
- Third-party libraries:
  - `Intl.Segmenter` becomes `unicode-segmentation` (already a dependency);
  - `string-width` becomes `ink_rs::text_width`;
  - `wrap-ansi` (hard) becomes `TextWrap::Hard`;
  - faker becomes fixed rows;
  - `Math.random` becomes a local xorshift of a few lines, seeded from `std::collections::hash_map::RandomState`.
- No new dependencies.
- aria and select-input port the visual output. The aria props and the screen-reader branch wait for M5. Leave a one-line comment where they would go and add no stub API.
- incremental-rendering: ink-rs has no `incrementalRendering` option, because it diffs cells in every mode. Drop the option with a comment.
- alternate-screen uses `Options { alternate_screen: true, .. }`.

## 3. Tests

Create `tests/examples.rs`:
- Include each tested example as `#[path = "../examples/<name>/app.rs"] mod <snake_name>;`.
- No `#[allow(dead_code)]`. Every item in `app.rs` must be reached from the entry point or used by a test. Items that only `main` needs stay in `main.rs`.
- The header cites `test/render.tsx` and `test/alternate-screen-example.tsx` (Ink 8.0.0, commit 26d2c3f).
- Each test carries a comment with its `render.tsx` line.
- Keep Ink's names in snake_case and Ink's order.

Port these 18 tests, following the "Rust form" column of research §2:

1. `IME example deletes whole graphemes with backspace` (271)
2. `IME example ignores Enter when editing text` (299)
3. `IME example places the cursor after wrapped input` (320)
4. `table example keeps all columns visible in narrow terminals` (344): unit, `render_to_string` at 40 columns
5. `stdout example updates displayed dimensions after resize` (392)
6. `aria example applies 1 consecutive checkbox toggles` (497)
7. `aria example applies 2 consecutive checkbox toggles`
8. `aria example applies 3 consecutive checkbox toggles`
9. `chat example submits intact text after deleting an emoji` (538)
10. `chat example submits pending edits from the same input chunk` (561)
11. `chat example does not insert modified shortcuts into messages` (586)
12. `scroll example handles consecutive arrow keys in one input chunk` (606)
13. `input example handles consecutive movement keys` (638)
14. `incremental example keeps its selection visible after resize` (672)
15. `incremental example grows its logs after terminal resize` (703)
16. `snake ignores a reversal queued between ticks` (727)
17. `snake can move into the tail cell when the tail moves away` (alternate-screen-example.tsx 9): unit on `reduce`
18. `snake ends with a win when it fills the board` (alternate-screen-example.tsx 39): unit on `reduce` with the 20×15 serpentine board

Mapping:
- In-process tests use `testing::mount` with the terminal size of Ink's `term` call (default 100 columns), feed bytes with `TestApp::input`, move time with `advance`, and read rows and the cursor through `Screen`.
- Ink's `waitForOutput` becomes `advance` past the throttle window, as the `flush` helper in `tests/render.rs` does.
- Byte checks on `eraseLines`, `cursorTo` and `cursorUp` become screen-row and cursor-position checks.
- `CI=true` becomes `interactive: Some(false)`, and the check reads the final frame at unmount.
- The aria fixture delivers all keys in one read, so its test makes one `input` call with N repeats.

When an assertion cannot keep Ink's form, keep its intent, say why in a comment, and list it in the report. Add no tests beyond these 18.

## Rules

- No `unsafe`. No new dependencies.
- No code that exists only to satisfy a test. The examples are user-facing: write them as a reader of the crate docs would, short and idiomatic.
- Match the surrounding code:
  - `pub(crate)` or narrower for internal library items;
  - imports at the top;
  - comments that say why and cite Ink where the code ports it.
- Run `rustfmt --edition 2024` on every file you touch.
- Do not edit `docs/`. Report the documentation changes instead.
- Run only the following. Every test in these targets must pass.
  - `cargo test --test examples --test text_width`
  - `cargo build --examples`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration. If the sandbox has no network, add `--offline`.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

Write the report in English.

1. Proposed commits, with message and files:
   - `text_width` first;
   - then the examples without tests;
   - then the tested examples with `tests/examples.rs`.

   Each commit must build and pass its tests on its own.
2. Per example:
   - the tick rule it uses (2.3);
   - each departure from the Ink example, with the reason.
3. The ported tests:
   - their names;
   - every adapted assertion, with the reason.
4. Results of the commands above. Name any failing test, with your diagnosis.
5. Documentation:
   - the rows and entries to change in `docs/ink-tests.md`, including the 4 not-applicable examples;
   - what `docs/design.md` should record;
   - open questions.
