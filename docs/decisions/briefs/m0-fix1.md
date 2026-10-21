# ink-rs M0 phase 3, batch 1: a styled mark without a base, and two paint tidy-ups

You are working in `<repo>`. Read `CLAUDE.md` and `docs/design.md` first. Your phase 2 work is committed (`02b7134`, `8bd762e`, `77290c2`); the integrator has also fixed some ported tests (`5171096`). The Ink reference checkout is at `<scratchpad>/ink-src/ink` (`INK`); read it, never run anything from it.

Layout differences from Yoga (aspect ratio, baseline, flex basis) are out of scope for this batch; do not touch `src/layout.rs` beyond what task 3 needs.

## 1. A styled mark without a base character

`tests/styled_combining_marks.rs` `a_styled_mark_without_a_base_character_keeps_its_position_and_style` fails:

```
expected: \u{301}{fg=red}X{/}Y
actual:   {fg=red}\u{301}X{/}Y
```

In Ink, a nested `<Text color="red">` becomes `chalk.red(...)` inside the squashed string, and `sanitizeAnsi` (`INK/src/sanitize-ansi.ts`, ported in `src/text/sanitize.rs`) moves an SGR that precedes combining marks to after them, so marks keep the style of what precedes them. When nothing precedes them, they keep the enclosing style. chalk re-opens its style after each newline, so this holds at the start of every line of the span. A top-level Text's own style is applied at write time (`output.ts`) without sanitizing, so it styles every cell including leading marks.

Implement that rule for nested spans: a nested span's style starts at its first grapheme that is not made only of marks, on each line; the leading marks keep the style from outside the span. The leaf's own line transform at paint stays as it is. `styled::append` already merges a mark into a preceding grapheme; make the two rules read as one design, not two patches. The other tests in `tests/styled_combining_marks.rs` must keep passing.

## 2. `Frame::serialize`

`src/paint.rs` `Frame::serialize` is one long chained line that rustfmt cannot format. Split it so it reads and formats cleanly (for example a function that turns one row into a styled line, trimming trailing default blanks as Ink's `trimEnd()` does).

## 3. Absolute positions in paint

`layout::get` walks every ancestor to find the absolute position it needs for Yoga's rounding, and paint calls it for every node, so a paint costs O(nodes × depth). M2 repaints every frame. Pass the parent's unrounded absolute position down the paint walk instead (and wherever else `get` is called) so the lookup is O(1). Keep the rounding rule unchanged.

## Rules

- No `unsafe`, no new dependencies (`CARGO_NET_OFFLINE=true`), no new tests, no expectation changes, do not edit `tests/` or `src/**/tests.rs`.
- Match the surrounding code; comments explain why and cite the Ink file. Run `rustfmt --edition 2024` on every file you touch.
- Run only:
  - `cargo test --lib`
  - `cargo test --test styled_combining_marks --test render_to_string --test text --test text_width --test overflow --test content_offset --test borders --test background`
  - `cargo clippy --all-targets -- -D warnings`

  `content_offset` has one known failure (`disjoint_nested_clips_do_not_overwrite_content_when_a_wide_character_straddles_the_edge`, a Yoga flex-basis difference handled later); every other test in these targets must pass.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target` and delete it when done.
- Your sandbox cannot write `.git`, so leave the work uncommitted. In the report, list the commits you would make (message and files), one logical change each.

## Report

1. Proposed commits (message and files).
2. The rule you implemented for marks, in one paragraph, and where it lives.
3. Results of the commands above, with any failing test and your diagnosis.
4. Open questions.
