// Ported from ink test/log-update.tsx @ 26d2c3f.
// The viewport height stands in for the trailing newline Ink picks by hand.

use super::Terminal;

const THREE: &[&str] = &["Line 1", "Line 2", "Line 3"];
const UPDATED: &[&str] = &["Line 1", "Updated", "Line 3"];
const FOUR: &[&str] = &["Line 1", "Line 2", "Line 3", "Line 4"];

#[test]
fn standard_rendering_renders_and_updates_output() {
    let mut t = Terminal::new(100, 10);
    t.render(&["Hello"], None);
    t.assert_rows(&["Hello"]);
    t.assert_cursor(0, 1, false);
    t.render(&["World"], None);
    t.assert_rows(&["World"]);
    t.assert_cursor(0, 1, false);
}

#[test]
fn standard_rendering_skips_identical_output() {
    let mut t = Terminal::new(100, 10);
    t.render(&["Hello"], None);
    assert!(t.render(&["Hello"], None).is_empty());
    t.assert_rows(&["Hello"]);
}

#[test]
fn incremental_rendering_surgical_updates() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, None);
    let bytes = t.render(UPDATED, None);
    assert!(!bytes.contains("Line 1"));
    assert!(!bytes.contains("Line 3"));
    t.assert_rows(UPDATED);
    t.assert_cursor(0, 3, false);
}

#[test]
fn incremental_rendering_same_height_update_rewinds_cursor_to_top_with_trailing_newline() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, None);
    t.render(UPDATED, None);
    t.assert_rows(UPDATED);
    t.assert_cursor(0, 3, false);
}

#[test]
fn incremental_rendering_clears_extra_lines_when_output_shrinks() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, None);
    t.render(&["Line 1"], None);
    t.assert_rows(&["Line 1"]);
    t.assert_cursor(0, 1, false);
}

#[test]
fn incremental_rendering_when_output_grows() {
    let mut t = Terminal::new(100, 10);
    t.render(&["Line 1"], None);
    let bytes = t.render(THREE, None);
    assert!(!bytes.contains("Line 1"));
    t.assert_rows(THREE);
    t.assert_cursor(0, 3, false);
}

#[test]
fn incremental_rendering_shrinking_output_keeps_screen_tight() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, None);
    t.render(&THREE[..2], None);
    t.render(&THREE[..1], None);
    t.assert_rows(&THREE[..1]);
    t.assert_cursor(0, 1, false);
}

#[test]
fn incremental_rendering_clear_fully_resets_incremental_state() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, None);
    t.clear();
    t.assert_rows(&[]);
    t.assert_cursor(0, 0, false);
    t.render(&THREE[..1], None);
    t.assert_rows(&THREE[..1]);
    t.assert_cursor(0, 1, false);
}

#[test]
fn incremental_rendering_done_resets_before_next_render() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, None);
    t.done();
    t.assert_rows(THREE);
    t.assert_cursor(0, 3, true);
    t.render(&THREE[..1], None);
    t.assert_rows(&["Line 1", "Line 2", "Line 3", "Line 1"]);
    t.assert_cursor(0, 4, false);
}

#[test]
fn incremental_rendering_multiple_consecutive_clear_calls_should_be_harmless_no_ops() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, None);
    t.clear();
    assert!(t.clear().is_empty());
    assert!(t.clear().is_empty());
    t.assert_rows(&[]);
    t.render(&["New content"], None);
    t.assert_rows(&["New content"]);
    t.assert_cursor(0, 1, false);
}

#[test]
fn standard_rendering_positions_cursor_after_output_when_cursor_position_is_set() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, Some((5, 1)));
    t.assert_rows(THREE);
    t.assert_cursor(5, 1, true);
}

#[test]
fn standard_rendering_hides_cursor_before_erase_when_cursor_was_previously_shown() {
    let mut t = Terminal::new(100, 10);
    t.render(&["Hello"], Some((0, 0)));
    t.render(&["World"], Some((0, 0)));
    t.assert_rows(&["World"]);
    t.assert_cursor(0, 0, true);
}

#[test]
fn standard_rendering_no_cursor_positioning_when_cursor_position_is_undefined() {
    let mut t = Terminal::new(100, 10);
    t.render(&["Hello"], None);
    t.assert_rows(&["Hello"]);
    t.assert_cursor(0, 1, false);
}

#[test]
fn standard_rendering_cursor_position_at_second_to_last_line_emits_cursor_up_1() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, Some((3, 2)));
    t.assert_rows(THREE);
    t.assert_cursor(3, 2, true);
}

fn clear_with_cursor(y: usize) {
    let mut t = Terminal::new(100, 10);
    t.feed(b"shell\r\n");
    t.render(THREE, Some((5, y)));
    t.clear();
    t.assert_rows(&["shell"]);
    t.assert_cursor(0, 1, false);
}

#[test]
fn standard_rendering_clear_erases_the_frame_with_the_cursor_at_row_0() {
    clear_with_cursor(0);
}

#[test]
fn standard_rendering_clear_erases_the_frame_with_the_cursor_at_row_5() {
    clear_with_cursor(5);
}

#[test]
fn standard_rendering_clearing_cursor_position_stops_cursor_positioning() {
    let mut t = Terminal::new(100, 10);
    t.render(&["Hello"], Some((0, 0)));
    t.render(&["World"], None);
    t.assert_rows(&["World"]);
    t.assert_cursor(0, 1, false);
}

