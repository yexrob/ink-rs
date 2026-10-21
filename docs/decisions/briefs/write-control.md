# ink-rs: out-of-band control writes

You are working in a git worktree of ink-rs at `<scratchpad>/wt/write-control`. Read `CLAUDE.md`, then in `docs/design.md` the bullets about external writes (`use_stdout`), 非全屏模式的规则 and the suspension notes. Then read:

- `src/runtime/output.rs` (all of it);
- `src/app.rs`: `AppOutput` (all methods), `App::render_if_due`, `App::paint`, `App::suspend`, `App::resume`, `App::unmount`;
- `src/testing.rs`: `DebugOutput` and its `WriteDestination` impl, `TestTerminal` (`writes`, `output`), `mount`, `TestApp` (`step`, `advance`, `suspend_terminal`, `resume_terminal`, `unmount`);
- `src/runtime/hooks.rs`: `use_stdout`, `use_stderr`;
- `tests/suspension_output.rs` (style: capturing an `OutputWriter` in the test body).

Another worker changes `src/driver/`, `src/runtime/hooks.rs` (a new hook) and `src/runtime/mod.rs` at the same time. Stay out of `src/driver/` and `src/runtime/mod.rs`; in `hooks.rs` touch only the doc comments of `use_stdout`/`use_stderr` if needed.

## 1. Why

Apps need to send escape sequences the frame model cannot hold: terminal graphics (kitty image transmission), OSC title, OSC 52 clipboard, OSC 8 setup. `OutputWriter::write` is for log lines: it erases the frame, writes the text, adds a line break and redraws. Writing to the process stdout directly races the frame diff. The new call queues raw bytes and sends them after the frame that includes the caller's state changes, without touching the frame.

## 2. API

```rust
impl OutputWriter {
    /// Queues raw bytes, such as a graphics or OSC sequence, for this stream.
    /// ...
    #[track_caller]
    pub fn write_control(self, bytes: impl AsRef<[u8]>);
}
```

Same checks as `write`: UI thread assertion, `assert_not_rendering`, empty input ignored, no store → dropped, `unmounting` → ignored, destination looked up the same way. Add `fn write_control(&self, stream: OutputStream, bytes: &[u8])` to `WriteDestination`; no default body.

Doc comment content (write it in the style of `write`):
- The bytes go out unchanged after the frame that includes every state change made before the call. When that step paints nothing, they go out at the end of the step.
- The frame is not erased or redrawn, no line break is added, and the bytes do not count toward the frame height.
- Bytes that print text or move the cursor without restoring it put the terminal out of step with the next frame's diff.
- While the terminal is handed over (`suspend_terminal`, Ctrl+Z), interactive apps keep the bytes until the first frame after the handover ends. Unmount during a handover drops them.
- Without a driver (`testing::render`), the bytes are dropped.

## 3. Behavior

`AppOutput` gains a queue `control: Vec<(OutputStream, Vec<u8>)>`. `WriteDestination::write_control` for `RefCell<AppOutput>` only pushes onto it. A flush sends each entry in queue order as its own `Terminal::write` (stdout) or `Terminal::write_stderr` (stderr), with `self.restore_bytes()` as the restore snapshot, not wrapped in synchronized output, then empties the queue.

Flush points:
1. `AppOutput::paint`, after the frame bytes are written, interactive or not, including an empty diff. This covers throttled frames, resize frames, the first frame after resume, and the final paint in `unmount`.
2. The end of `App::render_if_due`, after the raw/paste mode sync and before the exit check, when no frame waits: non-interactive, or interactive and none of `suspended`, `self.pending`, `runtime.has_work()` holds. `has_work` covers state set by passive effects in this step: the next step commits it and its paint flushes. Non-interactive apps never paint live frames, so their bytes go out at the end of each step.
3. `App::unmount`: after the interactive/non-interactive paint block and before `runtime.begin_unmount()`, flush unless interactive and suspended; in that case clear the queue. The panicking branch clears the queue without writing.

Not flush points: `AppOutput::external_write` (it redraws the last painted frame, not the next one) and `AppOutput::clear`. Queued bytes stay queued across them.

`DebugOutput` (`testing::render`) drops control writes.

Update the `use_stdout` doc comment in `hooks.rs` with one sentence pointing to `OutputWriter::write_control` for raw sequences, if it fits there; otherwise leave it.

## 4. Tests

New file `tests/write_control.rs`. In-process with `testing::mount` and `TestTerminal`; capture the `OutputWriter` and a `State` setter (or `AppHandle`) in the test body as `tests/suspension_output.rs` does, then call `write_control` from the test body and step with `app.advance(..)`. Use ASCII payloads such as `"\x1b]2;title\x07"` (`TestTerminal` stores UTF-8). Assert with `terminal.output()` / `terminal.writes()` order. One test per flush rule:

1. `control bytes follow the throttled frame of an earlier state change`: interactive TTY, `max_fps` low enough that a second change falls in the throttle window. Change the state, `write_control`, step: neither the new frame nor the bytes are written. Advance past the window: the frame write with the new content, then the control write as the next write, unchanged.
2. `control bytes go out in the same step when no frame waits`: no state change; `write_control`, one step: exactly one new write, equal to the bytes (no erase, no redraw, no line break).
3. `control bytes wait for the first frame after a terminal handover`: `suspend_terminal()`, `write_control`, step: nothing is written. `resume_terminal()`: the redrawn frame, then the bytes.
4. `non-interactive control bytes go out at the end of the step`: `interactive: Some(false)`, change the state and `write_control`, one step: the bytes are written in that step.
5. `unmount writes queued control bytes after the final frame`: throttled state change and `write_control` as in 1, then `unmount()`: the final frame write, then the bytes, then the restore write.

No other new tests. Before the change, the file fails to compile (the method does not exist); that is the expected failing state.

## 5. Rules

- No unsafe code. No new dependencies.
- `cargo fmt`. Comments sparse, in the style of the surrounding code; one comment at each flush point saying why it flushes there.
- Do not edit `docs/`. I update the docs at integration.
- Run only these commands, all with `--offline`:
  - `cargo build --offline`
  - `cargo test --offline --lib`
  - `cargo test --offline --test write_control --test suspension_output --test suspend_terminal --test hooks --test render --test cursor --test errors`
  - `cargo clippy --offline --all-targets -- -D warnings`
  Do not run the full `cargo test`. I run it at integration.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## 6. Report

Your final message is the report:
- what you changed, by file;
- every deviation from this brief and why;
- the exact commands you ran and their results.
