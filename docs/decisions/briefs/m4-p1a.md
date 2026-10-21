# ink-rs M4 phase 1a: `use_animation`

You are working in a git worktree of ink-rs at `<scratchpad>/wt/m4-p1a`. Read `CLAUDE.md`, then `docs/design.md`: 运行时 (scheduling: `dispatch`, `render_if_due`, the wake time, the throttle paragraph near line 74), 驱动 (the natural-end paragraph), 测试, and 移植 Ink 测试的方法. Then read:

- `src/runtime/mod.rs` (`Runtime::render_if_due`, `commit`, `Dirty`), `src/runtime/state.rs` (`Store`, `State::set`), `src/runtime/hooks.rs` (`use_state`, `use_layout_effect`, `use_window_size`), `src/runtime/input.rs` (`input_deadline`, `flush_input`: the precedent for a deadline held by the core);
- `src/app.rs` (`App::new` computes `window`; `App::render_if_due` returns the wake time);
- `src/testing.rs` (`TestInstance::settle`, `TestInstance::advance`, `TestApp::advance`), `src/render.rs`;
- `src/driver/mod.rs` (`run_loop`, `natural_end`);
- `tests/pty.rs` (pipe fixtures: `run_pipe`, `INK_RS_PTY_FIXTURE`), `tests/measure_element.rs` (thread_local capture pattern).

Design reference (read §1, §3, §4, §5.1–5.3 and §6): `<scratchpad>/m4/research-animation-metrics.md`. The user approved its recommendations; the decisions below restate them.

Ink checkout, read only, never run anything from it: `<scratchpad>/ink-src/ink`. Sources: `src/hooks/use-animation.ts`, `src/components/AnimationContext.ts`, `src/components/App.tsx` (the animation scheduler), `src/ink.tsx` (`renderThrottleMs`). Tests: `test/use-animation.tsx`, and in `test/render.tsx` the two tests `useAnimation can drive non-interactive process exit` and `useAnimation can drive explicitly non-interactive process exit` with their fixtures in `test/fixtures/`.

Another worker builds `use_box_metrics` in parallel and changes `NodeRef`, the commit loop's ref refresh and `use_node_ref`. Do not touch those. To keep the merge clean: put the new `Store` field after `cursor` (struct and `Store::new`), and put `use_animation` right after `use_window_size` in `hooks.rs`.

## 1. What to build

### 1.1 API

New module `src/runtime/animation.rs`; the hook goes in `hooks.rs`. No new hook `Kind`: the hook composes `use_state` and `use_layout_effect`.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationOptions {
    pub interval: Duration, // default 100 ms
    pub active: bool,       // default true
}
// Implement Default.

#[derive(Clone, Copy)]
pub struct Animation {
    pub frame: u64,      // elapsed / interval, rounded down
    pub time: Duration,  // since the subscription started
    pub delta: Duration, // since the previous delivered tick (first tick: since start)
    reset: /* private handle */,
}
impl Animation { pub fn reset(&self); }

