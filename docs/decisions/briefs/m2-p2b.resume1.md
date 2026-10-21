Review of your M2 phase 2b work so far. The design is right: the runtime split, the throttle, the alternate-screen replay and the uncapped non-TTY output match the brief. Continue and finish the phase, with these answers and changes.

1. **The failing column test is a transcription error in your port, not an Ink expectation.** Ink asserts `cursorTo(29) + eraseEndLine + count`: `Counter: ` starts at column 20, the end of the 20-column left box, so the gap after `UNCHANGED LEFT` (14 cells) is 6 spaces. The stop rule protects Ink's recorded expectations. A value you derived wrongly while porting is yours to fix: check it against the Ink source and `expectations/`, and say in the report which ones you corrected. In that test, import at the top of the file instead of writing `crate::` paths inline.
2. **The compile errors** in `tests/render.rs` and `tests/issue_973_static_commit.rs` (Static closures returning `Text`): fix them.
3. **Unmount writes.** Two operations are right: the pending frame of step 1 is a frame operation with its own write, and the restore of step 3 is another write. Ink writes them separately as well.
   - The alternate-screen exit carries no bsu/esu: Ink's `finishUnmount` writes `exitAlternativeScreen` and `?25h` without synchronization, and nothing may follow them. Remove the `synchronize` call on the empty buffer in that branch and its comment.
   - While the thread is panicking, write the restore of step 3 before dropping the runtime. The cleanups of step 2 run user code that can panic again, which aborts the process; the terminal must be restored by then. The normal path keeps the order 1, 2, 3. Put the reason in a comment.
4. **`Runtime::paint` runs `layout::compute`.** Keep it if it is the simplest way to cover a paint with no commit after the Static item unmount (the non-interactive final frame). Say in the report what it costs when the tree is clean (taffy's root cache), and update the comment in `after_write` that says every later present goes through the commit's layout pass: it no longer holds.
5. **`src/testing.rs`:** merge the two `use crate::...` groups.

Then finish the rest of the brief: run every listed test to its assertions, fix what is yours, update `docs/ink-tests.md` (rows, 差异 entry for the height-cap adaptations, the not-ported lists, the APIs this phase does not add, `write-synchronized.tsx`) and `docs/design.md` (non-TTY interactive output is not capped, and anything else you decided). Same rules as before: stop and report only on an Ink expectation that fails for a reason other than the height cap.

Run only:
- `cargo test --lib`
- `cargo test --test render --test clear_rerender --test issue_973_static_commit --test components --test static_runtime_blank_lines --test measure_element --test runtime --test render_to_string --test static_string_replacement`
- `cargo clippy --all-targets -- -D warnings`

Report in the seven sections of the brief.
