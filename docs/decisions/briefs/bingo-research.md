# Research: ink-rs changes for bingo (test frame, mouse)

## Context

ink-rs is a Rust port of Ink, the React renderer for terminal UIs. bingo, a coding agent built on ink-rs and eink-rs, sent requirement briefs. The user approved this plan: research every item now, agree on all the designs in one round, then dispatch one Codex worker per item, each in its own git worktree, at the same time.

This is research only. Change no files in the repository and do not commit. Write only the report file named below.

## Read first

- `<repo>/CLAUDE.md` and `<repo>/docs/design.md`.
- Your topic's requirement brief, named below. The bingo session wrote it. Its "Change" section is a wish list, not an agreed design: check each premise against the code.

## Topics

The prompt gives you one topic.

### test-frame

Requirement: `<repo>/docs/decisions/briefs/bingo-test-frame.md`.

1. How `testing::render` produces `TestInstance::last_frame` and `frames`, and how `testing::mount` and `TestApp` drive `App` with a `TestTerminal`. Where the renderer keeps its frame grid after a paint (record 063), and whether `TestApp` can read it without parsing bytes, in inline mode and with `alternate_screen`.
2. How `Static` items are written (once, above the live frame), and where a `static_output` could collect their text. The text format: the same as `last_frame` (styles stripped? trailing spaces? wide characters?).
3. Paint timing. `TestApp` paints on `max_fps` windows. What "after the latest paint" means for a test that sends input and reads at once: can a paint be pending, and how do the existing `TestApp` tests wait for one?
4. What a test cannot read with the proposed API: colors, the cursor position, images, a frame between paints.
5. Overlap with the other topics in flight: signal ownership changes `src/driver/` and adds a process-level setting; mouse changes `src/input/`, `src/runtime/input.rs` and `src/app.rs`.

### mouse

Requirement: `<repo>/docs/decisions/briefs/bingo-mouse.md`. bingo needs it after the other items; design it now so its worker can run in parallel.

1. Parser: where SGR 1006 reports (`CSI < b ; x ; y M` and `m`) fit in `src/input/parser.rs`, beside the string sequences added in record 069. Split reads and the escape timeout. What to do with X10 or 1015 reports.
2. Modes: how bracketed paste and the kitty keyboard turn on while a hook holds them and off on exit, suspend and panic (`src/app.rs`, the driver's restore state). Which modes to send: 1000, 1002 or 1003 (hover needs any-motion reports), with 1006. The cost of 1003 and when to turn it on.
3. Windows: how ink-rs reads console input (`src/driver/sys/windows.rs`). Can mouse input work there, and how (`MOUSE_EVENT` records, `ENABLE_MOUSE_INPUT`, quick edit mode, VT input)?
4. Coordinates and hit testing:
   - What `use_box_metrics` and `use_node_ref` give, and relative to what.
   - Alternate screen: does the frame's row 0 sit on screen row 0?
   - Inline mode: the live frame's top row moves on the screen as `Static` output and the frame grow and the screen scrolls. Can ink-rs know the screen row of the frame's first line, from its own output accounting or a cursor position report (`CSI 6n`)? If it cannot know it in every case, say so and propose the rule, for example mouse in the alternate screen only, or coordinates counted from the frame's last row, which sits at the cursor.
   - The hit-test API: a node's screen rect, a mouse hook scoped to a node ref, or another shape.
5. Routing: how `use_input` handlers are ordered and how focus works, so a click can move focus. Whether mouse events go through `use_input` or a separate hook.
6. Text selection while reporting is on (Shift+drag in most terminals), and what to document.
7. Overlap with the other topics in flight: signal ownership (`src/driver/`), test-frame (`src/testing.rs`, the renderer).

## Report

Write the report in Chinese with no adverbs (no 再, 都, 还, 就, 也, 只, 仍, 已, 先, 直接, 最 and the like) to `<scratchpad>/research/bingo/<topic>.md`:

1. 现状：what exists today, with `file:line`.
2. 方案：the API as Rust signatures, where the code goes, the data flow. For each choice the requirement leaves open ("name open", "say which and why"), list the options and recommend one with the reason.
3. 需求里不成立或要改的前提.
4. 改动范围：files, rough size, overlap with the other topics.
5. 测试：the tests that are needed, with the reason for each. No tests for their own sake.
6. 待用户定的问题：at most three, each with your recommendation.

Keep it under 2,000 Chinese characters plus code. Reply to the main agent in five lines or fewer: the report path and the main recommendation.
