# ink-rs: write a frame cut by the height cap in full at unmount

You are working in a git worktree of ink-rs at `<scratchpad>/wt/final-frame`. Read `CLAUDE.md`, then in `docs/design.md` the bullets 非全屏模式的规则, 光标停留的位置 and 恢复终端. Then read:

- `src/output.rs` (all of it) and `src/output/tests/mod.rs` (style only);
- `src/app.rs`: `AppOutput` (`restore`, `restore_bytes`, `paint`, `clear`, `external_write`) and `App::unmount`;
- `tests/render.rs` (`column`, the `issue_935_*` test) and `tests/common/mod.rs` (`Screen`);
- `src/testing.rs` (`TestTerminal::write`, `mount`, `TestApp::unmount`).

Ink checkout, read only; never run anything from it: `<scratchpad>/ink-src/ink`. Reference: `src/ink.tsx` `shouldClearTerminalForFrame`, `renderInteractiveFrame` and `unmount`.

## 1. Behavior

Today the inline output caps the live region at the screen height and drops the frame's top rows: cut rows never enter scrollback. That stays for every frame while the app runs. Ink writes an overflowing frame in full, so after Ink exits the whole last frame is in scrollback and on screen.

New rule: at unmount, inline interactive output on a TTY, when the last painted frame is taller than the viewport, erase the visible part of the frame and rewrite the whole frame without the cap. The cut rows enter scrollback once, followed by the rest of the frame. Then park below the frame as `done` does today. The visible screen is the same as before (the frame's last `rows - 1` rows and a blank row for the shell); scrollback gains the cut rows, each once.

Unchanged:
- alternate screen (discarded on exit, as in Ink);
- non-interactive output (already writes the full final frame);
- interactive output on a non-TTY (renders with `rows = usize::MAX`, never cuts);
- a suspended app (`restore` returns before this);
- `clear()` followed by unmount: the output holds no frame, nothing is rewritten (Ink keeps `clear()` + unmount clean);
- the restore snapshot from `restore_bytes`, used by signal and panic-hook emergency restore: it stays capped. Do not put a full frame into every write's snapshot.

## 2. Design

`src/output.rs`, a new method next to `done`:

```rust
/// Unmount: rewrites a frame whose top the height cap cut, so the cut rows
/// enter scrollback once. A cleared output stays clean.
pub(crate) fn uncap(&mut self, frame: &Frame, columns: usize, output: &mut Vec<u8>) {
    // `frame` is the last rendered frame; `live` holds the rows it kept.
    if self.live.is_empty() || frame.live.len() <= self.live.len() {
        return;
    }
    self.clear(output);
    self.render(frame, None, columns, usize::MAX, output);
}
```

- `live.is_empty()` means cleared or never rendered. An empty frame renders one blank live row, so `frame.live.len() <= self.live.len()` also covers it.
- `clear` already handles a declared cursor (returns to the top from the real cursor) and uses EL per row otherwise, so it does not trigger tmux `scroll-on-clear`.
- `render` with `rows = usize::MAX` writes every row, CRLF at the bottom margin scrolls the top into scrollback, and parks on the row below the frame (`full` is false). Pass `None` as the cursor: `done` moves to the rest position and shows the cursor.

`src/app.rs`, `AppOutput::restore`, the non-alternate branch:

```rust
if self.interactive {
    if let Some(frame) = &self.last_frame {
        self.output.uncap(frame, self.columns, &mut final_output);
    }
    self.output.done(&mut final_output);
}
```

`restore` already wraps the bytes in synchronized output on a TTY, so the erase and rewrite reach the terminal as one update. `last_frame` holds the frame without its Static lines; that is what `uncap` needs. `App::unmount` paints a pending frame before `restore`, so `last_frame` is the final frame. The panicking branch of `unmount` also reaches `restore`; let it rewrite too, no special case.

If the borrow checker or the code reads better with a different shape, keep the behavior and the method boundary (Output decides whether the frame was cut; AppOutput supplies the frame), and report the change.

## 3. Test

One integration test in `tests/render.rs`, placed after the `issue_935_*` test. It is an ink-rs scenario, not a port; name it `unmount_writes_a_frame_taller_than_the_viewport_in_full`.

- `TestTerminal::new(100, 5)`; write `"$ app\r\n"` as shell history with `terminal.write`;
- mount `column((0..8).map(|n| format!("line {n}")))` with `Options::default()`, then `app.unmount()`;
- `Screen::new(&terminal, 100, 5)`;
- assert that scrollback followed by screen rows, with trailing blank rows removed, equals `["$ app", "line 0", …, "line 7"]` (every frame row exactly once, no duplicates);
- assert the cursor is at row 4, column 0 and visible.

Before the change the cut rows `line 0`–`line 2` are missing from this history, so the test fails on master. Check that.

No other new tests.

## 4. Existing tests

Some tests may assert the old capped exit, for example a scrollback check made after unmount on a frame taller than the screen (`tests/render.rs` overflow cases, `tests/cursor.rs`, `tests/examples.rs`, `tests/errors.rs` with a tall error overview). If one fails:
- if it asserts state after unmount and the new result matches what the original Ink test expects, change the assertion to the new result and report the test, the old and the new expectation;
- if it fails for another reason, fix the code, not the test;
- do not weaken or delete assertions.

## 5. Rules

- No unsafe code. No new dependencies.
- `cargo fmt`. Comments sparse, in the style of the surrounding code.
- Do not edit `docs/`. I update the docs at integration.
- Run only these commands, all with `--offline`:
  - `cargo build --offline`
  - `cargo test --offline --lib`
  - `cargo test --offline --test render --test cursor --test cursor_exit_position --test examples --test errors --test clear_rerender --test issue_973_static_commit --test terminal_resize --test static_runtime_blank_lines`
  - `cargo clippy --offline --all-targets -- -D warnings`
  Do not run the full `cargo test`. I run it at integration.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## 6. Report

Your final message is the report (it goes to the `-o` file):
- what you changed, by file;
- every deviation from this brief and why;
- every existing test you changed, with the old and new expectation;
- the exact commands you ran and their results, including the new test failing on the unchanged code.
