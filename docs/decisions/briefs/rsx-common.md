## The macro

`ink_rs::rsx!` (in `macros/src/lib.rs`; read its doc comment, and `tests/rsx.rs` for usage) builds the same values as the builder:

- `Name(positional, ..., method: value, ...) { children }`. With positional args it calls `Name::new(..)`, otherwise `<Name>::default()`; named args are builder method calls in order; `key: k` is applied last and turns the value into an `Element`. There are no bare flags: write `bold: true`.
- Children: elements, string literals, `{expr}`, `if cond { .. } else if .. { .. } else { .. }`, `for pat in iter { .. }`. Children go to the parent's `child`. `if`/`for` branches need not share a type.
- A string literal containing `{` becomes `format!(lit)`: it captures identifiers only (`"{count} tests"`, `"{x:>3}"`). Field accesses and calls go in `{format!(..)}` or in a local bound before the macro. `{{` escapes a brace.
- A bare identifier child is parsed as an element (`spinner` becomes `<spinner>::default()`): put variables in braces, `{spinner}`.
- No `let` inside the macro. Compute values before it.
- Components stay `{component(f, props)}`; `Static(items, |item, i| ...)` and `Transform(|line, i| ...)` take positional args; `Newline` and `Spacer` need no parens.
- One root returns the builder value; a function returning `Element` ends with `rsx! { .. }.into()` (the macro is followed by `.into()` on the closing brace line). Several roots become a fragment.
- The expansion imports `Styled` itself: remove `Styled` imports that become unused.

## How to rewrite

- Rewrite every element tree built with `View`, `Text`, `Static`, `Transform`, `Newline`, `Spacer` (and `Element::fragment`) into `rsx!`. Prefer the JSX-like shape Ink's example has: `.children(iter.map(..))` becomes `for`, conditional children become `if`, `Text::new(format!("{}", a.b))` becomes `Text { "{x}" }` with a local or `Text { {format!(..)} }`, whichever reads better.
- Behavior must stay identical: same tree, same styles, same keys, same output. Do not change state, hooks, input handling, reducers or other logic. Keep existing comments; reword only those that describe builder code.
- rustfmt does not format inside `rsx! { }`. Format the macro bodies by hand: 4-space indentation, one child per line; a leaf with a single short child may stay on one line (`Text(bold: true) { "Total" }`); when an element's argument list would pass 100 columns, put one argument per line with a trailing comma, the way rustfmt formats a call. `cargo fmt --all --check` must pass for the code outside the macros.

## Rules

- No unsafe code. No new dependencies. No new tests; the existing tests that cover the examples must pass unchanged.
- Do not edit `docs/`, `src/` or `macros/`. I update the docs at integration.
- Run only these commands, all with `--offline`:
  - `cargo build --offline --examples`
  - `cargo test --offline --test examples --test examples_pty`
  - `cargo clippy --offline --workspace --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  Do not run the full `cargo test`. I run it at integration. Do not run the examples interactively.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report:
- the files you changed;
- anything in an example that `rsx!` could not express cleanly, and what you did;
- every deviation from this brief, and why;
- the exact commands you ran and their results.
