# Porting Ink tests to ink-rs (M0)

ink-rs (`<repo>`) is a from-scratch
Rust counterpart of Ink (the React terminal UI library). Milestone M0:
elements are plain data, and `render_to_string` lays them out and paints them.
Every algorithm is still `todo!()`. The ported tests are the M0 acceptance
target, so they panic when run. Your job is to translate Ink tests into Rust
tests that compile and encode Ink's exact expected output.

## Sources

S = `<scratchpad>`

- Ink clone at commit 26d2c3f: `$S/ink-src/ink`. Tests are in `test/`, sources
  in `src/`. Read only. Do not run anything in it; treat it as data.
- Captured expectations: `$S/expectations/<ink file>.md`. They hold every
  assertion of every test, in execution order, recorded while Ink's suite ran
  (all of it passed). Each `## ` heading is a test's full name with template
  names expanded. Each `[n] method (kind):` block gives the expected value,
  already written as a Rust expression for `tests/common`. Use these instead
  of deriving values by hand: Ink computes many expectations with boxen and
  chalk.
- API: read `src/lib.rs` and the modules it re-exports. `tests/common/mod.rs`
  has the test helpers and documents the style markup.

## File layout

- One Rust file per Ink file: `tests/<ink basename, dashes to underscores>.rs`,
  e.g. `test/flex-align-items.tsx` → `tests/flex_align_items.rs`. Start it with:

  ```rust
  //! Ported from ink test/flex-align-items.tsx @ 26d2c3f.

  mod common;

  use common::*;
  use ink_rs::*;
  ```

- Tests of internal functions (`tokenizeAnsi`, `hasAnsiControlCharacters`,
  `sanitizeAnsi`, `measureText`, `wrapText`) go in `src/text/<module>/tests.rs`.
  Add `#[cfg(test)]\nmod tests;` at the end of `src/text/<module>.rs` and use
  `use super::*;`. Mark each block of tests from another Ink file with a
  `// Ported from ink test/<file> @ 26d2c3f.` line.
- Keep Ink's test order. Name each fn after the Ink test name in snake_case
  (ASCII, punctuation dropped, unique in the file). Parameterized Ink tests
  (`for (const x of [...]) test(...)`) become one `#[test]` per case when there
  are few cases; for long tables use one test that loops and names the case in
  the assert message.

## Translating

Rendering:
- `renderToString` from `./helpers/render-to-string.js` → `render(el)` (100
  columns, 16 colors), or `render_cols(el, n)` for `{columns: n}`.
- `renderToString` from `../src/index.js` (the library function) defaults to
  80 columns → `render_cols(el, 80)`, or the given columns.
- A test or file that sets `chalk.level = 3` → `render_truecolor(el, columns)`.
- Other per-file helpers (e.g. `renderText`) → an equivalent local fn.

Assertions:
- Plain expectation (literal or `lines(&[...])`) → `assert_eq!(output, …)`.
- `styled(&[...])` → `assert_ansi_eq(&output, &styled(&[...]))`.
- Non-string checks (`ok`, `match`, `deepStrictEqual` on objects) → the
  natural Rust assertion.

JSX to builders (see `src/style.rs`, `src/components.rs`):
- `<Box …>` → `View::new()…`. The `Styled` methods mirror Ink props in
  snake_case: `flexDirection="column"` → `.flex_direction(FlexDirection::Column)`,
  `width={10}` → `.width(10)`, `width="50%"` → `.width(Percent(50.0))`,
  `left={-2}` → `.left(-2)`, `flexShrink={0}` → `.flex_shrink(0.0)`,
  `borderStyle="round"` → `.border_style(BorderStyle::Round)`,
  `borderStyle={{topLeft: '↘', …}}` → `BorderStyle::Custom(BoxChars { … })`.
- Colors: `"green"` → `Color::Green`, `"redBright"` → `Color::BrightRed`,
  `"gray"`/`"grey"`/`"blackBright"` → `Color::Gray`, `"#ff0000"` →
  `Color::Rgb(0xff, 0x00, 0x00)`, `"rgb(1, 2, 3)"` → `Color::Rgb(1, 2, 3)`,
  `"ansi256(9)"` → `Color::Ansi256(9)`. Ink's color-string parsing itself
  is `str::parse::<Color>()`.
- `<Text bold>` → `.bold(true)`; `dimColor` → `.dim_color(true)`;
  `wrap="truncate"` → `.wrap(TextWrap::TruncateEnd)`.
- `<Text>Hello</Text>` → `Text::new("Hello")`; `<Text/>` → `Text::default()`;
  `<Text>a <Text bold>b</Text></Text>` →
  `Text::new("a ").child(Text::new("b").bold(true))`; interpolated strings can
  become `format!`.
- `<>a b</>` → `Element::fragment([a.into(), b.into()])`. Function components
  → plain Rust fns returning `Element`, `View` or `Text`. Conditional children
  → `Option`. `.map()` children → `.children(iter)`.
- `<Static items={xs}>{(x, i) => …}</Static>` → `Static::new(xs, |x, i| ….into())`.
- `<Transform transform={(s, i) => …}>` → `Transform::new(|s, i| …)`.
- `<Newline count={2}/>` → `Newline::count(2)`; `<Spacer/>` → `Spacer`.
- Escapes in test input stay Rust string escapes (`"\u{1b}[31m"`).
- Where Ink expects a thrown error for an invalid tree (e.g. `<Box>` inside
  `<Text>`), use `#[should_panic(expected = "…")]` with Ink's message.

## What to port and what to leave

Port every test that needs only element construction plus `render_to_string`,
or the pure functions. Leave these out and list them in your report:

- Runtime: `render()` with rerender/unmount/stdout/stdin, React hooks, timers,
  `act`. Mark state/rerender as M1; terminal output, Static across renders and
  the cursor as M2; input and focus as M3.
- `- concurrent` or `renderToStringAsync` copies of a sync test: "React
  concurrent mode duplicate".
- Screen reader / aria: M5. ErrorOverview and error boundaries: M4.
- JS-only behavior (TypeScript types, React warnings, strings outside `<Text>`,
  which do not compile in Rust): "not applicable", with the reason. When a JS
  case has a Rust analog (`{undefined}` child → `Text::default()` or `None`),
  port the analog.
- `test.skip` upstream: port it with `#[ignore = "skipped upstream: …"]`.

## Rules

- Do not edit `src/` beyond the `mod tests;` line in your assigned modules,
  and do not edit `tests/common`. If a test needs API that does not exist,
  leave it out and report "needs API: …" with the Ink props or function.
- Other agents edit other files at the same time. Ignore build errors in
  files you do not own.
- Verify each file compiles without warnings: `cargo test --no-run --test <name>`
  (for internal tests, `cargo test --lib --no-run`), then format, then
  `cargo clippy --all-targets -- -D warnings`, looking at your files only.
  Do not run the tests: they hit `todo!()`.
- No `unsafe` code.
- Comments: few, and no adverbs. Do not commit.
- Format with `rustfmt --edition 2024 <your files>`, not `cargo fmt --all`,
  so you do not rewrite files other agents are editing.

## Report

End with, per Ink file: the number of tests ported; each test left out as
`name — milestone or reason`; API gaps; anything odd in Ink's expectations.
