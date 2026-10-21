# Task: let children and callbacks be component props

## Goal

Component props must be `PartialEq`: equal props skip the component's re-render. `Element` and closures are not `PartialEq`, so a component cannot take children or an event handler as a prop without a hand-written newtype that compares pointers (`examples/subprocess-output/app.rs:18`, `tests/suspend_terminal.rs:18`). A component library and a CLI framework will be built on ink-rs in another repository; their components take children (`Layout { title, children }`) and handlers (`Select { options, on_change }`). Fix both.

## 1. `PartialEq` for `Element`

- Implement `PartialEq` for `Element` and for the public builder types it contains: `View`, `Text`, `Transform`, `Static` (and the internal `Node` and `Component`). The comparison is structural: key, node kind, style fields, wrap, children in order.
- Component nodes are equal when the component type (`kind`) is the same and the props are equal through the existing `same_props`. Do not compare rendered output.
- Function fields compare by pointer (`Rc::ptr_eq`): `Transform`'s transform function and `Static`'s render function. `NodeRef` already compares by hook slot.
- After this, `#[derive(PartialEq)] struct LayoutProps { title: String, children: Element }` compiles, and a component whose children are equal is skipped.
- State the equality rule on `Element`'s rustdoc in one or two sentences: structural down to component boundaries, where props are compared; functions by pointer.

## 2. `Callback`

A new public type exported from the crate root, in a new module (it is not runtime-internal):

```rust
pub struct Callback<A = (), R = ()>(Rc<dyn Fn(A) -> R>);
```

- `Clone`; `PartialEq` by pointer; an opaque `Debug`.
- `pub fn call(&self, arg: A) -> R`.
- Construction from any `Fn(A) -> R + 'static`: `impl From<F>`, `Callback::new`, or both; pick what reads best and say why in the report.
- Two callbacks built from separate closures are unequal, even with identical code. A component that receives a fresh `Callback` on every render re-renders, as in React without `useCallback`.

## 3. `Scope::use_callback`

```rust
pub fn use_callback<A: 'static, R: 'static>(&mut self, f: impl Fn(A) -> R + 'static) -> Callback<A, R>
```

- Returns the same `Callback` (pointer-equal) on every render of the component, so a child that takes it as a prop is not re-rendered by it.
- Calling it runs the closure passed in the latest render; each render replaces the stored closure.
- No dependencies: this differs from React's `useCallback` and matches Dioxus's `use_callback`. Say so in the rustdoc.
- Do not hold a `RefCell` borrow while the closure runs: clone the inner `Rc` out first, so the closure can write state or reach the callback again.
- After unmount the handle still calls the last closure; state writes from it are ignored as for any unmounted owner. Document this in one sentence.
- A new `Kind` variant keyed by the `TypeId` of the signature, so the debug hook-order check covers it.

## Tests

Two tests in an existing integration test file that fits (`tests/hooks.rs` or `tests/reconciler.rs`; read them first and follow their style). Each one is needed because it covers new skipping behaviour:

1. A component whose props hold an `Element` is skipped when its parent re-renders with equal children, and re-renders when the children change. Count renders and check the frame.
2. A child that takes a `use_callback` handle as a prop is skipped when the owner re-renders; invoking the handle after the owner re-rendered with a new prop runs the latest closure (for example, the closure captures the prop and writes it to state shown in the frame).

Do not rewrite the existing pointer-equality newtypes in `examples/` and `tests/`: they wrap channels and `FnOnce + Send` values that `Callback` does not cover.

## Leave unchanged

`src/runtime/mod.rs` reconcile logic, `src/driver/`, `macros/`, `docs/`. Another worker is adding a context API at the same time (`Scope::provide` / `Scope::use_context`, new `Kind` variants in `src/runtime/hooks.rs`, a provider list on `Instance` in `src/runtime/mod.rs`). Keep your `hooks.rs` edits to your own additions.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo test --offline --lib`
- `cargo test --offline --test hooks --test reconciler --test runtime --test components --test rsx --test render_to_string`
- `cargo test --offline --no-default-features --test hooks --test reconciler`
- `cargo test --offline --doc`
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`

Do not run the full `cargo test`; I run it at integration.

## Project context

ink-rs is a Rust port of Ink (React for CLIs). Read `CLAUDE.md` / `AGENTS.md` and the 编程模型 and 写法 parts of `docs/design.md` before you start. Read `src/element.rs`, `src/components.rs`, `src/runtime/mod.rs` (`RenderComponent`, `Component`, `component`) and `src/runtime/hooks.rs`.

## Rules

- No unsafe code. No new dependencies. Code reads like the surrounding code: same naming, comment density and idiom. No code written only to satisfy a test.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report, in Chinese, no adverbs:
- the files you changed, and what changed in each;
- the public API you added, with signatures;
- design choices this brief left open, and what you chose;
- every deviation from this brief, and why;
- the exact commands you ran and their results.
