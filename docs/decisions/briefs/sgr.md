Repo: <repo> (Rust port of Ink; tests ported from Ink, all algorithms still todo!()).
Read tests/common/mod.rs for styled() markup and assert_ansi_eq.

Problem: some ported tests check chalk's SGR bytes with output.contains("\u{1b}[41m") etc. That locks one encoding of the styles. Replace those checks with a full semantic comparison, assert_ansi_eq(&output, &styled(&[...])).

The full outputs were captured from Ink's own test run. They are in <scratchpad>/expectations-includes/<ink file>.md, one "## <test name>" section per test, with the includes() calls made and the output written as a Rust expression. Copy the styled(&[...]) expressions verbatim; do not derive outputs by hand.

Edit only these files:

1. tests/background.rs, tests/border_backgrounds.rs, tests/borders.rs: for each test with a section in background.tsx.md, border-backgrounds.tsx.md, borders.tsx.md, replace all positive output.contains(...) asserts in that test (SGR bytes, plain words, border chars: the full assertion covers them) with one assert_ansi_eq(&output, &styled(&[...])). Check the test renders with the columns the captured output implies (border-backgrounds rows are 100 cells wide).
2. tests/text.rs: only these three tests.
   - preserve_sgr_color_sequences_in_text: replace assert!(output.contains("\u{1b}[")) with the assert_ansi_eq from "preserve SGR color sequences in text".
   - preserve_colors_encoded_with_colon_parameters: delete the contains("\u{1b}[38;2;255;100;0m") line; the test already has the assert_ansi_eq.
   - preserve_sgr_sequences_around_stripped_sos_control_strings: replace assert!(output.contains("\u{1b}[")) with the assert_ansi_eq from its section.
   Leave every other assert in text.rs as it is (negative checks for stripped sequences, OSC checks, strip_ansi equalities).

Ignore components.tsx.md: those tests are runtime and not ported.

Rules: no unsafe; no new comments; do not edit src/ or tests/common. Format with rustfmt --edition 2024 on the files you edit. Verify with cargo test --no-run --test background --test border_backgrounds --test borders --test text and cargo clippy --all-targets -- -D warnings (if the shared target dir lock fails in your sandbox, use CARGO_TARGET_DIR under /private/tmp and delete it afterwards). Do not run the tests: they hit todo!(). Do not commit.

Report: per file, the tests changed and the asserts removed.
