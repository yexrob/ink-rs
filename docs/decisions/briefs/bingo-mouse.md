# Task: mouse input

## Goal

bingo draws buttons, panes and a band above the prompt, following Claude Code's fullscreen terminal: a click presses a button or moves focus, the wheel scrolls a pane, and hover changes a style. ink-rs has no mouse input; the docs list it as not ported.

This is a later need for bingo. Do it after the other briefs.

## Read first

- `<repo>/CLAUDE.md`; `<repo>/docs/design.md`, the input part.
- `<repo>/src/input/parser.rs` (CSI parsing), `<repo>/src/input/mod.rs` (`KeyEvent` and the input events), `<repo>/src/runtime/input.rs`.
- `<repo>/src/runtime/hooks.rs`: `use_input`, `use_paste`, `use_focus`, `use_node_ref`, `use_box_metrics`.
- `<repo>/src/app.rs`: how bracketed paste and the kitty keyboard are switched on while a hook needs them.

## Change

- Parse SGR mouse reports (`CSI < b ; x ; y M` and `m`, mode 1006) into a mouse event: press, release, drag, wheel up and down, with the button, modifiers and cell coordinates.
- `use_mouse(active, FnMut(MouseEvent))` (name open). Mouse reporting (1000 or 1002, with 1006) is on while a mouse hook is active and off on exit, suspend and panic, as bracketed paste is.
- Hit testing: a way to map a cell to the node under it, for example the screen rect of a node from `use_box_metrics`, or a helper that calls a handler when a click lands in a node's rect.
- Say how coordinates map in inline mode, where the live frame's screen row moves as `Static` output grows, and in the alternate screen.
- Mouse reporting takes the terminal's own text selection away. Document it; reporting is off by default.

## Tests

Parser tests for SGR reports, split reads and the escape timeout; a mounted test where a click in a box's rect reaches its handler.

## Report

The API, the coordinate rules for inline mode and the alternate screen, and what happens to text selection.
