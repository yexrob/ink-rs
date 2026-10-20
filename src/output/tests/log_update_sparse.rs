// Ported from ink test/log-update-sparse.ts @ 26d2c3f.
// CSI cursor-forward gaps are blank cells in a paint::Frame.

use super::Terminal;

#[test]
fn incremental_updates_clear_old_text_beneath_cursor_forward_gaps() {
    let mut t = Terminal::new(5, 1);
    t.render(&["aaaaa"], None);
    t.render(&["b   X"], None);
    t.assert_rows(&["b   X"]);
    assert!(t.scrollback().is_empty());
}

fn changed_rows(rows: usize) {
    let mut t = Terminal::new(5, rows);
    t.render(&["aaaaa"], None);
    t.render(&["b   X"], None);
    t.assert_rows(&["b   X"]);
    assert!(t.scrollback().is_empty());
}

#[test]
fn changed_rows_clear_before_painting_trailing_newline_false() {
    changed_rows(1);
}

#[test]
fn changed_rows_clear_before_painting_trailing_newline_true() {
    changed_rows(2);
}

#[test]
fn full_width_updates_do_not_erase_after_the_last_character() {
    let mut t = Terminal::new(5, 1);
    t.render(&["aaaaa"], None);
    let bytes = t.render(&["bbbbX"], None);
    t.assert_rows(&["bbbbX"]);
    // vt100's pending-wrap EL is a no-op; xterm erases the last cell.
    assert!(!bytes.contains("X\x1b[K"));
    assert!(!bytes.contains("\x1b[K"));
}

#[test]
fn full_width_prefix_updates_clear_only_the_changed_suffix() {
    let mut t = Terminal::new(5, 1);
    t.render(&["LEFTX"], None);
    for suffix in ["Y", ""] {
        let line = format!("LEFT{suffix}");
        let bytes = t.render(&[&line], None);
        assert!(!bytes.contains("LEFT"));
        t.assert_rows(&[&line]);
        assert!(t.scrollback().is_empty());
    }
}

#[test]
fn shorter_empty_and_styled_rows_remove_old_trailing_content() {
    let mut t = Terminal::new(10, 2);
    for (row, expected) in [
        ("long text", "long text"),
        ("\x1b[31mred\x1b[39m", "red"),
        ("x", "x"),
        ("", ""),
    ] {
        t.render(&[row, "end"], None);
        t.assert_rows(&[expected, "end"]);
        if expected == "red" {
            assert_eq!(
                t.parser.screen().cell(0, 0).unwrap().fgcolor(),
                vt100::Color::Idx(1)
            );
        }
        assert_eq!(
            t.parser.screen().cell(1, 0).unwrap().fgcolor(),
            vt100::Color::Default
        );
    }
}

#[test]
fn sparse_updates_preserve_unchanged_rows_and_the_requested_cursor() {
    let mut t = Terminal::new(5, 3);
    t.render(&["top", "aaaaa", "end"], Some((2, 1)));
    let lines = &["top", "b   X", "end"];
    let bytes = t.render(lines, Some((2, 1)));
    assert!(!bytes.contains("top"));
    assert!(!bytes.contains("end"));
    t.assert_rows(lines);
    t.assert_cursor(2, 1, true);
    assert!(t.render(lines, Some((2, 1))).is_empty());
}

#[test]
fn new_rows_can_grow_and_shrink_around_sparse_updates() {
    let mut t = Terminal::new(5, 5);
    t.render(&["top", "aaaaa"], None);
    t.render(&["top", "b   X", "end"], None);
    t.render(&["top", "c  Y"], None);
    t.assert_rows(&["top", "c  Y", "", "", ""]);
    t.assert_cursor(0, 2, false);
}
