# Task: components in `rsx!` through a `Props` derive

## Goal

`rsx!` builds elements through builders: `Name(pos, method: value) { children }` expands to `Name::new(pos)` or `<Name>::default()`, then one method call per named argument, `child` per child, `key` last, and the result goes through `Into<Element>`. Components are written as `{component(f, props)}` expressions. Add a derive on a props struct so that a component reads like `View` and `Text`:

```rust
#[derive(Default, PartialEq, Props)]
#[props(component = layout)]
struct Layout {
    /// Shown in the border.
    #[props(into)]
    title: String,
    width: u16,
    on_back: Option<Callback>,
    children: Vec<Element>,
}

fn layout(cx: &mut Scope, p: &Layout) -> Element { /* ... */ }

rsx! {
    Layout(title: "Deploy", width: 40, key: "main") {
        Text { "step 1" }
    }
}
```

The derive is sugar over the builder API and adds no capability: everything it generates can be written by hand. Do not change `rsx!`'s grammar.

## 1. `#[derive(Props)]`

`#[proc_macro_derive(Props, attributes(props))]` in `ink-rs-macros`, re-exported from the crate root next to `rsx` under the `macros` feature. Put the derive in its own module of the macros crate; keep `lib.rs` for the entry points and `rsx!`.

- Applies to structs with named fields; anything else is an error at the item.
- `#[props(component = path)]` on the struct is required. The path is a component function item for `ink_rs::component`, such as `layout` or `select::<T>`.
- `#[props(into)]` on a field is the only field option.
- Unknown keys, a missing `component`, and duplicate options are errors with spans on the offending tokens.

Generated items, all with the struct's visibility and generics (`split_for_impl`, the struct's where clause kept):

- For each field except `children`, a setter named after the field: `fn title(mut self, title: T) -> Self`, taking the field type as is. With `#[props(into)]` it takes `impl Into<T>`. Exact types are the default because `impl Into<u16>` rejects an integer literal such as `40` (it falls back to `i32`). Copy the field's doc comments onto the setter.
- For a field named `children`, which must be `Vec<Element>`: `child(impl Into<Element>)` pushes, and `children<I>(I)` extends, with the same signatures as `View::child` and `View::children`. `#[props(into)]` on `children` is an error.
- `fn key(self, key: impl Into<Key>) -> Element`, as on the built-in builders.
- `impl From<Struct> for Element` calling `::ink_rs::component(path, props)`. Add `Struct<..>: PartialEq + 'static` (the props type, not `Self`, which is `Element` there) to its where clause, so a missing bound reports on the impl instead of inside `component`.
- A field named `key` or `child` is an error: the generated methods use those names.

Refer to ink-rs items by absolute paths (`::ink_rs::Element`, `::ink_rs::Key`, `::ink_rs::component`, `::core::convert::Into`) and keep user spans on field names and types, so a wrong argument type in `rsx!` reports at the user's tokens.

Not part of this task: compile-time checks for required props (a type-state builder), default-value attributes, `Default` for `Callback`, `From` closures into `Callback`. A component with required props writes its own `new` and takes them as positional arguments.

## 2. Generic props in `rsx!`

`<Select>::default()` does not infer a generic struct's parameters (E0107, missing generics). When no segment of the element path has generic arguments, construct with `Path::default()` / `Path::new(..)`, so `Select(items: vec![1u8])` infers `Select<u8>`. When any segment has arguments (`Select<u8>` or `Select::<u8>`), keep `<Path>::default()` / `<Path>::new(..)`. Built-in elements behave as before.

## Docs

- Rustdoc on the derive: what it generates, the two options, the `children` rule, and one example. Wording style: plain declarative sentences like the existing `rsx!` doc.
- Update `rsx!`'s rustdoc: the sentence on components (`{component(f, props)}`) now points to the derive, and the constructor rule states the generic-argument case.
- Leave `docs/` unchanged; I update `docs/design.md` at integration.

## Tests

Two tests in `tests/rsx.rs`, following its style (`assert_rendered`). Each covers new public behaviour:

1. A derived component with an `into` field, an exact-type numeric field, two children and a `key`, written in `rsx!` inside a `View`. Compare against the same tree built with `component(layout, Layout { .. })` and `.key(..)`: by `render_to_string`, and the component element itself by `PartialEq` (`Element` implements it, so this checks the key and props too).
2. A generic props struct used in `rsx!` without type arguments renders the inferred items.

No trybuild or compile-fail tests.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo test --offline --test rsx`
- `cargo test --offline --doc --workspace`
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings` (this also builds the examples, which use `rsx!`)
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`

Do not run the full `cargo test`; I run it at integration.

## Project context

ink-rs is a Rust port of Ink (React for CLIs). Read `CLAUDE.md` / `AGENTS.md` and the 编程模型 and 写法 parts of `docs/design.md` before you start. Read `macros/src/lib.rs`, `tests/rsx.rs`, `src/components.rs` (`View`'s builder methods), `src/element.rs` and `component` in `src/runtime/mod.rs`.

## Rules

- No unsafe code. No new dependencies: `syn` (full), `quote` and `proc-macro2` are there. Code reads like the surrounding code: same naming, comment density and idiom. No code written only to satisfy a test.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report, in Chinese, no adverbs:
- the files you changed, and what changed in each;
- the generated code for the example `Layout` above, written out;
- design choices this brief left open, and what you chose;
- every deviation from this brief, and why;
- the exact commands you ran and their results.
