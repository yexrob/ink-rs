# ink-rs M0 phase 2: host tree, layout, paint, `render_to_string`

You are working in `<repo>`, a from-scratch Rust counterpart of Ink. Ink's tests are the spec. Read `CLAUDE.md` and `docs/design.md` first ("分层", "跨层约定", "布局与文本", "终端输出", "移植 Ink 测试的方法"). Phase 1 (committed) built the text layer in `src/text/`: `styled::{parse, serialize, expand_tabs, CharStyle, StyledChar, StyledLine, StyledText}`, `wrap::wrap_text`, `measure::measure_text`, `width`, and `Color::{at_level, sgr}` in `src/color.rs`. Build on it; change it where phase 2 needs a better API.

Two phase 1 decisions you inherit: the truncation ellipsis follows cli-truncate (it takes the kept side's style without the link at the start and end, and no style in middle mode or at one column), and wrap-ansi's NFC normalization is not reproduced. Report any test that depends on NFC rather than adding it.

The Ink reference checkout with `node_modules` is at
`<scratchpad>/ink-src/ink` (`INK` below). Read it, never run anything from it. Yoga there is `yoga-layout` 3.2.1; we use `taffy` 0.15 (already in `Cargo.toml`).

This phase makes `render_to_string` work. M0 has no runtime: elements are plain data, and every call builds a fresh host tree. Later milestones keep the host tree across frames and drive it with mutations, so design for that now.

## Architecture you must follow

- **Host tree = `TaffyTree<HostNode>`.** The host API is a set of mutations (create a View / Text node, set its props, append / insert / remove children). Converting an `Element` into a host tree is one client of that API; the M1 runtime is the next. Do not let `Element` leak below the host layer.
- **A `Text` is one taffy leaf.** Its nested `Text`, `Transform` and string children form an inline tree stored on the host node, not taffy nodes. The leaf's measure function squashes the inline tree into styled lines and measures them. No measure cache in M0.
- **Cells all the way down.** Squash, wrap, clip and paint work on `StyledChar`s. Strings appear at the boundaries only: `Text` content, the argument and result of a `Transform` function, and the `String` that `render_to_string` returns.
- **Paint is a pure function** from the laid-out host tree to a `Frame` (a grid of cells for the live region, plus the Static lines). The grid will be diffed against the previous frame in M2, so represent wide-character continuation cells explicitly. `render_to_string` serializes the frame.
- **No code that exists only to satisfy a test.** Implement the general rule. When an expectation looks like an artifact of Ink's JavaScript implementation, stop and report it; do not special-case it and do not change the expectation. Two are known already and stay failing in this phase: `tests/overflow.rs` `out_of_bounds_writes_do_not_crash` (JS sparse-array holes) and the triple OSC 8 open at `tests/c1_rendering.rs:54`.

Suggested modules: `src/host.rs` (or `src/host/`), `src/layout.rs`, `src/paint.rs` (or `src/paint/`), with `src/render.rs` wiring them. Names are yours if you have a better reason.

## Ink behaviour to port

Read the Ink files named here; this list is where they matter, not a substitute for reading them.

### Host tree (`INK/src/dom.ts`, `INK/src/reconciler.ts`, `INK/src/components/*.tsx`)

- `View` is Ink's `<Box>`; its defaults are already in `Style::default()`. `Spacer` is a growing `View`; `Newline` outside text is a `Text("\n")`.
- `Text` with no children renders nothing (Ink returns `null`); `Text::new("")` is a node with empty content.
- `Transform` outside a `Text` is a text leaf of its own whose function runs at paint time (see Paint). Inside a `Text` it runs during squash, per line, on the squashed lines of its children (`INK/src/squash-text-nodes.ts`).
- A `View` inside a `Text` (through `TextChild(Element)`) panics with Ink's message (`INK/src/reconciler.ts`).
- Background inheritance (`INK/src/components/BackgroundContext.ts`, `Box.tsx`, `Text.tsx`, `Static.tsx`): a `View`'s or `Static`'s background, or the inherited one, flows to descendant `Text`s that set none. Resolve it when the host tree is built.
- `Static` (`INK/src/components/Static.tsx`, `INK/src/renderer.ts`): an absolutely positioned column box holding `render(i)` for every item; `display: none` when there are no items. Its subtree is painted into its own output, positioned at its own computed left/top (not offset by its ancestors), and excluded from the live output.

### Layout (`INK/src/styles.ts`, `INK/src/dom.ts` `measureTextNode`, `INK/src/render-to-string.ts`)

Map `Style` to `taffy::Style`, smoothing over the Yoga differences in the mapping:

- The root node has Yoga's defaults (column, `flex_shrink: 0`), width = `columns`, height auto.
- Unset min width / height is 0, not CSS automatic minimum size (Ink sets `minWidth ?? 0`; Yoga has no min-content floor).
- Unset `align_content` is `FlexStart` (Yoga's default, not CSS `normal`).
- `Overflow` does not affect layout: Ink never passes it to Yoga, it only clips at paint time. Keep taffy's overflow `Visible`.
- `position: static` is relative with insets ignored.
- Shorthand resolution: `margin_top` over `margin_y` over `margin`, and likewise for padding, border sides, `overflow_x`/`overflow_y`. Border sides are 1 cell wide when `border_style` is set and the side is visible.
- Text measure (`measureTextNode`): natural size when the width is unconstrained (map taffy's min-content and max-content to Yoga's undefined mode) or when the text fits; otherwise wrap / truncate to the available width and measure that, and for the truncate modes reserve the full available width. Round text widths up.

### Paint (`INK/src/render-node-to-output.ts`, `INK/src/output.ts`, `INK/src/render-border.ts`, `INK/src/render-background.ts`, `INK/src/get-max-width.ts`)

- Walk order, `display: none`, content offsets (`content_offset_x/y`, truncated to integers), and clip push / pop with nested clips intersected, half-open bounds, horizontal clipping only when both x bounds exist, likewise vertical.
- Text leaf: squash; when the widest line exceeds the content width, wrap / truncate with the node's `TextWrap`; then per line, after clipping, apply the leaf's own line transform with the line index relative to the text's first line. For a `Text` the transform is its style; for a `Transform` it is the user function: serialize the line, call the function, parse the result. Applying the `Text`'s own style after wrapping and clipping matters, for example the middle truncation ellipsis gets the `Text`'s color.
- Text style layering: fill unset attributes from outer to inner (`CharStyle::fill_from`), with colors from props converted by `Color::at_level(color_level)`. `ColorLevel::None` means prop colors and attributes add no styling, as with chalk level 0; raw SGR inside user strings stays.
- The grid starts as root width × root height blank cells. Writes to rows outside it are dropped; writes past the right edge extend the row; cells at negative columns are dropped.
- Wide characters (`output.ts` `applyWriteOperation`, `sliceLineToColumns`): continuation cells, a wide character cut by a clip edge or a negative column becomes a blank cell keeping its style, and a write that lands on half of an existing wide character blanks the other half keeping its style.
- A grapheme made only of default-ignorable code points takes no cell; any other grapheme advances at least one column (`Math.max(1, width)` in `output.ts`).
- Background fill inside the border, and borders with per-side color, dim and background (`render-border.ts`, `render-background.ts`), with prop colors converted by the color level.
- Serialize each row with `styled::serialize` after dropping trailing blank cells that have the default style (Ink's `trimEnd()` on the styled string keeps styled trailing spaces).

### `render_to_string` (`INK/src/render-to-string.ts`, `INK/src/renderer.ts`)

Static output first: its rows followed by `\n`, sized as in `renderer.ts` (left + width + right margin, top + height + bottom margin); none when the Static node or an ancestor is `display: none`. Then the live output rows joined with `\n`. When the live output has zero rows, the result is the Static output without its trailing newline. Behaviour in the doc comment of `render_to_string` stays as written.

## Rules

- No `unsafe`. No new dependencies; `CARGO_NET_OFFLINE=true`.
- No new tests. Temporary checks are fine while you work; delete them before you finish.
- Do not change any expected value, in `tests/` or in `src/**/tests.rs`. Do not edit `tests/common`.
- Remove `#![allow(dead_code)]` from `src/lib.rs` and resolve what it hid: delete code nothing needs, keep what phase 2 uses.
- Match the surrounding code: crate-internal items are `pub(crate)`, imports at the top of the file, comments explain why (cite the Ink file when a rule comes from Ink). Run `rustfmt --edition 2024` on every file you touch.
- Run only these:
  - `cargo test --lib`
  - `cargo test --test render_to_string --test text --test flex_direction --test padding --test borders --test background --test colorize`
  - `cargo clippy --all-targets -- -D warnings`

  Never run `cargo test` without `--lib` or `--test`, and no other integration test files: the integrator runs the full suite and sends failures back.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target` and delete it when you are done.
- Commit in small steps as `CLAUDE.md` rule 3 describes: one logical change per commit, Conventional Commits, stage your files by path, never push. End each commit message with the line `Co-Authored-By: Codex <noreply@openai.com>`. If git cannot write in your sandbox, leave the work uncommitted and say so in the report.

## Report

1. Commits (or files changed) and the internal API: host mutations, layout entry point, `Frame`, paint entry point.
2. Design notes the M1 runtime and the M2 output diff need to know.
3. Every deviation from Ink and why.
4. Results of the commands above, with the names of failing tests and your diagnosis of each.
5. Open questions, and expectations you suspect are Ink artifacts.
