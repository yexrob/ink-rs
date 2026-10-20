// Ported from ink test/log-update-blank-growth.ts @ 26d2c3f.

use super::Terminal;

fn blank_line_growth(rows: usize) {
    let mut t = Terminal::new(100, rows);
    t.feed(b"history1\r\nhistory2\r\nhistory3\r\n");
    t.render(&["A"], None);
    t.render(&["A", ""], None);
    t.render(&["B", ""], None);
    let mut lines = t.scrollback();
    lines.extend(t.rows());
    assert_eq!(&lines[..4], &["history1", "history2", "history3", "B"]);
    assert!(lines[4..].iter().all(String::is_empty));
    t.assert_cursor(0, 5.min(rows - 1) as u16, false);
}

#[test]
fn incremental_blank_line_growth_preserves_history_in_a_4_row_terminal() {
    blank_line_growth(4);
}

#[test]
fn incremental_blank_line_growth_preserves_history_in_a_6_row_terminal() {
    blank_line_growth(6);
}

#[test]
fn incremental_blank_line_growth_preserves_history_in_a_10_row_terminal() {
    blank_line_growth(10);
}
