# ink-rs M4 phase 2: review fixes

You are working in the same worktree as before: `<scratchpad>/wt/m4-p2`. Your earlier work is there, uncommitted. The review accepted the hook coordination, the guarded regions, the tree replacement and the exit precedence. Fix the items below and change nothing else.

The examples worker has finished. You may now edit `src/lib.rs`.

## 1. Module placement

- Declare `mod error_overview;` in `src/lib.rs` as a private crate module. Remove the `#[path]` declaration from `src/app.rs`, and call `crate::error_overview::build`.
- Use plain `#[cfg(test)] mod tests;` in `src/app.rs` and in `src/error_overview.rs`. Rust finds `src/app/tests.rs` and `src/error_overview/tests.rs` without `#[path]`.
- In `src/lib.rs`, export `PanicError` and `PanicLocation` beside `Error`.

## 2. The ported overview tests

`tests/error_overview.rs` compiles `src/error_overview.rs` a second time through `#[path]`. Its `use crate::{…}` then resolves against the test crate's imports, and the builder's own unit tests run twice. Ink's `ErrorOverview` is an internal component. `docs/design.md` (移植 Ink 测试的方法) puts tests of internal code in the module's `tests.rs`.
- Move the six ported tests into `src/error_overview/tests.rs`, in Ink's order, ahead of the existing builder tests. Add a comment naming the Ink file and commit above them.
- Delete `tests/error_overview.rs`. Keep `tests/fixtures/error source.txt`: unit tests run with the crate root as cwd.

## 3. Stack height

A manual check in a 100×30 tmux pane, with `RUST_BACKTRACE=1` (the user's shell sets it), panicked in a component via `.expect()`. The overview listed every frame below the panic: the ink-rs runtime, the std closures, `std::rt::lang_start` and `_main`, about 60 frames. Many symbols wrapped over 2–4 rows. The frame was taller than the screen, and the app height cap cut the top. So the ERROR line, the message, the location and the excerpt never reached the terminal. Only the tail of the stack was visible. Without `RUST_BACKTRACE`, the overview looked right.

Change `frames()`:
- Drop every frame whose symbol is in `std`, `core`, `alloc`, `panic_unwind`, `__rustc` or the two hooks, including the `<std::…`, `<core::…` and `<alloc::…` impl forms. This drops frames anywhere in the stack, not only leading ones. Ink does the same: `stack-utils` with `nodeInternals()` removes Node's internal frames.
- Then cut the stack at the first `ink_rs` frame (including `<ink_rs::…`) that follows a non-`ink_rs` frame. That frame is the runtime calling the component. Drop it and everything below it.
- Keep leading `ink_rs` frames. They show the ink-rs API that panicked, such as `State::get`.
- Frames without a symbol path, such as `_main`, follow the same rules as the others.

Change the row layout: render each frame as one `Text` with nested children for `- `, the bold symbol and the gray ` (location)`. A long Rust symbol then wraps as one paragraph. Today it wraps as two columns side by side. Keep the colors. Note this departure from Ink's three-`Text` row in the report.

Add builder tests in `src/error_overview/tests.rs` for the new rule, from crafted std-format backtrace text:
- leading std/core frames, user frames with a `core::iter` frame between them, then ink-rs runtime frames and `std::rt` frames, then `_main`: only the user frames remain;
- a panic inside an ink-rs API: the leading `ink_rs` frame stays.

In `tests/errors.rs`, `catch_and_display_error` uses a taller terminal to avoid stack clipping. Return it to Ink's size if the trimmed stack fits, so the test also guards the height.

## 4. `Runtime::guard`'s flag

The parameter `overview: bool` reads as "this is the overview". Rename it after what it does. For example, `rethrow` with a doc comment: true for the regions that render the overview itself. A panic there while the overview is mounted is resumed, so a broken overview cannot loop.

## Check

Run only these:
- `rustfmt --edition 2024` on every file you touch;
- `cargo test --offline --test errors --test render --test render_to_string --test pty --test exit`;
- `cargo test --offline --lib`;
- `cargo clippy --offline --all-targets -- -D warnings`.

Never run `cargo test` without `--lib` or `--test`. Leave the work uncommitted.

## Report

In English:
- the files you changed;
- the new `frames()` rule as built;
- the adapted tests;
- the command results.