#[test]
fn incremental_rendering_positions_cursor_after_update() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, Some((2, 0)));
    t.render(UPDATED, Some((2, 0)));
    t.assert_rows(UPDATED);
    t.assert_cursor(2, 0, true);
}

#[test]
fn standard_rendering_repositions_cursor_when_only_cursor_position_changes_same_output() {
    let mut t = Terminal::new(100, 10);
    t.render(&["Hello"], Some((2, 0)));
    let bytes = t.render(&["Hello"], Some((3, 0)));
    assert!(!bytes.contains("Hello"));
    t.assert_rows(&["Hello"]);
    t.assert_cursor(3, 0, true);
}

#[test]
fn standard_rendering_returns_to_bottom_before_erase_when_cursor_was_positioned() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, Some((0, 0)));
    t.render(&["Line A", "Line B", "Line C"], Some((5, 0)));
    t.assert_rows(&["Line A", "Line B", "Line C"]);
    t.assert_cursor(5, 0, true);
}

#[test]
fn incremental_rendering_single_line_without_trailing_newline_stays_on_the_same_row() {
    let mut t = Terminal::new(100, 1);
    t.render(&["Before"], None);
    t.render(&["After"], None);
    t.assert_rows(&["After"]);
    assert!(t.scrollback().is_empty());
    assert_eq!(t.parser.screen().cursor_position().0, 0);
}

#[test]
fn incremental_rendering_growing_a_single_line_without_trailing_newline_starts_on_the_same_row() {
    let mut t = Terminal::new(100, 2);
    t.render(&["First"], None);
    t.render(&["First", "Second"], None);
    t.assert_rows(&["First", "Second"]);
    assert!(t.scrollback().is_empty());
}

#[test]
fn incremental_rendering_no_trailing_newline_no_trailing_to_no_trailing_update() {
    let mut t = Terminal::new(100, 2);
    t.render(&["A", "B"], None);
    t.render(&["A", "C"], None);
    t.assert_rows(&["A", "C"]);
    assert!(t.scrollback().is_empty());
}

#[test]
fn incremental_rendering_no_trailing_newline_shrink() {
    let mut t = Terminal::new(100, 2);
    t.render(&["A", "B"], None);
    // Ink rests on row 0; a short frame rests on the row below it (design.md).
    t.render(&["A"], None);
    t.assert_rows(&["A"]);
    t.assert_cursor(0, 1, false);
    assert!(t.scrollback().is_empty());
}

#[test]
fn incremental_rendering_no_trailing_newline_cursor_after_shrink() {
    let mut t = Terminal::new(100, 3);
    t.render(THREE, Some((2, 0)));
    t.render(&THREE[..2], Some((2, 0)));
    t.assert_rows(&THREE[..2]);
    t.assert_cursor(2, 0, true);
}

#[test]
fn incremental_rendering_no_trailing_newline_grow() {
    let mut t = Terminal::new(100, 3);
    t.render(&["A"], None);
    t.render(&["A", "B", "C"], None);
    t.assert_rows(&["A", "B", "C"]);
    assert!(t.scrollback().is_empty());
}

#[test]
fn incremental_rendering_no_trailing_newline_unchanged_lines_do_not_overshoot_cursor() {
    let mut t = Terminal::new(100, 2);
    t.render(&["A", "B"], None);
    assert!(t.render(&["A", "B"], None).is_empty());
    t.render(&["X", "B"], None);
    t.assert_rows(&["X", "B"]);
    assert_eq!(t.parser.screen().cursor_position().0, 1);
    assert!(t.scrollback().is_empty());
}

#[test]
fn incremental_rendering_no_trailing_newline_cursor_lands_on_the_target_line() {
    let mut t = Terminal::new(100, 4);
    t.render(FOUR, Some((7, 1)));
    t.assert_rows(FOUR);
    t.assert_cursor(7, 1, true);
    t.render(&["Line 1", "Line 2!", "Line 3", "Line 4"], Some((7, 1)));
    t.assert_rows(&["Line 1", "Line 2!", "Line 3", "Line 4"]);
    t.assert_cursor(7, 1, true);
}

#[test]
fn standard_rendering_no_trailing_newline_cursor_lands_on_the_target_line() {
    let mut t = Terminal::new(100, 4);
    t.render(FOUR, Some((7, 1)));
    t.assert_rows(FOUR);
    t.assert_cursor(7, 1, true);
}

#[test]
fn standard_rendering_cursor_only_update_with_no_trailing_newline_lands_on_the_target_line() {
    let mut t = Terminal::new(100, 4);
    t.render(FOUR, Some((0, 3)));
    let bytes = t.render(FOUR, Some((5, 0)));
    assert!(!bytes.contains("Line"));
    t.assert_rows(FOUR);
    t.assert_cursor(5, 0, true);
    assert!(t.scrollback().is_empty());
}

#[test]
fn incremental_rendering_render_to_empty_string_full_clear_vs_early_exit() {
    let mut t = Terminal::new(100, 10);
    t.render(THREE, None);
    t.render(&[], None);
    t.assert_rows(&[]);
    t.assert_cursor(0, 1, false);
    assert!(t.render(&[], None).is_empty());
}
