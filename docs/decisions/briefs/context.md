# Task: add a context API to ink-rs

## Goal

Components can share a value with their descendants without passing it through every level of props. A component library (themes), a CLI framework and stores will be built on it in another repository.

## API

Two hooks on `Scope` (`src/runtime/hooks.rs`):

```rust
pub fn provide<T: Clone + PartialEq + 'static>(&mut self, value: T)
pub fn use_context<T: Clone + 'static>(&mut self) -> Option<T>
```

Both occupy a hook slot, follow the hook-order rule, and get their own `Kind` variants carrying the `TypeId`, so the debug hook-order check covers them.

## Semantics

1. Contexts are keyed by type. Two contexts of the same type need newtypes.
2. `provide` makes the value visible to the subtree the component renders: the elements it returns and all their descendants, through `View`, `Text`, `Static` items, fragments and nested components. The providing component's own `use_context` reads its ancestors, not its own `provide`, as in React, where the Provider sits in the returned tree.
3. The nearest ancestor provider wins. If one component calls `provide::<T>` twice, the later call wins for its descendants.
4. `use_context` without a provider returns `None`. It returns a clone of the value on each call.
5. `provide` compares the value with the previous render's value. When they differ, it stores the new value and schedules every mounted reader of that provider. When they are equal, nothing happens.
6. Readers re-render even when every component between the provider and the reader is skipped by props equality. Use the existing `Dirty` marks (`own` / `descendants`, bubbled like React lanes): the provider marks its readers during its own render; readers are its descendants, so the same commit renders them with the new value. One commit, one frame, no intermediate frame. Check that the reconcile walk reaches a marked reader below a component whose props are equal (`reconcile_children` with `next = None` walks the children with pending marks).
7. A mounted instance's ancestry never changes (a type or key change remounts), and hook order is fixed after the first render, so the providers visible to an instance are fixed for its lifetime. A reader resolves its provider on its first render, keeps it, and registers itself with that provider through a weak reference to its `Dirty`. Unmounted readers are skipped and pruned (`Dirty` has a `mounted` flag).
8. Marking readers is not a state write. It must not trigger the debug panic for state writes during render (`src/runtime/state.rs:151`).

## Implementation guidance

- Each `Instance` holds the providers visible to it, for example an `Rc` linked list of `(TypeId, Rc<provider cell>)`, given by its parent at creation. After a component renders, its children receive the parent's list extended with the cells held by that component's `provide` hooks. Other nodes pass their list through unchanged.
- Put the context types in a new `src/runtime/context.rs`.
- Avoid `Rc` cycles: a provider cell holds its readers weakly; a reader's hook may hold the cell strongly.
- Leave the built-in background-colour inheritance as it is. It updates Text host nodes without re-running user components and has different rules; do not fold it into the new mechanism.

## Tests

Three tests in an existing integration test file that fits (`tests/hooks.rs`, `tests/reconciler.rs` or `tests/runtime.rs`; read them first and follow their style). Each one is needed because it covers new scheduling behaviour:

1. A reader below a component whose props are equal re-renders when the provider's value changes, and the intermediate component does not. The frame after the change shows the new value, with no intermediate frame. Providing an equal value re-renders no reader.
2. With nested providers of the same type, the nearest wins; a component that provides and reads `T` reads its ancestor's value.
3. `use_context` without a provider returns `None`.

## Docs

Rustdoc on both methods in the style of the neighbouring hooks. Do not edit `docs/`; I update `docs/design.md`.

## Leave unchanged

`src/element.rs`, `src/components.rs`, `macros/`, `src/driver/`, `docs/`. Another worker is adding `PartialEq` for `Element` and a `Callback` type with `Scope::use_callback` at the same time; it also adds a `Kind` variant and a method in `src/runtime/hooks.rs`. Keep your `hooks.rs` edits to your own additions.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo test --offline --lib`
- `cargo test --offline --test hooks --test reconciler --test runtime --test focus --test use_box_metrics --test background`
- `cargo test --offline --no-default-features --test hooks --test reconciler --test runtime`
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`

Do not run the full `cargo test`; I run it at integration.

## Project context

ink-rs is a Rust port of Ink (React for CLIs). Read `CLAUDE.md` / `AGENTS.md` and the 编程模型 and 事件循环 parts of `docs/design.md` before you start. Read `src/runtime/mod.rs` (`Dirty`, `Instance`, `reconcile`, `reconcile_children`) and `src/runtime/hooks.rs`.

## Rules

- No unsafe code. No new dependencies. Code reads like the surrounding code: same naming, comment density and idiom. No code written only to satisfy a test.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report, in Chinese, no adverbs:
- the files you changed, and what changed in each;
- how the provider list reaches each instance, and how readers are registered and notified;
- design choices this brief left open, and what you chose;
- every deviation from this brief, and why;
- the exact commands you ran and their results.
