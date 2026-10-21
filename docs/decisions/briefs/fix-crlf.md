# Task: no blank line after external writes that end in a newline plus escapes

## Bug

`App::external_write` in `src/app.rs` (around line 240) appends `\r\n` when `data` does not end with `\n`:

```rust
let mut data_bytes = data.as_bytes().to_vec();
if !data.ends_with('\n') {
    data_bytes.extend_from_slice(b"\r\n");
}
```

The `\r\n` exists because the redraw after the data starts at column 0 of the row below the data (see the comment above it, and the paragraph 外部输出的换行 in `docs/design.md`). But colored output often ends with a reset after its newline: `"\x1b[31mabc\n\x1b[0m"` (owo-colors, chalk). Data like `"abc\n\r"` does the same. In both cases the cursor already sits at column 0 of a fresh row, and the extra `\r\n` leaves a blank line between the data and the frame.

## Fix

Decide whether the data leaves the cursor at column 0 of a fresh row by looking at its last printable text, not its last byte:

- Tokenize the data with `crate::text::ansi::tokenize` (`src/text/ansi.rs`).
- Walk back from the end. Skip every token that is not `Token::Text` (CSI, ESC, control strings, ST, C1, invalid sequences), and skip `\r` characters inside text.
- If the last remaining character is `\n`, or no text remains (empty data or escapes only), append nothing. Otherwise append `\r\n` as today.

Keep the logic small: a private helper in `src/app.rs` (or `src/text/ansi.rs` if it reads better there). Data newlines still go out as written (the TTY's ONLCR handles them). The non-interactive path is unchanged.

`"abc\r"` still gets `\r\n`: the cursor is at column 0, but on the data's own row, and the redraw would overwrite `abc`.

## Docs

Update the paragraph **外部输出的换行** in `docs/design.md` (around line 146): the rule is now "the data's last text character, ignoring escape sequences and `\r`, is not `\n`".

## Test

Add one test to `tests/hooks.rs`, next to `use_stdout_write_to_stdout` / `use_stderr_write_to_stderr`. It covers both streams: an interactive TTY app writes `"\x1b[31mabc\n\x1b[0m"` through `use_stdout().write` and through `use_stderr().write`, and the screen (use the `Screen` helper in `tests/common/mod.rs`, which models ONLCR) shows `abc` directly above the frame with no blank row between them. Look at how the existing tests in `tests/hooks.rs` and `tests/render.rs` set up interactive TTY apps and screens, and follow them.

## Commands

- `cargo test --offline --test hooks`
- `cargo test --offline --lib`

## Project context

ink-rs is a Rust port of Ink (React for CLIs). Ink's tests are the spec. Read `CLAUDE.md`, `AGENTS.md` and the relevant parts of `docs/design.md` before you start. A read-only clone of Ink, with its `node_modules`, is at
`<scratchpad>/ink-src/ink`. Read and grep it; never run anything from it.

## Rules

- No unsafe code. No dependencies beyond the ones this brief names. Code reads like the surrounding code: same naming, comment density and idiom. No code written only to satisfy a test.
- Add only the tests this brief asks for. Existing tests keep their expectations unless this brief says otherwise; if one must change, say which and why in the report.
- Docs are in Chinese, in the style of `docs/design.md`: plain statements, no adverbs. Update only the doc passages this brief names, plus any other passage your change makes false.
- Stay inside the files this brief names. Other workers are changing other parts of the tree in parallel.
- Run only the commands this brief lists, all with `--offline`, plus `cargo fmt --all --check` and `cargo clippy --offline --workspace --all-targets -- -D warnings`. Do not run the full `cargo test`; I run it at integration.
- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- The sandbox cannot write `.git`. Do not commit. Leave the changes in the worktree. Do not write report files into the worktree.

## Report

Your final message is the report, in Chinese, no adverbs:
- the files you changed, and what changed in each;
- design choices the brief left open, and what you chose;
- every deviation from this brief, and why;
- the exact commands you ran and their results.
