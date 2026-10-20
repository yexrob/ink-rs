// Ported from ink test/log-update-blank-growth.ts @ 26d2c3f.

use std::time::Duration;

use super::Terminal;
use crate::testing::{TestTerminal, mount};
use crate::{Options, Styled, Text, View};

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

#[test]
fn growing_a_rendered_box_with_an_empty_row_preserves_terminal_history() {
    let terminal = TestTerminal::new(100, 4);
    terminal.write("history1\r\nhistory2\r\nhistory3\r\n");
    let frame = |height, text| View::new().height(height).child(Text::new(text));
    let mut app = mount(frame(1, "A"), Options::default(), &terminal);
    for (height, text) in [(2, "A"), (2, "B")] {
        app.rerender(frame(height, text));
        app.advance(Duration::from_millis(34));
    }
    let mut screen = Terminal::new(100, 4);
    for write in terminal.writes() {
        screen.feed(write.as_bytes());
    }
    let mut lines = screen.scrollback();
    lines.extend(screen.rows());
    assert_eq!(lines, ["history1", "history2", "history3", "B", "", ""]);
}
