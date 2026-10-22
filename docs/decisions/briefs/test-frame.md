# Task: read the screen of a mounted test app

## Goal

bingo's TUI tests mount the whole app with `testing::mount` (raw input bytes, resize, suspend) and want to assert what the screen shows, as `TestInstance::last_frame` allows for `testing::render`. `TestApp` gives raw bytes through `TestTerminal::writes` and `output`, so a test has to parse escape sequences to read a frame. The requirement from bingo is `<repo>/docs/decisions/briefs/bingo-test-frame.md`.

Add two readers to `TestApp`, fed from the renderer's frame, not from parsing bytes.

## Read first

- `<repo>/CLAUDE.md`.
- `<repo>/docs/decisions/073-test-frame.md`.
- `<repo>/src/testing.rs`: `render`, `render_with`, `TestInstance::last_frame` and `frames`, `mount`, `TestApp`, `TestTerminal`.
- `<repo>/src/app.rs`: `AppOutput`, `paint` (the alternate-screen merge of `static_rows` into the live rows), `last_frame`, and where `paint` is called.
- The `Frame` type `paint` takes, its row type, and the code that turns a row into the text `TestInstance` stores (`serialize_row` or its equivalent).
- `<repo>/src/components.rs` (`Static`).
- `<repo>/docs/decisions/063-frame-grid.md`: when a step paints.

## Change

```rust
impl TestApp {
    /// The live frame from the latest paint, one line per row.
    pub fn live_frame(&self) -> String;
    /// Every row Static has written since mount, in write order.
    pub fn static_output(&self) -> String;
}
```

- Do not name the first one `last_frame`: `TestInstance::last_frame` includes the Static history, and the same name with another meaning misleads.
- **Where the rows come from**: `AppOutput::paint`, before the alternate-screen merge, so `live_frame` holds the live rows alone and `static_output` the Static rows alone, in inline mode and with `alternate_screen`.
- **Recording is off in production.** `AppOutput` gets a recorder that is `None` unless `testing::mount` turns it on. The driver path keeps no extra copy of the frame.
- **Format**: the row serialisation `TestInstance` uses, so a test written against `render` reads the same text from `mount`. Rows are joined by `\n`; say in the doc comment whether SGR sequences stay in, matching `TestInstance`.
- **`static_output`** accumulates: a change of the first `Static` instance does not clear it, because rows already written to the scrollback stay there. In alternate mode it holds every Static row, not the one viewport that `paint` replays.
- **Timing**: both read the latest paint. A step paints when the frame throttle allows, so a test calls `advance` after input to see the result. Say this in the doc comments.
- Readers return an empty string before the first paint.

What a test cannot read with these, to name in the doc comments and the report: the cursor position, colours as structured values, external writes through `use_stdout` and `use_stderr`, and images or other bytes from `write_control`.

## Tests

Two cases, one inline and one with `alternate_screen`: a mounted app renders a `Static` list and a live line. After input that adds an item and an `advance`, `static_output` holds the item and `live_frame` holds the live line and not the item. Two cases because the alternate-screen path merges Static rows into the live rows, and the recorder must take them before the merge.

## Out of scope

`docs/`, README, CHANGELOG, examples, the version number. `TestInstance` does not change.

## Rules

- `CLAUDE.md` holds. No `unsafe`.
- Behaviour outside these readers does not change; the existing tests are the check.
- Code reads like the existing code: naming, comment density, idiom.
- New public items get doc comments in the style of `TestInstance::last_frame`.
- No new dependencies.
- **The sandbox cannot write `.git`.** Do not commit. Do not write report files into the repository.

## Commands

- `export PATH=$HOME/.cargo/bin:$PATH` first; the mise shim cannot write its cache in the sandbox.
- `cargo fmt --all --check`
- `cargo clippy --offline --workspace --all-targets -- -D warnings`
- `cargo clippy --offline --workspace --all-targets --no-default-features -- -D warnings`
- `cargo test --offline --lib`
- `cargo test --offline --test static_runtime_blank_lines --test issue_973_static_commit --test suspension_output --test terminal_resize --test cursor`, plus the file you add the tests to.
- Run only these; the main agent runs the full suite.

## Report

Your final message is the report, in Chinese, without adverbs. It covers:

- the files you changed, and what each change does;
- the API and its format;
- where the recorder lives, how `testing::mount` turns it on, and why production keeps no copy;
- what a test cannot read;
- every deviation from this brief, and why;
- the tests you added;
- the exact commands you ran and their results.