impl Scope { pub fn use_animation(&mut self, options: AnimationOptions) -> Animation; }
```

Export `Animation` and `AnimationOptions` from the crate root. Document the hook as the other hooks are documented: the shared timer, pause and resume, `reset`, the throttle, and that an active animation keeps a live app running.

### 1.2 Semantics (Ink `use-animation.ts`, `App.tsx`)

- Interval: clamp to `1 ms ..= 2_147_483_647 ms` (Ink clamps to the `setTimeout` range). NaN, Infinity and negative inputs cannot be expressed in `Duration`.
- All values are zero at mount. A subscription starts at the current step time: `start = now`, `next_due = start + interval`, `last_render = start`.
- Pause (`active: false`) unsubscribes and freezes the values. Resume, an interval change and `reset()` zero the values and start a new subscription from now. Rerendering with the same options changes nothing.
- `reset()` zeroes the values and bumps a reset key that is part of the subscription dependencies. While paused, `reset()` zeroes the values and the animation stays paused. A copy of `Animation` from an earlier render resets the live hook (Ink: "reset is a stable function reference").
- Unmount removes the subscription.

### 1.3 Shared timer in the core

One scheduler per `Store` (a `RefCell<AnimationState>`), beside the input deadline. It holds the last step time and the entries `{interval, start, next_due, last_render, values: State<Values>}`.

- `tick_animations(now, throttle)`: record `now` as the step time. For each entry with `now >= next_due`: advance `next_due` with Ink's catch-up formula, `start + (floor((now - start) / interval) + 1) * interval`, so a late wake skips missed slots and `frame` jumps. Then skip the entry when `now < last_render + throttle`. Otherwise write `Values { frame, time: now - start, delta: now - last_render }` and set `last_render = now`. Collect the writes, release the `RefCell` borrow, then call `State::set`.
- `animation_deadline(throttle) -> Option<Instant>`: the minimum over all entries of the first grid point `start + k * interval` (k >= 1) that is `>= max(next_due, last_render + throttle)`. `None` when there are no entries. This drops the wakes that land inside a throttle window; the ticks that are delivered match Ink's.
- The throttle is passed on each call; the runtime does not store it. This changes design.md's "runtime 不接收节流参数" to "runtime 不保存节流窗口"; report the change and do not edit docs.
- A subscription made in a commit outside a step (rerender or unmount paths) uses the last recorded step time.
- `Subscription` is the layout effect's cleanup value. Its `Drop` removes the entry.
- Tick writes go through `State::set`, which calls `queue.notify()`. That costs one empty driver pass per tick, as input does. Do not add a quiet write path.
- No render-time zeroing. Ink zeroes during render to keep a stale frame off screen. In ink-rs, layout-effect state writes re-render before paint, so the zeros reach the same frame. If a ported test records every rendered value and fails on the extra stale render, add a committed-options check and report it.

### 1.4 Wiring

- `App::render_if_due`: call `tick_animations(now, self.window)` before the queue drain and commit. Fold `animation_deadline(self.window)` into the returned wake time, the same way `input_deadline` is folded in, interactive or not. Non-interactive fixtures must tick.
- `Runtime::render_if_due`, used by `TestInstance` and `render_to_string`: throttle `Duration::ZERO`, as Ink's debug mode. Return `Some(now)` when there is work, else the animation deadline. `render_to_string` runs one step and drops the runtime, so frame stays 0, as Ink without an App.
- `TestInstance::settle` loops while `step()` returns a due time `<= now`. Today it loops while `step().is_some()`, which would spin on a future deadline.
- `TestInstance::advance(d)` steps through each due time up to `now + d`, as `TestApp::advance` does, so each tick gets its own `delta` (FakeTimers `tickAsync`). Keep the lone-Esc flush working. Check every existing `testing::render` test that calls `advance` or `settle`.
- Sync driver: no change. `natural_end` requires a `None` wake, so an active animation keeps the app alive (Ink's timer is ref'd). Inactive or unmounted animations return `None`, and the app ends.

## 2. Tests

Create `tests/use_animation.rs`. Its header cites `test/use-animation.tsx` (Ink 8.0.0, commit 26d2c3f). Keep Ink's test names in snake_case and Ink's order.

Mapping:
- Ink `delay()` and FakeTimers become `advance(d)` on the fake clock. Ink's `>=` bounds become exact values.
- `debug: true` tests use `testing::render`. Non-debug tests use `testing::mount` with `Options { max_fps, .. }`.
- A `setTimeout` inside a component becomes a test-side `advance` plus a thread_local `State` write.
- Values captured during render and render counts go into thread_local vectors and counters.

Port 38 of 45:
- Cadence (10). `animations advance in debug mode when interactive is false`: mount with `interactive: Some(false)` and check the frame at unmount; ink-rs has no runtime debug option.
- Interval normalization: port `clamps zero interval to 1ms` and `clamps oversized finite interval to the timer maximum` (use `Duration::MAX`).
- Activity and reset (16). For `cleans up on unmount`, unmount the animated child through parent state and assert no frames after `advance`.
- Shared timer (8): port the frame assertions. The timer-count assertions are not applicable.
- Throttle (2): `low maxFps caps animation rerenders` (max_fps 1) and `delta accounts for throttled ticks` (max_fps 5).

Not applicable (7):
- the NaN, Infinity, negative Infinity and negative interval tests (`Duration` cannot express them);
- `wall clock changes do not move animations backwards` (the core reads no clock);
- the two concurrent and transition tests.

In `tests/pty.rs`, add the two `render.tsx` tests as pipe fixtures. Each uses `use_animation` with an 8 ms interval; a `use_effect` calls `AppHandle::exit` when `frame >= 3`; `main` prints `exited` after `wait_until_exit`. One fixture leaves `interactive` unset, the other sets `Some(false)`.

One runtime unit test in the module's `tests.rs`, because design.md promises that idle zero wakeups are tested: `animation_deadline` is a single minimum over the entries, and `None` when none is active. Add no other tests unless a rule has no other coverage; name the rule if you add one.

## Rules

- No `unsafe`. No new dependencies.
- No code that exists only to satisfy a test.
- Match the surrounding code: use `pub(crate)` or narrower for internal items, put imports at the top, and write comments that say why and cite Ink where the code ports it. Run `rustfmt --edition 2024` on every file you touch.
- Do not edit `docs/`. Report the documentation changes instead.
- Run only the following. Every test in these targets must pass.
  - `cargo test --test use_animation --test pty`
  - `cargo test --lib runtime`
  - `cargo test --test hooks --test hooks_use_input --test hooks_use_input_navigation --test hooks_use_paste --test focus --test runtime --test reconciler --test measure_element --test render --test exit --test terminal_resize --test components --test input_buffered_ctrl_c --test kitty_negotiation`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration. If the sandbox has no network, add `--offline`.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

Write the report in English.

1. Proposed commits, with message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. The design as built: the scheduler state, the tick and deadline algorithms, the step-time rule, and how `TestInstance` steps now.
3. Where the behaviour departs from Ink, with the reason.
4. The ported tests: names, every assertion you adapted with the reason, and the tests you marked not applicable.
5. Results of the commands above. Name any failing test, with your diagnosis.
6. The rows to change in `docs/ink-tests.md`, the entries to move out of its lists, the design.md wording changes, and open questions.
