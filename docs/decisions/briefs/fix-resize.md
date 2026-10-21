# Task: erase frame rows that a terminal pulls back from scrollback when it grows taller

## The bug

Reproduced in tmux. A 28-row inline frame on a 30-row screen; the window shrinks to 20 rows, ink-rs redraws; the window grows to 50 rows, ink-rs redraws. Afterwards rows 00–08 of the frame appear twice: stale copies above the live frame.

1. **Shrink.** The terminal (tmux, and kitty, iTerm2, VTE alike) drops blank lines below the cursor, then pushes `cursor_row + 1 - new_rows` rows from the top of the screen into scrollback. With the frame at the bottom of the screen, those are the frame's top rows.
2. ink-rs then redraws. `Output::resize` (`src/output.rs` around lines 160–205) erases and redraws the capped frame from screen row 0 (`CSI 1;1H` + erase down for a full frame; `clear` for a declared cursor; the width path's `CUU` stops at the screen top too). The pushed rows stay in scrollback as stale copies.
3. **Growth.** The terminal pulls `min(lines pushed into history, growth)` lines back from scrollback onto the top of the screen and moves the cursor down by the same amount. The newest scrollback lines are the stale frame rows.
4. `src/app.rs` `paint` (around lines 140–155) passes only `width_decreased` / `height_decreased` to `Output::resize`, so a growth falls through to the diff, which works relative to the frame top. The pulled-back rows above the frame are never erased.

Ink has the same bug (`resized` in `src/ink.tsx`). ink-rs fixes it.

## The fix

`Output` records how many frame rows it knows went into scrollback, and erases that many rows above the frame when the screen grows taller.

- New field on `Output`, e.g. `spilled: usize`, separate from `forget()` (the shrink paths call `forget`, and the count must survive it). Doc comment: frame rows the terminal pushed into scrollback since the frame was last drawn from the screen top; a nonzero value means the frame top is at screen row 0.
- **Record on shrink.** In each height-decrease path, add `(cursor_row + 1).saturating_sub(rows)` where `cursor_row` is the real cursor row relative to the frame top before the resize (`self.cursor.1`; for a declared cursor that is the declared row). Add it to the current value: a second shrink pushes more frame rows on top of the earlier ones.
- **Width decreases.** The terminal reflows, so a spilled row may have become several physical rows, and a later width increase rejoins them. Record a spill on the width path only when every row from the frame top through the cursor row fits the new width unchanged (`reflow` yields one row each); then the count is exact (`above + offset + 1 - rows`, saturating). Otherwise record nothing. Also track the widest spilled row; when a later resize makes the width smaller than it, drop the count to 0 (the rows rewrap; this stays the known limitation design.md already describes).
- **Erase on growth.** When the height grows and `spilled > 0`: hide the cursor, `CUU(cursor_row + spilled)` (it stops at the screen top, so it never reaches above what the terminal pulled back), erase down, `forget()`, and let the render that follows redraw. Then `spilled -= min(spilled, growth)`: rows beyond the growth are still in scrollback, and the frame top is at row 0 again, so the invariant holds. If the width also decreased in the same resize, combine the two erases into one upward move; work out the cases and keep the code readable.
- **Reset to 0** whenever bytes may scroll the screen, because the newest scrollback lines are then no longer frame rows: Static rows written in `render`, `clear` (an external write follows it), `done`, `uncap`, suspend/resume (find how `src/app.rs` resets `Output` for suspension). When in doubt, reset: a missed erase leaves today's behavior, an extra erase would delete the user's shell history.
- **Alternate screen** has no scrollback: record nothing there (`App` knows `self.alternate`). Non-TTY interactive output uses `rows = usize::MAX`, which records nothing by itself.
- Change the `Output::resize` signature as needed: pass the previous and new sizes instead of two booleans. Update the caller in `src/app.rs`.
- Keep the existing comments that cite Ink. Add a comment on the growth path that states the deviation from Ink.

A frame update between the shrink and the growth (a normal diff) keeps `spilled`: the frame top stays at row 0 and nothing scrolls, since the live region is capped at the screen height.

## Test helper

`Screen::resize` in `tests/common/mod.rs` models tmux for width reflow but says "Height growth does not pull rows from scrollback". Make height growth on the primary screen pull back `min(scrollback lines, growth)` lines and move the cursor down by that count, as tmux and most terminals do. The existing `reflow` rebuild can do this (rebuild with the screen top moved up by the pulled count), or find a simpler way with vt100. Update the doc comment. Alternate screens pull nothing back.

Then run the resize tests below. If an existing expectation changes because of the new model, decide whether the new expectation shows the fixed behavior; update it only then, and list each change and its reason in the report.

## Tests

Add to `tests/terminal_resize.rs`, using its `shell_frame` / `trimmed` helpers where they fit:

1. Shell lines, then a 28-row frame (`row 00` … `row 27`) on a 100×30 screen. Resize to 100×20 and render, then to 100×50 and render. Every `row NN` appears exactly once across scrollback plus screen, and every shell line is kept.
2. The same with a declared cursor (`use_cursor`) inside the frame.
3. A Static row printed between the shrink and the growth: no erase above the frame, shell lines and the Static row kept, and no duplicate frame rows on screen.

These cover the regression and the two ways the fix could delete real history. No other new tests.

## Docs

`docs/design.md`, the **resize** paragraph (around line 142): add the rule in the same style: on height decrease, Output counts frame rows the terminal pushed into scrollback; on height growth it moves up past them and erases, because tmux and most terminals pull those rows back; the count resets when other output scrolls the screen; width reflow records a count only when no row rewraps. State that Ink does not do this.

## Commands

- `cargo test --offline --test terminal_resize --test suspension_resize --test render --test use_box_metrics`
- `cargo test --offline --test examples --test examples_pty`
- `cargo test --offline --lib`

Manual repro scripts from the research are in `<scratchpad>/research/resize/` (read them for the byte sequences; do not start tmux servers).

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
