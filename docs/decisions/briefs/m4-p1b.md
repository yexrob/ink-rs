# ink-rs M4 phase 1b: `use_box_metrics`

You are working in a git worktree of ink-rs at `<scratchpad>/wt/m4-p1b`. Read `CLAUDE.md`, then `docs/design.md`: 运行时 (commit order, layout effects), 测试, and 移植 Ink 测试的方法. Then read:

- `src/runtime/state.rs` (`ElementMetrics`, `NodeRef`, `NodeRef::refresh`, `measure_element`, `Store`);
- `src/runtime/mod.rs` (`Runtime::commit`: the ref detach and attach, the `self.refs.retain` refresh after `layout::compute`, the layout-effect rounds; `after_write`; `Instance::retire`);
- `src/runtime/hooks.rs` (`use_node_ref`, `use_memo`, `use_focus_manager`);
- `src/runtime/focus.rs` (`FocusState::add_reader` and how readers get marked: the registry pattern to follow);
- `src/layout.rs` (`layout::get`);
- `tests/measure_element.rs` and `tests/terminal_resize.rs` (test patterns).

Design reference (read §2, §3 useBoxMetrics, §4, §5.1, §5.4 and §6 items 8 and 11): `<scratchpad>/m4/research-animation-metrics.md`. The user approved its recommendations; the decisions below restate them.

Ink checkout, read only, never run anything from it: `<scratchpad>/ink-src/ink`. Sources: `src/hooks/use-box-metrics.ts`, `src/measure-element.ts`, `src/reconciler.ts` (`resetAfterCommit`, the layout listeners), `src/dom.ts`. Tests: `test/use-box-metrics.tsx`.

Another worker builds `use_animation` in parallel. It changes `Runtime::render_if_due`, `App::render_if_due`, `src/testing.rs`, and adds a `Store` field after `cursor` and a hook after `use_window_size`. Do not touch those. Put your `Store` field right after `focus` (struct and `Store::new`), and put `use_box_metrics` right after `use_node_ref` in `hooks.rs`.

## 1. What to build

### 1.1 API

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoxMetrics {
    pub width: u32,
    pub height: u32,
    pub left: i32,          // relative to the parent's layout box, no scroll offset
    pub top: i32,
    pub client_width: u32,  // width minus borders, padding included
    pub client_height: u32,
    pub has_measured: bool,
}
impl Scope { pub fn use_box_metrics(&mut self, node_ref: NodeRef) -> BoxMetrics; }
```

Export `BoxMetrics` from the crate root. Document the hook: the field meanings; zeros and `has_measured: false` before attachment and after detachment; the values are available in the frame where the View first lays out (see 1.4); and an unchanged layout schedules nothing.

### 1.2 NodeRef stores attachment

The ref slot becomes `State<Option<NodeLayout>>`, with a crate-private `NodeLayout { metrics: ElementMetrics, left: i32, top: i32 }`. `None` means detached.
- Attach and refresh in `commit` write `Some`. `left` and `top` come from the bound node's own `layout.location`, which is relative to its parent, from the same `layout::get` walk that computes `x` and `y`.
- Detach in `commit` and the Static item retirement in `after_write` write `None`.
- `measure_element` keeps its public contract and returns zeros for `None`.
- A refresh still does not schedule anything (React ref attachment does not).
- Use this shape, not a separate attached flag: `None` already says "detached", and a second field would have to stay in step with it.

### 1.3 Readers

Add a `Store` registry of box readers, following the focus readers: weak entries, with dead or unmounted ones pruned. A reader holds the `NodeRef` from its last render, the metrics that render returned, and the owning component's `Dirty`.

The hook memoizes one reader per hook slot and registers it on first render. Each render stores the current `node_ref`, computes the metrics from the slot (`has_measured` is `Some(..)`), stores them as seen, and returns them.

### 1.4 Commit check

In each round of `Runtime::commit`, after the ref refresh, compare every live reader's current metrics with the metrics it last returned. A difference marks the reader's `Dirty`, so the round loop re-renders before paint.

This covers every Ink trigger:
- memoized components (every ink-rs component is memoized);
- sibling changes;
- refs attached by children after the first render;
- detach and unmount of the tracked View;
- terminal resize, which commits through `layout_pending`.

Idle has no commit, so it has no check and no wakeups. An unchanged layout marks nothing, so there is no extra render.

Difference from Ink: Ink paints zeros first and delivers metrics one frame later (passive effect plus microtask). ink-rs shows the metrics in the first frame. Report it as a difference; Ink's tests assert the final output, so their expectations hold.

## 2. Tests

Create `tests/use_box_metrics.rs`. Its header cites `test/use-box-metrics.tsx` (Ink 8.0.0, commit 26d2c3f). Keep Ink's test names in snake_case and Ink's order.

Mapping:
- Ink's `waitUntilRenderFlush`, `delay(50)` and `act()` become `settle()`.
- Resize tests use `testing::mount`, then `app.resize`, then `advance` past the throttle window, as `tests/terminal_resize.rs` does.
- Render counts and values captured during render go into thread_local counters and vectors.

Port 16 of 19:
- Values (5): returns correct size on first render; returns correct position; returns zeros when ref is not attached; returns viewport client size and content size for overflowing content; content size updates when content grows.
- Update triggers (6): updates when terminal is resized; uses latest tracked ref when terminal is resized; updates when sibling content changes; updates when sibling content changes but tracked component is memoized; updates when tracked ref attaches after initial render and component is memoized; does not trigger extra re-renders when layout is unchanged.
- hasMeasured and detach (5): hasMeasured becomes true when tracked element is mounted on initial render; hasMeasured resets when tracked ref switches to a detached element; hasMeasured becomes true after the tracked element is measured; resets metrics when tracked element unmounts; tracks a box mounted by child state - concurrent=false.

Not applicable (3):
- `tracks a box mounted by child state - concurrent=true` duplicates the concurrent=false case.
- `removes resize listener on unmount` tests Node's EventEmitter, and unmount consumes the app (the useWindowSize precedent).
- `does not crash when resize fires after unmount` has the same reason.

If a test that counts renders or records every rendered value differs because the metrics arrive one frame earlier, keep Ink's final-state assertion. Then adapt the count, with the reason in a comment.

No new tests beyond the port unless a rule has no other coverage; name the rule if you add one.

## Rules

- No `unsafe`. No new dependencies.
- No code that exists only to satisfy a test.
- Match the surrounding code: use `pub(crate)` or narrower for internal items, put imports at the top, and write comments that say why and cite Ink where the code ports it. Run `rustfmt --edition 2024` on every file you touch.
- Do not edit `docs/`. Report the documentation changes instead.
- Run only the following. Every test in these targets must pass.
  - `cargo test --test use_box_metrics --test measure_element`
  - `cargo test --lib runtime`
  - `cargo test --test hooks --test reconciler --test runtime --test components --test terminal_resize --test static_runtime_blank_lines --test issue_973_static_commit --test focus`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration. If the sandbox has no network, add `--offline`.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

Write the report in English.

1. Proposed commits, with message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. The design as built: the NodeRef slot, the reader registry and pruning, and where the commit check runs.
3. Where the behaviour departs from Ink, with the reason.
4. The ported tests: names, every assertion you adapted with the reason, and the tests you marked not applicable.
5. Results of the commands above. Name any failing test, with your diagnosis.
6. The rows to change in `docs/ink-tests.md`, what `docs/design.md` should record, and open questions.
