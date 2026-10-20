// Ported from ink test/log-update-columns.tsx @ 26d2c3f.

use std::time::Duration;

use super::Terminal;
use crate::testing::{TestTerminal, mount};
use crate::{Options, Styled, Text, View};

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

#[test]
fn side_by_side_public_components_do_not_rewrite_the_unchanged_left_region() {
    let terminal = TestTerminal::new(60, 24);
    let frame = |count| {
        View::new()
            .width(50)
            .height(4)
            .child(View::new().width(20).child(Text::new("UNCHANGED LEFT")))
            .child(Text::new(format!("Counter: {count}")))
    };
    let mut app = mount(
        frame(0),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    );
    for count in 1..=3 {
        let before = terminal.writes().len();
        app.rerender(frame(count));
        app.advance(Duration::from_millis(34));
        assert!(
            !terminal.writes()[before..]
                .join("")
                .contains("UNCHANGED LEFT")
        );
        let mut screen = Terminal::new(60, 24);
        for write in terminal.writes() {
            screen.feed(write.as_bytes());
        }
        assert_eq!(
            screen.rows()[0],
            format!("UNCHANGED LEFT      Counter: {count}")
        );
    }
}
