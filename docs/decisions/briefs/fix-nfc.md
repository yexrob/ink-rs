# Task: NFC-normalize wrapped text, as Ink does

## Background

Ink wraps `Text` with wrap-ansi (`wrap` uses `{trim: false, hard: true}`; read `src/wrap-text.ts` in the Ink clone and `node_modules/wrap-ansi/index.js`). wrap-ansi calls `String.prototype.normalize('NFC')` on its input. Ink calls it only when the text is wider than the box (`src/render-node-to-output.ts` and the measure function in `src/dom.ts`); text that fits is written as given, and truncation modes do not normalize.

ink-rs does not normalize. The visible differences in wrapped text:
- decomposed sequences stay decomposed in the output bytes (`e` + U+0301 instead of `é`, Hangul jamo sequences instead of syllables);
- 42 split vowel-sign compositions (Tamil, Malayalam, Bengali and others, e.g. U+0B92/… with U+0BCA = U+0BC6 + U+0BBE) change width under string-width, so wrap points move.

`tests/text_width.rs` (around lines 84–92) expects un-normalized text when it fits. That must stay.

## Change

- Add the dependency `unicode-normalization = "0.1"` (0.1.25 is in the local registry; it supports Unicode 17). Build offline.
- In `src/text/wrap.rs` `wrap_text`, in the `TextWrap::Wrap | TextWrap::Hard` branch after the `max_width == 0` return, NFC-normalize each line before `wrap_line`. Each `StyledChar` cell holds one grapheme with its style: normalize the grapheme per cell and recompute the cell's width with the same grapheme-width function the rest of `src/text` uses. Composition stays inside a grapheme cluster (a base and its marks, a Hangul L V T sequence), so per-cell normalization matches normalizing the string, except across style changes inside one grapheme, which is fine.
- Confirm that `wrap_text` is reached for these modes only when the text does not fit, the way Ink does it, so fitting text stays as given. If ink-rs calls `wrap_text` for fitting text too, normalize only where the line is wider than `max_width`, and say so in the report.
- Measurement (layout height) must agree with paint: if measure and paint both go through `wrap_text`, nothing else is needed. Check.
- Keep the change small: a helper of a few lines, one comment citing wrap-ansi.

## Test

Add one test in `tests/text_width.rs`, next to the existing test at lines 84–92: a `Text` that must wrap, containing a decomposed `e` + U+0301 and a Tamil split vowel sign (base + U+0BC6 + U+0BBE), renders the composed forms (`é`, U+0BCA) and wraps at the composed widths. Compute the expected widths from string-width's rules (the composed Tamil vowel sign U+0BCA is a spacing mark).

## Docs

`docs/design.md` describes text wrapping (search 折行 / wrap-ansi). Add one sentence: wrap and hard modes NFC-normalize the lines they wrap, as wrap-ansi does; text that fits and truncation are not normalized.

## Commands

- `cargo test --offline --test text_width`
- `cargo test --offline --test text --test wrap` (only the ones that exist; list them with `ls tests`)
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
