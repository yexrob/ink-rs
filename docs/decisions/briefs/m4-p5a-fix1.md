# ink-rs M4 phase 5, batch 1: review fixes

You are working in the same worktree as before: `<scratchpad>/wt/m4-p5a`. Your earlier work is there, uncommitted. The review found no logic problems. Fix the layout issues below and change nothing else.

## 1. Blank lines and import groups

Match `examples/counter.rs`:
- `use std::…` lines first, a blank line, then `use ink_rs::…`. In `tests/examples.rs`, the `#[path] mod` block, then a blank line, then the imports.
- A blank line between top-level items: `const`, `struct`, `enum`, `impl`, `fn`, `mod`. A doc or line comment and attributes stay attached to the item below them.
- A blank line after the file's header comment, unless an item's comment is the header, as in `counter.rs`.
- In `tests/examples.rs`, a blank line between the helpers and between tests. The `// render.tsx:N` comment sits right above its `#[test]`.

This applies to every new file under `examples/` and to `tests/examples.rs`.

## 2. Chains rustfmt skipped

rustfmt gives up on a whole expression when a string literal inside it pushes a line past 100 columns, and `--check` still passes. These chains are unformatted:
- `examples/use-focus.rs`, line 14;
- `examples/use-focus-with-id.rs`, line 23;
- `examples/aria/app.rs`, lines 14–17;
- `examples/scroll/app.rs`, lines 43–50.

Bind the long literal or the long `format!` to a `const` or a `let` before the chain, then run rustfmt so it lays the chain out. In `scroll/app.rs`, split the content view and the status line into locals.

Keep as they are:
- the `SERVICES` table in `incremental-rendering/app.rs` (data rows);
- the log `format!` string in the same file.

## 3. Repeated row match in tests

`tests/examples.rs` repeats `screen.rows().iter().any(|row| row.trim() == …)` five times. Add a helper `has_row(screen, text)` beside `contains` and use it.

## Check

Run only these:
- `rustfmt --edition 2024` on every file you touch;
- `awk 'length > 100 {print FILENAME": "FNR}' examples/*.rs examples/*/*.rs tests/examples.rs`: it may list only the `SERVICES` rows and the log format line in `incremental-rendering/app.rs`;
- `cargo test --offline --test examples --test text_width`;
- `cargo build --offline --examples`;
- `cargo clippy --offline --all-targets -- -D warnings`.

Never run `cargo test` without `--test`. Leave the work uncommitted.

## Report

In English: the files you changed, the awk output, and the command results.
