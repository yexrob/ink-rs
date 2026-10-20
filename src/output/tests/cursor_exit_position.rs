//! Ported from ink test/cursor-exit-position.tsx @ 26d2c3f.

use super::Terminal;

#[test]
fn done_returns_the_cursor_once_without_erasing_output_incremental_false() {
    let mut terminal = Terminal::new(100, 24);
    terminal.render(&["Header", "Footer"], Some((2, 0)));
    let mut bytes = Vec::new();
    terminal.output.done(&mut bytes);
    assert!(!bytes.is_empty());
    terminal.feed(&bytes);
    terminal.assert_rows(&["Header", "Footer"]);
    terminal.assert_cursor(0, 2, true);
    bytes.clear();
    terminal.output.done(&mut bytes);
    assert!(bytes.is_empty());
}
