# Task: cut the per-frame cost of reconcile, layout and paint in ink-rs

## Goal

A streamed Markdown reply in eink-rs (a downstream crate) spends most of each frame in ink-rs. Two costs dominate, and neither changes what ink-rs renders:

1. Each text host node is squashed again on every measurement and on every paint.
2. Reconcile deep-clones element subtrees at every level of the tree.

Remove both. Output, effects, hook order, Static behavior and the public API stay as they are.

## Read first

- `<repo>/CLAUDE.md`.
- `<repo>/docs/decisions/068-render-perf.md`: this plan and the user's choices.
- `<repo>/docs/design.md`: 「布局与文本」 (measurement cache, Transform) and 「编程模型」 (how `Element` compares).
- `<repo>/src/host.rs`: `TextNode::squash`, `squash`, `set_props`, `create`, `mark_dirty`.
- `<repo>/src/layout.rs`: the measure closure in `compute_child_layout`, and `constrain_measurement`.
- `<repo>/src/paint.rs`: the text branch of `walk`.
- `<repo>/src/runtime/mod.rs`: `Instance::new`, `matches`, `abandon`, `reconcile_children`, `reconcile`, `inline_children`, and every other reader of `Instance::element`.
- `<repo>/src/element.rs`: `Element`, `Node`.

## What the profile shows

macOS `sample`, release build, eink-rs `markdown-stress` driving a streamed Markdown reply through `ink_rs::testing::mount` at 120 fps. Shares are inclusive, of the time in `TestApp::advance`.

| Scenario | p50 per frame | `TextNode::squash` | `Node::clone` | `drop_in_place<Node>` |
|---|---|---|---|---|
| list item holding a 300-line code block (the tail re-renders it each frame) | 37.8 ms | 47.7% | 9.3% | 13.2% |
| 4 KB of a mixed reply per frame (about 100 new Static pieces per frame) | 13.0 ms | 29.4% | 6.6% | 4.3% |

Counting calls showed `squash` running about 13 times per text node per frame: the taffy cache misses through the AtMost rewrite in `constrain_measurement`, so each measurement squashes again, and paint squashes once more. A lazy squash cache cut the first scenario to 20.5 ms and the second to 10.9 ms. With that cache in place, `Node::clone` rose to 15% and `drop_in_place<Node>` to 22.8% of the first scenario. Both come from `reconcile`: `instance.element.0.clone()`, the `children.clone()` arms, and `Instance::new(element.clone(), …)`. Each clone copies the whole subtree, so a tree of depth d is copied about d times per render.

## Change 1: squash once per text host node

- When `set_props` (and so `create`) sets text props, squash the inline tree and expand tabs once, and keep the lines and their natural size (`measure_text`) on the host node. Measurement and paint read them in place of calling `squash`.
- Keep the cache out of the props' equality: reconcile compares `TextNode`s to decide whether to call `set_props`, and that comparison must keep its meaning.
- Wrapping at a width stays where it is.
- Transform functions nested inside a `Text` now run when the text's props change, not on every measurement and paint. The root `Text`'s own transform keeps running at paint as it does now. Ink calls nested transforms on every measurement and render; the output is the same for pure functions. The main agent records this in `docs/design.md`.

## Change 2: reconcile without deep clones

- Move child elements out of the incoming element instead of cloning them, and construct new instances without copying the element that `reconcile` is about to receive.
- Before moving anything, find every reader of `Instance::element` and of an element's children after reconcile: `matches`, `same_props`, `inline_children`, the Static arm (`render`, `len`, style), the View arm (style, `node_ref`), `abandon` with `overview`, and any others you find. Each must read what it reads today. If a reader needs children kept, keep them for that case alone, and say so in the report.
- Component props are not element children: never move out of them.
- `Element` keeps its public `Clone` and `PartialEq`. Do not change `Element`'s public representation; `Rc` children are out of scope (see record 068).

## Out of scope

- The AtMost rewrite in `constrain_measurement`. It matches Yoga; with the squash cache in place, removing it gained 20.5 → 18.6 ms in the first scenario.
- Painting only visible rows: Ink rewrites the whole frame when output is taller than the terminal.
- Any other optimisation. If you find another large cost, measure it and describe it in the report; do not fix it.

## Rules

- `CLAUDE.md` holds. No `unsafe`.
- No public API change and no behavior change. Existing tests are the check. Add a test only where a behavior these changes touch has no test, and say in the report why it is needed.
- Code reads like the existing code: naming, comment density, idiom. Comments that cite Ink or Yoga stay accurate after the change.
- Do not touch README, `docs/`, examples or other crates; the main agent writes the docs.
- **The sandbox cannot write `.git`.** Do not commit. Do not write report files into the repository.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`
- `cargo test --offline --lib`
- `cargo test --offline --test squash_text_nodes --test text --test reconciler --test components --test component_regressions --test render --test render_to_string --test errors --test static_blank_lines --test static_runtime_blank_lines --test static_string_replacement --test static_trailing_layout --test issue_973_static_commit`
- Run only these tests; the main agent runs the full suite and the timing.

## Report

Your final message is the report, in Chinese, without adverbs. It covers:

- the files you changed, and what each change does;
- where the squash cache lives and when it is filled;
- every reader of `Instance::element` you found, and how each reads the same thing after the change;
- every deviation from this brief, and why;
- tests you added, and why each is needed;
- other large costs you saw, with numbers and how you measured them;
- the exact commands you ran and their results.
