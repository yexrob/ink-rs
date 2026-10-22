# Task: two hooks for eink-rs components: the colour level and a cursor anchored to a node

## Goal

eink-rs is building two components on ink-rs that each need a primitive ink-rs lacks:

- `Code` draws diff lines on a tinted background. Tints degraded to 16 colours look wrong, so the component wants to pick background tints at TrueColor and 256 colours and fall back to foreground colours below. Components cannot read the colour level today; ink-rs applies it only when painting.
- `Composer` is a multi-line prompt that wraps its own lines and places the terminal cursor for IME. `use_cursor` takes live-frame coordinates, so the component computes them from `measure_element`, which reads the previous commit's layout. When an ancestor moves (a line above grows), the cursor lags one frame and the IME candidate window jumps.

Add both hooks.

## Read first

- `<repo>/CLAUDE.md`.
- `<repo>/docs/decisions/074-color-level-node-cursor.md`.
- `<repo>/src/runtime/hooks.rs`: `use_cursor`, `use_node_ref`, `use_window_size`.
- `<repo>/src/runtime/output.rs`: `CursorPosition`.
- `<repo>/src/runtime/state.rs`: `Store` (its `cursor` cell), `NodeRef`, `NodeLayout`, `ElementMetrics`, `measure_element`.
- `<repo>/src/runtime/mod.rs`: `Runtime::cursor` and where the commit writes node layouts; where `RenderOptions::color_level` reaches the runtime.
- `<repo>/src/app.rs`: `Options::color_level`, where the app reads the cursor and calls `paint`.
- `<repo>/src/render.rs`, `<repo>/src/testing.rs`: how `render`, `render_with`, `render_to_string` and `mount` pass a colour level.
- `<repo>/src/driver/environment.rs`: how the driver detects the level.

## Change 1: `Scope::use_color_level()`

```rust
/// The colour level this app paints at: [`Options::color_level`], or the level
/// the driver detected. Colours a component chooses are degraded to it when painted.
pub fn use_color_level(&mut self) -> ColorLevel;
```

- It returns the level the painter uses for this runtime, on every path: driver mount, `testing::mount`, `render`, `render_with`, `render_to_string`.
- The level is fixed for an app's life, so the hook schedules nothing. If you find a path where it changes, say so and stop there.

## Change 2: a cursor anchored to a node

```rust
/// Declares the terminal cursor at `position` cells from the top-left of the
/// View `node` is attached to. ink-rs resolves the node's position from the
/// layout of the frame being painted, so the cursor follows the node in the
/// same frame when an ancestor moves. `None`, or a node with no layout, hides it.
pub fn use_node_cursor(&mut self, node: NodeRef, position: Option<CursorPosition>);
```

- The name is open; say why in the report if you change it.
- It shares the single cursor slot with `use_cursor`, with the same commit, skip and unmount rules. The last declaration committed wins, as today.
- The store keeps the declaration unresolved (the node plus the offset). The app resolves it after layout and before `paint`, from the layout committed in the same step. Check that `ElementMetrics::x` and `y` share the origin of `CursorPosition` (the live frame's top-left, below Static); if they differ, convert and say how.
- The offset is from the node's border-box top-left, the origin of `ElementMetrics`. Padding and borders are the caller's to add.
- Clipping to the visible frame stays where it is today.
- Debug and string rendering ignore it, as they ignore `use_cursor`.
- `use_cursor` keeps its signature and behaviour; `CursorPosition` keeps its fields.

## Tests

- **Anchored cursor**: a mounted app, through `testing::mount` and `TestTerminal`, puts a `View` with a node ref below a line whose height changes on input, and declares `use_node_cursor` at an offset inside the `View`. After the input, the cursor move in `writes()` lands on the `View`'s new row in the same paint. This is the behaviour the hook exists for; a frame-coordinate cursor computed from `measure_element` in render fails it.
- **Hidden**: the anchored node unmounts while the declaring component stays; the cursor is hidden. It covers the no-layout rule.
- **Colour level**: a component reads `use_color_level` under `render_with` with a set level, and under `testing::mount` with `Options::color_level`, and sees that level. One test covering both paths, because each path passes the level its own way.

Add nothing beyond these.

## Out of scope

`docs/`, README, CHANGELOG, examples, the version number.

## Rules

- `CLAUDE.md` holds. No `unsafe`.
- Behaviour outside these hooks does not change; the existing tests are the check.
- Code reads like the existing code: naming, comment density, idiom.
- New public items get doc comments in the style of `use_cursor` and `use_window_size`.
- No new dependencies.
- **The sandbox cannot write `.git`.** Do not commit. Do not write report files into the repository.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`
- `cargo test --offline --lib`
- `cargo test --offline --test hooks --test runtime`, the test files that cover `use_cursor` (`grep -l use_cursor tests/*.rs`), and any test file you add.
- Run only these; the main agent runs the full suite.

## Report

Your final message is the report, in Chinese, without adverbs. It covers:

- the files you changed, and what each change does;
- the API and its names;
- where the level comes from on each path;
- how the anchored cursor is stored and where it is resolved, and the origin check;
- every deviation from this brief, and why;
- the tests you added;
- the exact commands you ran and their results.
