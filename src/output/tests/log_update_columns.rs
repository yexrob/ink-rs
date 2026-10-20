// Ported from ink test/log-update-columns.tsx @ 26d2c3f.

use super::Terminal;

#[test]
fn incremental_rendering_preserves_the_screen_across_shortened_and_grown_suffixes() {
    let mut t = Terminal::new(100, 10);
    for (i, value) in ["1000", "1", "200", ""].iter().enumerate() {
        let line = format!("UNCHANGED LEFT | {value}");
        let bytes = t.render(&[&line, "footer"], None);
        t.assert_rows(&[line.trim_end(), "footer"]);
        t.assert_cursor(0, 2, false);
        if i > 0 {
            assert!(!bytes.contains("UNCHANGED LEFT"));
        }
    }
}

#[test]
fn column_updates_preserve_explicit_cursor_positioning_without_a_trailing_newline() {
    let mut t = Terminal::new(100, 2);
    t.render(&["unchanged 0", "footer"], Some((2, 0)));
    let bytes = t.render(&["unchanged 1", "footer"], Some((2, 0)));
    assert!(!bytes.contains("unchanged"));
    t.assert_rows(&["unchanged 1", "footer"]);
    t.assert_cursor(2, 0, true);
}
