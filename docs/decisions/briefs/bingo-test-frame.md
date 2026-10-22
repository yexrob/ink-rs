# Task: read the screen of a mounted test app

## Goal

bingo's TUI tests mount the whole app with `testing::mount` (the real driver path: raw input bytes, resize, suspend) and want to assert what the screen shows, as `TestInstance::last_frame` allows for `testing::render`. `TestApp` gives raw bytes through `TestTerminal::writes` and `output`, so a test has to parse escape sequences to read a frame.

## Read first

- `<repo>/CLAUDE.md`.
- `<repo>/src/testing.rs`: `render`, `render_with`, `TestInstance::last_frame` and `frames`, `mount`, `TestApp`, `TestTerminal`.
- `<repo>/docs/decisions/063-frame-grid.md` and the frame and diff code it names: how a frame becomes cells and bytes.
- `<repo>/src/components.rs` (`Static`): how static output and the live frame are written.

## Change

- `TestApp::last_frame(&self) -> String` (name open): the live frame as text after the latest paint, in the format `TestInstance::last_frame` uses.
- A way to read what `Static` has written to the scrollback, for example `TestApp::static_output() -> String`, so a test sees transcript items that left the live frame.
- Both work in inline mode and with `alternate_screen`.
- Prefer the renderer's own frame grid over parsing the bytes. If you parse the bytes, say why.

## Tests

A mounted app renders a `Static` list and a live line. After input that adds an item, `static_output` holds the item and `last_frame` the live line.

## Report

The API, where the frame comes from, and what a test cannot read with it.
