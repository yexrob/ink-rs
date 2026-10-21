# ink-rs M0 phase 3, batch 2: our own host tree, and Yoga's flex basis and aspect ratio

You are working in `<repo>`. Read `CLAUDE.md` and `docs/design.md` first; the host and layout decisions there changed in `f863c3d`. Batch 1 is committed (`cdc2c95`, `6e131a3`). The Ink reference checkout is at `<scratchpad>/ink-src/ink`; read it, never run anything from it. It has no Yoga C++ source, so the Yoga rules you need are written out below.

taffy is `0.15.0`, source at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/taffy-0.15.0/src`. Read `tree/traits.rs`, `tree/taffy_tree.rs` (`TaffyView`'s trait impls, including `compute_child_layout` and `LayoutContainingBlock`), `compute/mod.rs` and `compute/flexbox.rs`.

## Why

Three layout tests fail because taffy follows CSS where Yoga does not:

- `tests/content_offset.rs` `disjoint_nested_clips_do_not_overwrite_content_when_a_wide_character_straddles_the_edge` (flex basis)
- `tests/width_height.rs` `set_aspect_ratio_with_height` and `set_aspect_ratio_with_width_and_height` (aspect ratio)

`TaffyTree` gives no hook to change how a container measures its children, so the host tree becomes our own structure implementing taffy's low-level traits. That is also the structure the M1 runtime will mutate across frames.

## 1. The host tree

Replace `HostTree = TaffyTree<HostNode>` with a host tree type of our own:

- Node storage owned by the host layer: parent, ordered children, the `HostNode` data (our `Style`, text, `is_static`), the mapped `taffy::Style`, a `taffy::Cache`, and the unrounded `taffy::Layout`, plus whatever `LayoutContainingBlock` needs (hoisted children). Nodes are addressed by `taffy::NodeId` because the traits require it. No new dependencies; a `Vec` arena is enough. `remove_child` keeps detaching without freeing; freeing comes with the M1 runtime.
- Keep the mutation API and its checks (`create_view`, `create_text`, `set_props`, `append_child`, `insert_child`, `remove_child`); methods on the tree are fine. `host::build` stays its client and `Element` stays above the host layer.
- Every mutation that can change layout clears the layout cache of the node and of each ancestor, as Yoga's `markDirty` does. With the per-node `taffy::Cache` this also serves as the text measurement cache that `docs/design.md` describes; add no other cache.
- Implement the taffy traits the layout entry point needs (`TraversePartialTree`, `TraverseTree`, `LayoutPartialTree`, `LayoutContainingBlock`, `CacheTree`, `LayoutFlexboxContainer`, and others only if required). Mirror `TaffyView`'s dispatch: hidden layout for `PerformHiddenLayout` and `Display::None`, `compute_flexbox_layout` for a View with children, `compute_leaf_layout` with the text measure for a leaf, all inside `compute_cached_layout`, and the out-of-flow pass as `TaffyView` runs it. Current absolute and Static positions must not change.
- Put the trait impls and the Yoga adjustments in `src/layout.rs` (or `src/layout/`), next to `map_style` and the text measure; the host module owns storage and mutations. Keep `layout::get` and its Yoga rounding unchanged. Update paint and render to the new tree; their output must not change.

## 2. Yoga's flex basis

Yoga (`computeFlexBasisForChild`, `calculateAvailableInnerDimension`) measures an in-flow child whose flex basis does not come from its style (no definite `flex_basis` with a definite container main size, no definite main-axis size) under these constraints: on each axis where the child has no definite style size, the child is measured **at most** the container's available inner size on that axis when that size is defined, and unbounded only when it is undefined. The same holds when Yoga lays a child out after flexing: a cross axis without a definite size is measured at most the container's available inner cross size. Ink never sets overflow on Yoga, so the overflow conditions in those functions are always true.

The container's available inner size on an axis is the size the container was given on that axis (its exact size when its style or its parent fixes it, otherwise the at-most size it was measured under) minus its padding and border, clamped by its min and max size minus padding and border. It is undefined when the container itself is measured unbounded on that axis. A child's own margin comes out of what it receives.

taffy measures the flex basis with `AvailableSpace::MaxContent` on the main axis (`determine_flex_base_size`, step E: "Map AvailableSpace::Definite to AvailableSpace::MaxContent"). Example from the failing test: a row of width 5 holding a View that holds `Text("你AAAA")`; Yoga measures the inner View at most 5 wide, the text wraps to two lines, and the box is 2 rows high; taffy measures it 6 wide and 1 row high.

Emulate Yoga in the host tree's `compute_child_layout`: when an in-flow child of a flex container is asked for a content-size measurement with an axis under `MaxContent`, and the container's available inner size on that axis is defined, measure it under `Definite(available inner size − the child's margin on that axis)` instead. Rewrite the inputs before the cache lookup so the cache keys on what was measured. The container's available inner size is known when the container's own `compute_child_layout` runs, before it enters `compute_flexbox_layout`; record it where its children's calls can read it (a per-node field or a stack; your choice, explain it). Absolutely positioned children are out of scope: leave them as taffy lays them out. taffy only asks for `MinContent` for automatic minimum sizes, which `map_style` disables; if you find another path that asks for it, treat it as Yoga's undefined, as the text measure does.

## 3. Yoga's aspect ratio

Yoga derives an item's cross size from its main size once the main size is resolved: cross = main ÷ ratio in a row, main × ratio in a column, measured exactly, ignoring the item's cross-size style and stretch (`distributeFreeSpaceSecondPass`, and the stretch pass of step 7). Its flex basis is the main-size style when definite; otherwise the cross size converted through the ratio when the cross size is definite (style, or stretched to an exact container cross size); otherwise its content. CSS (taffy) lets a definite cross size or stretch win over the ratio. Examples, in a column container: `height(3).aspect_ratio(2.0)` is 6 wide where taffy stretches it to the container; `width(8).height(3).aspect_ratio(2.0)` is 6 wide where taffy keeps 8.

Express this through `LayoutFlexboxContainer::get_flexbox_child_style`, which returns the style the flexbox algorithm uses for an item: for an item with an aspect ratio, a view of its style in which the size lives on the main axis and the cross size follows from the ratio (for example: the cross size becomes auto, a cross length moves to the main axis through the ratio when the main size is auto, and stretch no longer overrides the ratio). The item's own style as a container is not changed. Work out which Yoga cases this covers against the taffy code; a case it cannot express is a deviation to report, not a special case to add. These must keep passing: `set_aspect_ratio_with_width`, `set_aspect_ratio_with_max_height_constraint` and the other aspect ratio tests in `tests/`.

Baseline alignment is a recorded deviation (`docs/ink-tests.md`, `d374fa6`); `tests/flex_align_self.rs` `row_align_self_baseline` now expects taffy's CSS result and must keep passing.

## Rules

- No `unsafe`, no new dependencies (`CARGO_NET_OFFLINE=true`), no new tests, no expectation changes; do not edit `tests/` or `src/**/tests.rs`.
- No code that exists only to satisfy a test: implement the Yoga rule, and stop and report if an expectation needs a special case.
- Match the surrounding code: `pub(crate)` items, imports at the top, comments explain why and name the Yoga function a rule comes from. Run `rustfmt --edition 2024` on every file you touch. Update `docs/design.md` only if what you build differs from what it says, and say so.
- Run only:
  - `cargo test --lib`
  - `cargo test --test content_offset --test width_height --test flex --test flex_align_content --test flex_align_items --test flex_align_self --test flex_direction --test flex_justify_content --test flex_wrap --test gap --test margin --test padding --test position --test display --test overflow --test render_to_string --test text --test truncate_width --test absolute_truncation --test text_width --test borders --test components --test background --test static_blank_lines --test static_trailing_layout --test rendering_regressions`
  - `cargo clippy --all-targets -- -D warnings`

  Every test in these targets must pass, apart from the `#[ignore]`d ones. Never run `cargo test` without `--lib` or `--test`.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target` and delete it when done.
- Your sandbox cannot write `.git`, so leave the work uncommitted. In the report, list the commits you would make (message and files), one logical change each, each building and passing its tests on its own.

## Report

1. Proposed commits (message and files).
2. The host tree's API and data layout, and how the cache is invalidated.
3. The flex-basis rule as implemented: where the container's available inner size is computed and recorded, and which taffy calls the rewrite reaches.
4. The aspect-ratio view: the cases it covers and the Yoga cases it does not.
5. Results of the commands above, with any failing test and your diagnosis.
6. Open questions.
