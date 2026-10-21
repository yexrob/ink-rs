# Task: `Color::Reset`, Ink's `backgroundColor=""`

## Background

In Ink, `<Text backgroundColor="">` means "no background of my own, and do not take the enclosing `Box`'s background". Ink's `Text` computes `backgroundColor ?? inheritedBackgroundColor` (read `src/components/Text.tsx` and `src/components/Box.tsx` in the Ink clone): `""` is not nullish, so it wins over the inherited color, and chalk's colorize with `""` adds no color. A `Text` nested inside another `Text` still shows the outer Text's color, because the outer Text's ANSI wraps it.

ink-rs has no value for this: the builders take `impl Into<Option<Color>>`, `None` means "inherit", and the runtime resolves inheritance with `.or(background)` (`src/runtime/mod.rs` around line 813). The Ink test `Mixed text with and without background inheritance` in `test/background.tsx` is not ported for that reason (see `docs/ink-tests.md`, the 缺 API section).

## Change

Add a `Color::Reset` variant in `src/color.rs`: the terminal's default color. The name follows crossterm and ratatui.

- Doc comment on the variant: the terminal's default foreground or background; on a `Text`'s `background_color` it stops the enclosing `View`'s background, like Ink's `backgroundColor=""`. Mention it on the enum doc if that reads better.
- `at_level` returns `None` for `Reset` at every level, so it adds no SGR. That is what stops inheritance: `Some(Color::Reset)` wins the `.or(..)`, then resolves to no color. Nested Text keeps the outer Text's styling, as in Ink.
- `sgr` maps `Reset` to `39` / `49` so the match stays total, even though `at_level` keeps it from being called.
- `FromStr` does not parse `""`. Keep `Display`/other impls total; fix every match the compiler reports.
- Check every place that resolves a `Color` (`src/host.rs` `text_style`, `src/paint.rs` backgrounds and borders, `src/runtime/mod.rs` inheritance, error overview) and confirm `Reset` behaves as "no color" in each: a `View` with `background_color(Color::Reset)` paints no fill and its Text children inherit `Reset`, so they get no background. Borders with `Reset` get no color.

## Test

Port `Mixed text with and without background inheritance` from Ink's `test/background.tsx` into `tests/background.rs`, in Ink's order (it comes right before `Complex nested structure with background inheritance`). Follow the neighboring ported tests: `render_truecolor(.., 100)` and the `styled(&[..])` markup. The expected value is

```rust
styled(&["{bg=green}Inherited {/}No BG {bg=red}Red BG{/}"])
```

for a green `View` with `align_self(AlignSelf::FlexStart)` holding `Text "Inherited "`, `Text "No BG "` with `background_color(Color::Reset)` and `Text "Red BG"` with red background. Check the expected markup against Ink's expected string before you settle on it.

No other new test.

## Docs

- `docs/ink-tests.md`: the test is now ported. Update the totals in the summary table at the top (ported count +1, `#[test]` count +1, not-ported count −1), the `background.tsx` row (27 → 28), remove the 缺 API row and the 缺 API bullet for this test, and remove the `<Text backgroundColor="">` bullet in the differences section. Recount rather than trusting these numbers; keep the file consistent.
- `docs/design.md`: where colors are described (search for `Color` and `chalk`, around line 58), add one sentence on `Color::Reset`.

## Commands

- `cargo test --offline --test background`
- `cargo test --offline --lib`
- `cargo test --offline --test borders --test components` (only the ones that exist; list them with `ls tests`)

## Project context

ink-rs is a Rust port of Ink (React for CLIs). Ink's tests are the spec. Read `CLAUDE.md`, `AGENTS.md` and the relevant parts of `docs/design.md` before you start. A read-only clone of Ink, with its `node_modules`, is at
`<scratchpad>/ink-src/ink`. Read and grep it; never run anything from it.

## Rules

- No unsafe code. No dependencies beyond the ones this brief names. Code reads like the surrounding code: same naming, comment density and idiom. No code written only to satisfy a test.
- Add only the tests this brief asks for. Existing tests keep their expectations unless this brief says otherwise; if one must change, say which and why in the report.
- Docs are in Chinese, in the style of `docs/design.md`: plain statements, no adverbs. Update only the doc passages this brief names, plus any other passage your change makes false.
- Stay inside the files this brief names. Other workers are changing other parts of the tree in parallel.
- Run only the commands this brief lists, all with `--offline`, plus `cargo fmt --all --check` and `cargo clippy --offline --workspace --all-targets -- -D warnings`. Do not run the full `cargo test`; I run it at integration.
- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report, in Chinese, no adverbs:
- the files you changed, and what changed in each;
- design choices the brief left open, and what you chose;
- every deviation from this brief, and why;
- the exact commands you ran and their results.
