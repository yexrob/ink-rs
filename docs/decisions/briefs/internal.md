Read the brief at <scratchpad>/porting-brief.md first and follow it.

You own the internal modules under src/text/ (ansi, sanitize, measure, wrap) and these Ink files: ansi-tokenizer.ts, sanitize-ansi.ts, measure-text.tsx, wrap-text.tsx, text-controls.tsx, colorize.ts.

Placement:
- ansi-tokenizer.ts (`tokenizeAnsi`, `hasAnsiControlCharacters`) → src/text/ansi/tests.rs.
- sanitize-ansi.ts, plus the `sanitizeAnsi(...)` tests from text-controls.tsx → src/text/sanitize/tests.rs.
- measure-text.tsx → src/text/measure/tests.rs.
- wrap-text.tsx: the `wrapText` unit tests → src/text/wrap/tests.rs; its render tests → tests/wrap_text.rs. `wrapTextCache` has no Rust counterpart; report those tests as "needs API" or "not applicable" with your reasoning. Tests using `renderAsync`/`rerenderAsync` are M1.
- text-controls.tsx render tests → tests/text_controls.rs.
- colorize.ts → tests/colorize.rs. Ink's `colorize(text, color, type)` parses the color string and wraps the text in SGR. The public Rust analog is `str::parse::<Color>()`: assert that each valid spelling parses to the expected `Color` and each invalid spelling returns `Err`. Note in the report that the SGR-wrapping half is covered by render tests.

Map Ink's token objects to `Token` variants in src/text/ansi.rs. Ink token types: text, csi (parameterString, intermediateString, finalCharacter), esc, osc/dcs/pm/apc/sos, st, c1, invalid. If a field Ink asserts has no place in `Token`, report it as an API gap instead of changing the enum.

Add the `#[cfg(test)]\nmod tests;` lines to the src/text modules you use; no other src edits.
