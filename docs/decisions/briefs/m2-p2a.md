# ink-rs M2 phase 2a: `<Static>` across frames

You are working in `<repo>`. Read `CLAUDE.md`, then `docs/design.md` (编程模型, 一帧, and the Static bullets under 终端输出), then `docs/ink-tests.md`.

M1 and M2 phase 1 are committed:
- `src/runtime/` mounts components, keeps a tree mounted across frames (`Runtime::render_if_due`) and runs effects in React's order.
- `src/testing.rs` drives the runtime and records one string per presented frame.
- `src/output.rs` turns frames into terminal bytes. It takes `Frame::static_lines` as the rows that are new in this frame and keeps no Static history.

Read that code before you change it.

References, all read only:
- The Ink checkout at `<scratchpad>/ink-src/ink`. Read it; never run anything from it. The relevant sources:
  - `src/components/Static.tsx`
  - `src/reconciler.ts` (`resetAfterCommit`, `findStaticNode`, `clearStaticNodeIfContained`)
  - `src/renderer.ts`
  - `src/ink.tsx` (`handleStaticChange` and the debug branch of `onRender`)
- `<scratchpad>/m1-research/runtime.md` §1.5 and §4.1: how Ink commits a Static append.
- `<scratchpad>/m2-research/tests-render.md`: the `components.tsx` table (C1-C11) and the `static-runtime-blank-lines.tsx` table (SB1-SB8).
- `<scratchpad>/expectations/components.tsx.md` and `static-runtime-blank-lines.tsx.md`: values recorded from a run of Ink's tests. Expected values come from there.

## 1. What changes

Today the runtime renders every Static item on every commit, and `Frame::static_lines` holds all of them. After this phase, each item is painted once.

### The runtime

- **Written items.** Each mounted Static instance remembers how many items it has written.
  - It renders items `[written..len]` only, and calls the render closure with each item's absolute index.
  - When it has nothing to render, its host node gets `display: none`, as today for `len == 0`.
  - During reconcile, `written` is lowered to `len` when the items shrink. Ink's layout effect sets `index = items.length` on every length change (`Static.tsx`), so a later append renders from the new length.
- **After present.** Once a frame is written, every Static that rendered items sets `written = len` and unmounts the item instances. This covers each Static in the tree, not just the one paint uses: Ink's layout effect runs in every Static.
  - Order:
    1. paint;
    2. write;
    3. the commit's passive phase as today (passive cleanups, dropped states, passive setups);
    4. then the item unmount, which drains its layout cleanups, passive cleanups and states at once.
  - The source of this order is React: a sync update scheduled in a layout effect (Ink's `setIndex`) flushes the pending passive effects before it renders (`flushPassiveEffects` in `performSyncWorkOnRoot`). The items' own passive setups run before their cleanups.
  - The unmount needs no second commit and does not schedule a render. When no Static rendered items in this commit, the step costs nothing: no tree walk.
- **Host tree consistency.** The unmount changes the host tree after layout. The next paint must see a layout of the current host tree. Say in the report how you guarantee it.
- **Static identity.** Ink resets its accumulated static output when the first Static in tree order changes identity: mount, unmount, a key-driven remount, or the removal of an ancestor (`resetAfterCommit`, `handleStaticChange`).
  - Identity is the Static instance, not its `NodeId`. A remount that reuses the same arena slot is a new identity.
  - The first Static in tree order is chosen as paint chooses it today. A Static under a `display: none` ancestor still counts for identity, and paints nothing.
  - The frame tells its consumer about the change. A suggested shape is a `Frame` field that is true when the first Static differs from the one at the previous paint. Choose the shape and explain it in the report.

### `Frame` and its consumers

- `Frame::static_lines` keeps its meaning: rows new in this frame.
- `render_to_string` keeps its output. It renders one frame, and every item is new in it.
- `src/output.rs` needs no change.

### `testing::render`

It stands in for Ink's debug mode (design.md 测试, layer 2). Ink writes `fullStaticOutput + output` on every render (`src/ink.tsx`, the debug branch).

- `TestInstance` keeps the static rows written so far and resets them when the Static identity changes.
- Each recorded frame is the accumulated static rows, each followed by `\n`, then the live rows joined by `\n`.
  - Ink's static output is the static grid plus `\n` when its height is > 0 (`src/renderer.ts`).
  - A frame with static rows and no live rows therefore ends in `\n`, as in Ink.
- Keep the accumulation in `src/testing.rs`, not in the runtime. Update the rustdoc of `last_frame` and `frames`.

### Throttle

M2 phase 2b adds the throttle. In it, a commit that renders new Static items, or changes the Static identity, writes at once. This phase has no throttle. Do not build one, and do not block that rule.

## 2. Tests to port

- **Into `tests/components.rs`, in Ink's order among the tests already there:** C1-C11. Use Ink's names, converted as in the rest of the file.
  - The Ink tests use `render(.., {debug: true})`. Port them with `testing::render`.
  - A `stdout.get()` or `lastCall.args[0]` is `last_frame()`.
  - C7 and C8 use two instances; create two `TestInstance`s on the same thread.
  - C11's last assertion reads the last write after `unmount()`. Ink's unmount writes no new content here, because no render is pending and static output has been rendered (#397). Assert the last frame before `unmount()`, and say so in a comment.
- **Into a new file `tests/static_runtime_blank_lines.rs`:** SB1, SB3, SB5, the debug-output cases.
  - SB2, SB4, SB6 (non-interactive) and SB7 (screen) need the app layer of phase 2b. Do not port them.
- **`docs/ink-tests.md`:**
  - Update the `components.tsx` and `static-runtime-blank-lines.tsx` table rows.
  - In the M2 not-ported list of `components.tsx`, take out the 11 tests you ported. Do not add a list of ported tests.
  - Leave the overview counts alone.
  - Check the 差异 entry "Static 追加时写出一次" against what you built, and update it only if it no longer holds.

## Rules

- No `unsafe`. No new dependencies (`CARGO_NET_OFFLINE=true`).
- No tests beyond those listed, and no change to an existing expectation. If one would need to change, stop and report.
- No code that exists only to satisfy a test.
- Match the surrounding code:
  - `pub(crate)` for internal items, imports at the top.
  - Comments explain why and cite the Ink or React source a rule comes from. Do not cite the IDs (C1, SB1, ...) of the research tables in code or docs; they exist only in the scratchpad.
  - Rustdoc on public items.
  - Run `rustfmt --edition 2024` on every file you touch.
- Update `docs/design.md` only where what you build differs from it, and say so in the report.
- Run only the following. Every test in these targets must pass, apart from `#[ignore]`d ones.
  - `cargo test --lib`
  - `cargo test --test components --test static_runtime_blank_lines --test static_string_replacement --test static_blank_lines --test static_trailing_layout --test runtime --test reconciler --test measure_element --test background`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target`, and delete that directory when you are done.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

1. Proposed commits: message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. How written items are tracked, when items unmount, and how the host tree stays consistent with the layout.
3. How the Static identity is defined and reported, and the cost per frame.
4. Results of the commands above. Name any failing test, with your diagnosis.
5. Where you departed from this brief or from design.md, and open questions.
