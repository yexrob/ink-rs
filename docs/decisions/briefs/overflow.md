Read the brief at <scratchpad>/porting-brief.md first and follow it.

Your Ink files (check exact extensions in the test dir): overflow, content-offset, position, absolute-truncation, clip-wide-background, overlap-wide-background, display, truncate-width. Write one tests/<name>.rs per file (dashes → underscores).

clip-wide-background and overlap-wide-background set `chalk.level = 3`, so they use `render_truecolor`.
