# ink-rs M0 phase 1: the text layer

You are working in `<repo>`, a from-scratch Rust counterpart of Ink (the React terminal UI library). Ink's tests are the spec. Read `CLAUDE.md` and `docs/design.md` first, the sections "跨层约定" and "布局与文本" in particular: they fix the decisions below.

The Ink reference checkout, with its `node_modules`, is at
`<scratchpad>/ink-src/ink`
(called `INK` below). It is reference data: read it, never run anything from it.

This phase builds the text layer only. Phase 2, a separate task, builds the host tree, layout, paint and `render_to_string` on top of it. Design the APIs so phase 2 can use them, but do not start phase 2.

## Architecture you must follow

- **Cells are the internal representation.** A string is parsed once into styled graphemes; everything after that works on them. Strings appear at the boundaries: `Text` content, the argument and return value of a `Transform`, the output of `render_to_string`.
- **Styles layer, they do not wrap.** A `Text`'s style is applied to the graphemes inside it by filling in attributes the inner content left unset. We do not reproduce chalk's string wrapping.
- **Wrapping follows wrap-ansi, not UAX #14.** Break at spaces, hard-break words longer than the width by grapheme, keep leading and trailing spaces (`trim: false`).
- **Only OSC 8 hyperlinks survive parsing.** Other OSC sequences (window title, clipboard, …) are dropped when a string is parsed into cells. A link keeps its exact parameters and its terminator (BEL or `ESC \`), so output reproduces the bytes. C1 forms are emitted in their ESC form.
- **No code that exists only to satisfy a test.** Implement the general algorithm. If a test expectation looks like an artifact of Ink's JavaScript implementation and the general algorithm cannot produce it, stop and report it; do not special-case it and do not change the expectation.

## Deliverables

### 1. `src/color.rs`

- `Color::hex` (const fn, panics on malformed input) and `FromStr for Color`: Ink's color strings, see the doc comments in the file, `INK/src/colorize.ts`, and `tests/colorize.rs`.
- Crate-internal conversion of a prop color to a color level, the way chalk 6 / ansi-styles 7 do it (`INK/node_modules/chalk/source`, `INK/node_modules/ansi-styles/index.js`): truecolor → 256 → 16. `ColorLevel::None` means no styling at all, as with chalk level 0, so this returns `Option<Color>`. This applies to colors coming from props (text, border, background). Raw SGR in user strings is never downgraded.
- Crate-internal SGR codes for a color: named colors use 30–37 / 90–97 (40–47 / 100–107 for background), `Ansi256` uses `38;5;n`, `Rgb` uses `38;2;r;g;b`.

### 2. `src/text/ansi.rs` and `src/text/sanitize.rs`

Port `INK/src/ansi-tokenizer.ts` (`tokenize`, `has_ansi_control_characters`) and `INK/src/sanitize-ansi.ts` (`sanitize_ansi`). These stay string-level: sanitizing is the security boundary every external string passes before it becomes cells. Their unit tests in `src/text/ansi/tests.rs` and `src/text/sanitize/tests.rs` are byte-exact and must pass unchanged.

### 3. `src/text/width.rs` (new)

Grapheme and string width with the rules of string-width 8.3 (`INK/node_modules/string-width/index.js`, `INK/node_modules/get-east-asian-width`). Use `unicode-segmentation` and `unicode-width` (already in `Cargo.toml`), and implement string-width's rules on top where `unicode-width` disagrees. In summary: graphemes made only of default-ignorable, control, format or non-spacing / enclosing marks are 0 wide; RGI emoji, ZWJ sequences and keycaps are 2; Hangul L+V(+T) is 2; otherwise the East Asian Width of the first visible code point (ambiguous counts as 1). `tests/text_width.rs` shows the cases Ink cares about (🍔, ⏳, 👩‍💻, `e` + U+0301, CJK, ZWSP); it is a render test that runs in phase 2, so read it rather than run it.

### 4. `src/text/styled.rs` (new): the cell representation

Shape (names may change if you have a better reason; `Style` is taken by the public layout style):

- `CharStyle`: `fg`, `bg: Option<Color>`; `bold`, `dim`, `italic`, `underline`, `inverse`, `hidden`, `strikethrough`, `overline: bool`; `link: Option<…>` holding the link's open sequence in ESC form (identity is the exact sequence; the close sequence is `ESC ] 8 ; ;` plus the same terminator).
- `StyledChar { grapheme, width, style }`, width in cells (0, 1 or 2).
- A line is a `Vec<StyledChar>`; text is a list of lines split at `\n`. The empty string has zero lines; `"a\n"` has two.
- `parse(&str)`: sanitize, then build cells. Apply SGR parameters (reset, the attributes above and their off codes, 16/256/truecolor fg and bg, default fg/bg); ignore SGR parameters outside that set. Handle C1 CSI/OSC/ST like their ESC forms. Keep OSC 8, drop other OSC. Segment graphemes with `unicode-segmentation`; a grapheme takes the style in effect at its first code point.
- `serialize(&[StyledChar]) -> String`, the canonical form for one line: emit a transition only where the style changes; open SGR before the link and close the link before SGR; close attributes with chalk's close codes (39, 49, 22, 23, 24, 27, 28, 29, 55; when 22 turns off one of bold / dim and the other stays, reopen it); close everything at the end of the line. Example: `"\x1b[31mabc…\x1b[39m"`.
- Style layering for phase 2: fill the unset attributes of a `CharStyle` from an outer one (colors when `None`, flags by OR, link when `None`).
- Tab expansion on a line, matching what Ink gets from `wrapAnsi(text, Infinity, {trim: false})` (`INK/node_modules/wrap-ansi/index.js` around line 450; see also `INK/src/squash-text-nodes.ts`).

### 5. `src/text/measure.rs` and `src/text/wrap.rs`

Replace the `&str` functions with functions over styled lines:

- measure: widest line in cells and the number of lines, as `TextSize`. Empty text is 0×0. Reference: `INK/src/measure-text.ts`.
- wrap: `(lines, max_width, TextWrap) -> lines`, porting `INK/src/wrap-text.ts` with wrap-ansi 10 (`trim: false, hard: true`, plus `wordWrap: false` for `Hard`) and cli-truncate 6 for the truncate modes (each line on its own, ellipsis `…`). Width 0: `Wrap` and `Hard` return the text unchanged, truncation cuts every line to nothing. The ellipsis takes the style of the adjacent grapheme without its link (see the expected values in `src/text/wrap/tests.rs`). No cache: caching moves onto host nodes in a later milestone.

Then adapt the call sites in `src/text/wrap/tests.rs` and `src/text/measure/tests.rs`: add a private helper in each `tests.rs` named like the old function (`wrap_text`, `measure_text`) that parses the input string, calls the new function and, for wrap, serializes the lines joined with `\n`. Do not change any test body or expected value.

## Rules

- No `unsafe`. No new tests. Temporary checks are fine while you work; delete them before you finish.
- Do not change expected values anywhere. Leave `tests/` and `tests/common` untouched.
- Keep `#![allow(dead_code)]` in `src/lib.rs`: phase 2 is the consumer of most of this API.
- Match the surrounding code: crate-internal items are `pub(crate)`; comments explain why (cite the Ink file when a rule comes from Ink), not what.
- No new dependencies. The sandbox has no network; set `CARGO_NET_OFFLINE=true`.
- Run only these: `cargo test --lib text::`, `cargo test --test colorize`, `cargo clippy --all-targets -- -D warnings`, and `rustfmt --edition 2024` on the files you touch. Never run `cargo test` without a filter or other integration tests: they hit `todo!()` until phase 2, and the full suite is run by the integrator.
- If cargo cannot lock the shared target directory in your sandbox, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target` and delete that directory when you are done.
- Do not commit.

## Report

When done, write a report with:

1. Files changed and the internal API you ended up with (signatures).
2. Design notes: anything phase 2 needs to know about the API.
3. Every place you deviated from Ink's behaviour, and why.
4. Test results of the commands above.
5. Open questions and expectations you suspect are Ink artifacts.
