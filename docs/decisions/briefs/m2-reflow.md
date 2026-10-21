# ink-rs M2 follow-up: erase the reflowed frame when the terminal narrows

You are working in `<repo>`. Read `CLAUDE.md`, then the resize paragraph of 终端输出 and the 测试 section of `docs/design.md`, then the `terminal-resize.tsx` parts of `docs/ink-tests.md`. Read `src/output.rs`, the resize path in `src/app.rs` (`AppOutput::paint`), `tests/common/mod.rs` (`Screen`) and `tests/terminal_resize.rs` before you change them.

## The problem

When the terminal gets narrower, `Output::resize` calls `Output::clear`, which erases the previous frame by its painted row count, as Ink's `log.clear` does. Most terminals (tmux, iTerm2, Terminal.app, kitty, WezTerm, Ghostty) reflow on resize: every painted row longer than the new width wraps onto several rows. The erase then stops short of the frame's top and the old frame's head stays above the new frame.

Reproduced in tmux with `examples/counter.rs`: shell history, then the frame `68 tests passed`, then the window narrowed from 80 to 10 columns. The new frame was drawn correctly, and `68 tests p` remained above it.

The user decided: erase by the reflowed row count. This departs from Ink. The cost, accepted: a terminal that does not reflow (xterm) erases as many extra history rows above the frame as reflow would have added.

## 1. Output

On a width decrease, replace the call to `clear` with an erase that models reflow. Height decreases without a width decrease keep the current rules.

- Every live row is its own logical line: rows end with CR LF or cursor motion, never with an autowrap, so the terminal never joins two of them.
- A live row reflows at the new column count as the terminal lays out text: walk its glyphs left to right; a glyph of width `w` (one `Cell::Glyph` plus its `Cell::Continuation`s) starts a new physical row when it does not fit in the cells left on the current one and the current one is not empty. An empty row, and a row no wider than the new width, takes one row. A wide glyph wider than a one-column terminal takes a row of its own.
- The terminal keeps the real cursor at its cell in its logical line. The cursor is `self.cursor`: the rest position, or the declared cursor when the frame declared one. Its physical row within its line is the reflowed row holding its cell. A cell after the row's content continues the last reflowed row, `columns` cells per row (tmux maps the cursor by cell offset). The rest row below a short frame is an empty line, so its offset is 0.
- Distance to the frame top = the reflowed heights of the live rows above the cursor's row + the cursor's row within its own line. Move up that many rows, CR, `CSI J`, then forget the frame, as `clear` does. Hide the cursor first, as `clear` does.
- When the reflowed frame is taller than the screen, the move stops at the first row; the part that scrolled into history stays there. Ink has the same limit.
- On the alternate screen the frame starts at the first row (`src/app.rs` writes `CSI H` after entering it), and terminals do not reflow the alternate screen. Any extra upward motion stops at the first row, so the same erase is correct there; do not add a branch.

`Output::resize` needs the new column count. Choose the signature; keep the call in `AppOutput::paint` simple. `Output::clear` stays as it is: `Instance::clear` and the height-decrease-with-cursor rule use it, and no reflow happens there.

## 2. The test terminal model

`tests/common::Screen` runs output through vt100, which truncates rows on a width decrease. Give `Screen::resize` a reflow model for width decreases, so the resize tests describe the terminals the user targets:

- Applies only when the width decreases and the screen is not the alternate screen. Width increases keep the current behavior (no rejoining of wrapped rows); say so in the doc comment.
- Logical lines: scrollback rows, then screen rows. A row continues the previous one when vt100 marked the previous one as wrapped (`vt100::Screen::row_wrapped`, read through the scrollback offset as `Screen::scrollback` does). Blank rows below both the cursor and the last non-blank row are not lines.
- Rebuild the parser at the new width with the same row count and scrollback size, writing each logical line so vt100 wraps it itself and marks the wrap. Keep styles if `rows_formatted` makes that straightforward; text, wide glyphs and wrap marks must survive.
- Anchoring, as xterm.js and tmux behave: the old screen's first row stays the first screen row, and rows added by reflow fill the blank rows below the cursor; only when the cursor's row would fall below the last screen row does the screen scroll, by the smallest amount that keeps it visible, pushing the top into scrollback.
- The cursor goes to its cell in its logical line, mapped with the same rule as Output (cell offset; cells after the content continue the last row). Cursor visibility survives the rebuild.
- When both dimensions shrink, reflow first, then apply the existing row-shrink model.

Keep it small and readable; this is test infrastructure. Update the doc comment of `Screen::resize`.

## 3. Tests

Global rule for this project: add a test only when you can say why it is needed.

- The existing width-decrease tests in `tests/terminal_resize.rs` and `rerender_on_resize` in `tests/render.rs` paint rows as wide as the terminal at the top of the screen. Under the reflow model the old erase leaves residue there, so they now cover the under-erase. Run them first with the model and the old Output to confirm at least one fails, then with the new Output. Report which failed.
- They do not cover over-erase: none has history above a wide frame. Add two tests to `tests/terminal_resize.rs`, after the ported ones, marked in a comment as ink-rs additions (Ink erases by painted rows). Both use `shell_frame` (three shell rows above the frame) and assert, after the resize and `screen.update`, the full trimmed history and screen: the shell rows intact, the new frame, nothing of the old frame.
  1. The tmux case: a frame of one text row of about 15 cells, no declared cursor, 80 → 10 columns.
  2. A declared cursor inside the frame: three rows, each wider than the new width, the cursor on the second row at a column past the new width, so part of its own line reflows above it. The distance then sums reflowed heights and the cursor's offset row.
- No unit tests in `src/output/tests` for this: the two scenarios above check the same bytes through their effect on a reflowing screen.

## 4. Docs

- `docs/design.md`, resize paragraph of 终端输出: state the width-decrease rule as built, the departure from Ink and why, the cost on non-reflowing terminals, the cursor mapping, and the limit when the reflowed frame outgrows the screen. Replace the sentence that begins `tmux 实测`. Keep the rest where it still holds; the height rules do not change.
- `docs/design.md`, 测试 item 3: vt100 does not reflow; `Screen` now models reflow on width decreases (rules above) and still does not rejoin on growth. Remove `宽度重排等终端差异留给真实终端验收` or reword it to what still needs a real terminal.
- `docs/ink-tests.md`, `与 Ink 测试的差异`, the resize bullet: the width tests run under the reflow model, and the two ink-rs additions. Leave the counts in the tables alone: the additions are not ports.
- Write the docs in Chinese, matching their style. No adverbs.

## Rules

- No `unsafe`. No new dependencies.
- No code that exists only to satisfy a test.
- Match the surrounding code: `pub(crate)` for internal items, imports at the top, comments that say why and cite the Ink source when a rule comes from Ink. Run `rustfmt --edition 2024` on every file you touch.
- Run only the following. Every test in these targets must pass, apart from `#[ignore]`d ones.
  - `cargo test --lib output`
  - `cargo test --test terminal_resize --test render --test cursor --test clear_rerender`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration.
- Do not run anything on a real terminal; the tmux check runs at integration.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target`, and delete that directory when you are done.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

1. Proposed commits: message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. The erase as built: the reflow walk, the cursor mapping, the bytes for each case (rest row, full frame, declared cursor).
3. The Screen model as built, and where it departs from the rules above.
4. Which existing tests failed under the old Output with the new model; the results of the commands above. Name any failing test, with your diagnosis.
5. Doc changes; where you departed from this brief or from design.md; open questions.
