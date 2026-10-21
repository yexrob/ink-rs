Read the brief at <scratchpad>/porting-brief.md first and follow it.

Your Ink files (check exact extensions in the test dir): text, render-to-string, text-width, styled-combining-marks, static-blank-lines, static-trailing-layout, ansi-newlines, rendering-regressions, squash-text-nodes, c1-rendering, colon-colors. Write one tests/<name>.rs per file (dashes → underscores).

Notes:
- text (tab-stop tests) and styled-combining-marks set `chalk.level = 3` in some tests; those use `render_truecolor`.
- render-to-string tests the library's `renderToString` (80 columns default, Static output first). Parts of it use effects, state or `renderToStringAsync`; those are runtime (M1) or concurrent duplicates.
- Static in a single render is M0; Static across rerenders is M2.
