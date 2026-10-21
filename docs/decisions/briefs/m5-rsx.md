# ink-rs M5: the `rsx!` macro

You are working in a git worktree of ink-rs at `<scratchpad>/wt/m5-rsx`. Read `CLAUDE.md`, then:

- `docs/design.md`: the paragraph that starts with "`Box` 与 `std::boxed::Box` 重名" (the macro is sugar over the builder and can do nothing the builder cannot) and the crate-layout paragraph ("加过程宏时拆成 workspace");
- `src/components.rs` (`View`, `Text`, `Transform`, `Static`, `Newline`, `Spacer`: constructors, `child`, `key`, `Default` impls), `src/style.rs` (`Styled`: the style methods are trait methods), `src/element.rs` (`Element`, `TextChild`, the `From` impls);
- `src/lib.rs`, `Cargo.toml`, `Cargo.lock` (syn 2.0.119, quote 1.0.47, proc-macro2 1.0.107 are already locked);
- `examples/jest/app.rs` and `examples/chat/app.rs` as samples of builder code.

## 1. What it is

`ink_rs::rsx!` builds the same values as the builder, with less nesting. It calls only public builder API (`new`, `default`, builder methods, `child`, `key`, `Element::fragment`, `Element::from`), so anything it writes can be written by hand. Example:

```rust
rsx! {
    View(flex_direction: FlexDirection::Column, margin_top: 1) {
        View {
            View(width: 14) { Text(bold: true) { "Test Suites:" } }
            if failed > 0 {
                Text(bold: true, color: Color::Red) { "{failed} failed, " }
            }
            Text { "{total} total" }
        }
    }
}
```

String literals capture identifiers only, as `format!` does; other expressions go in `{}`: `Text { {format!("{} total", passed + failed)} }`.

## 2. Grammar

```
Input   := Node+
Node    := Element
         | LitStr
         | "{" Expr "}"
         | "if" Expr "{" Node* "}" ( "else" "if" ... | "else" "{" Node* "}" )?
         | "for" Pat "in" Expr "{" Node* "}"
Element := Path ( "(" Args? ")" )? ( "{" Node* "}" )?
Args    := Arg ( "," Arg )* ","?
Arg     := Ident ":" Expr        // named
         | Expr                  // positional
```

- Parse `if` conditions and `for` iterables with `Expr::parse_without_eager_brace`.
- Positional args come before named args; a positional arg after a named one is a compile error.
- A bare identifier is a positional expression, never a flag: `Text(content)` is `Text::new(content)`. Flags are written `bold: true`.

## 3. Expansion

Each element expands to a block that threads one builder value through statements, so every child goes straight to the parent's own `child` method and `if` / `for` branches never need a common type:

```rust
{
    let b = Path::new(p1, p2);          // positional args present
    // let b = <Path>::default();       // no positional args
    let b = b.flex_direction(x);        // named args, in order (except `key`)
    let b = b.child(child_1);
    let b = if cond { let b = b.child(a); b.child(c) } else { b.child(d) };
    let b = if cond { b.child(a) } else { b };            // `if` without `else`
    let b = { let mut b = b; for pat in iter { b = b.child(x); } b };
    b.key(k)                            // `key: k` is applied last; otherwise just `b`
}
```

- The builder variable uses a `Span::mixed_site()` identifier so user code cannot see or shadow it.
- A child node is: an element's expansion; `{expr}` → `expr` unchanged; a string literal → the literal itself, or `::std::format!(lit)` when it contains `{` (Rust 2021 inline captures; `{{` escapes as in `format!`). A string under `View` fails to compile because `View::child` takes `Into<Element>` — that is Ink's "text must be inside Text" rule; do not add a special error.
- The whole expansion starts with `#[allow(unused_imports)] use ::ink_rs::Styled as _;` once, in the outermost block, so `Styled` methods resolve without the caller importing the trait.
- Input with one root node of kind Element or `{expr}`: the result is that value (the builder type, or `Element` when `key` is set), so it can be chained or passed to `impl Into<Element>`. Two or more roots: `::ink_rs::Element::fragment([::ink_rs::Element::from(r1), ...])`. A root that is a string literal, `if` or `for`: compile error with a message saying to wrap it in an element or a `{}` expression.
- Keep user spans: method names from named args, paths and expressions keep their original spans, so a misspelled `colr: Color::Red` reports "no method named `colr`" at `colr`. Parse errors use `syn::Error` at the offending token.

No component sugar: components are `{component(f, props)}` children or roots.

## 4. Crate layout

- New crate `macros/` (package `ink-rs-macros`, version 0.1.0, edition 2024, `proc-macro = true`) depending on `syn` 2 (features `full`), `quote` 1, `proc-macro2` 1. Use the versions already in `Cargo.lock`; do not run `cargo update`. One `#[proc_macro] pub fn rsx`. Its doc comment holds the grammar and expansion rules above, in the style of the crate's other doc comments; doc examples in this crate are `ignore` blocks (it cannot depend on `ink-rs`).
- Root `Cargo.toml`: `[workspace] members = ["macros"]`; dependency `ink-rs-macros = { path = "macros", version = "0.1.0", optional = true }`; feature `macros = ["dep:ink-rs-macros"]`, and `default = ["tokio", "macros"]`.
- `src/lib.rs`: `#[cfg(feature = "macros")] pub use ink_rs_macros::rsx;` and one sentence plus a small runnable doc example in the crate docs (an `rsx!` tree passed to `render_to_string`, with its expected output asserted).

## 5. Tests

New file `tests/rsx.rs`, `#![cfg(feature = "macros")]`. Each test builds one tree twice, with `rsx!` and with the builder, and asserts that `render_to_string` gives the same string (and checks one expected string literally, so both being empty cannot pass). Five tests:

1. nesting, named args, `Styled` methods without importing `Styled`, string literals with and without `{capture}`;
2. positional args: `Text("a")`, `Transform(|line, _| ..) { "x" }`, `Static(items, |item, _| ..)`; `Text { "a" "b" }` (default + two children);
3. `if` / `else if` / `else` and `if` without `else`, under `View` and under `Text`; `for` under `View` with keyed children;
4. `{expr}` children: an `Option<Text>`, a `Vec<Element>`, a `component(..)` call; `Spacer` and `Newline` without parens;
5. `key` on a root, and a two-root fragment.

No compile-fail tests and no `trybuild`.

## 6. Rules

- No unsafe code. No new dependencies beyond `syn`, `quote`, `proc-macro2` in the macro crate.
- `cargo fmt --all`. Comments sparse, in the style of the surrounding code.
- Do not edit `docs/`. I update the docs at integration.
- Run only these commands, all with `--offline`:
  - `cargo build --offline` and `cargo build --offline --no-default-features`
  - `cargo test --offline --test rsx` and `cargo test --offline --doc`
  - `cargo clippy --offline --workspace --all-targets -- -D warnings` and `cargo clippy --offline --no-default-features --all-targets -- -D warnings`
  - `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`
  Do not run the full `cargo test`. I run it at integration.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## 7. Report

Your final message is the report:
- what you added and changed, by file;
- every deviation from this brief, and why;
- the exact commands you ran and their results.
