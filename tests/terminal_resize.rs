//! Ported from ink test/terminal-resize.tsx @ 26d2c3f.
mod common;

use std::cell::RefCell;
use std::time::Duration;

use common::{Screen, assert_no_full_clear};
use ink_rs::testing::{TestApp, TestTerminal, mount};
use ink_rs::*;

thread_local! {
    static TERMINAL: RefCell<Option<TestTerminal>> = const { RefCell::new(None) };
    static INITIAL_SIZE: RefCell<Option<WindowSize>> = const { RefCell::new(None) };
}

fn dimensions(cx: &mut Scope, _: &()) -> Element {
    let size = cx.use_window_size();
    Text::new(format!("{}x{}", size.columns, size.rows)).into()
}

#[test]
fn use_window_size_catches_resizes_before_its_subscription_is_installed() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let size = cx.use_window_size();
        INITIAL_SIZE.with_borrow_mut(|initial| {
            initial.get_or_insert(size);
        });
        cx.use_layout_effect((), || {
            TERMINAL.with_borrow(|terminal| terminal.as_ref().unwrap().resize(60, 20));
        });
        Text::new(format!("{}x{}", size.columns, size.rows)).into()
    }
    let terminal = TestTerminal::new(100, 40);
    TERMINAL.set(Some(terminal.clone()));
    INITIAL_SIZE.set(None);
    // The live driver replaces Ink's debug stream so an effect can queue an event.
    let mut app = mount(component(example, ()), Options::default(), &terminal);
    assert_eq!(
        INITIAL_SIZE.with_borrow(|size| *size),
        Some(WindowSize {
            columns: 100,
            rows: 40
        })
    );
    let mut screen = Screen::new(&terminal, 100, 40);
    screen.resize(60, 20);
    app.advance(Duration::ZERO);
    screen.update(&terminal);
    assert_eq!(screen.rows()[0], "60x20");
    TERMINAL.set(None);
}

#[test]
fn use_window_size_returns_current_terminal_dimensions_and_updates_on_resize() {
    let terminal = TestTerminal::new(100, 40);
    let mut app = mount(component(dimensions, ()), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 100, 40);
    assert_eq!(screen.rows()[0], "100x40");
    screen.resize(60, 20);
    app.resize(60, 20);
    app.advance(Duration::from_millis(100));
    screen.update(&terminal);
    assert_eq!(screen.rows()[0], "60x20");
}

#[test]
fn use_window_size_falls_back_to_a_positive_column_count_when_stdout_columns_is_0() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let size = cx.use_window_size();
        assert!(size.columns > 0);
        Text::new(size.columns.to_string()).into()
    }
    let terminal = TestTerminal::new(0, 24);
    let _app = mount(component(example, ()), Options::default(), &terminal);
}

fn bordered(_: &mut Scope, text: &&'static str) -> Element {
    View::new()
        .border_style(BorderStyle::Round)
        .child(Text::new(*text))
        .into()
}

#[test]
fn clear_screen_when_terminal_width_decreases() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(bordered, "Hello World"),
        Options::default(),
        &terminal,
    );
    let mut screen = Screen::new(&terminal, 100, 24);
    let initial = screen.rows();
    assert!(initial[1].contains("Hello World"));
    assert!(initial[0].contains('╭'));
    screen.resize(50, 24);
    app.resize(50, 24);
    app.advance(Duration::from_millis(100));
    screen.update(&terminal);
    assert_ne!(screen.rows(), initial);
    assert_eq!(
        &screen.rows()[..4],
        [
            "╭────────────────────────────────────────────────╮",
            "│Hello World                                     │",
            "╰────────────────────────────────────────────────╯",
            "",
        ]
    );
}

#[test]
fn no_screen_clear_when_terminal_width_increases() {
    let terminal = TestTerminal::new(50, 24);
    let mut app = mount(component(bordered, "Test"), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 50, 24);
    let initial = screen.rows();
    screen.resize(100, 24);
    app.resize(100, 24);
    app.advance(Duration::from_millis(100));
    screen.update(&terminal);
    assert_ne!(screen.rows(), initial);
    assert!(screen.rows()[1].contains("Test"));
    assert_no_full_clear(&terminal);
}

#[test]
fn consecutive_width_decreases_trigger_screen_clear_each_time() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(bordered, "Content"),
        Options::default(),
        &terminal,
    );
    let mut screen = Screen::new(&terminal, 100, 24);
    for columns in [80, 60] {
        let previous = screen.rows();
        screen.resize(columns, 24);
        app.resize(columns, 24);
        app.advance(Duration::from_millis(100));
        screen.update(&terminal);
        assert_ne!(screen.rows(), previous);
        assert!(screen.rows()[1].contains("Content"));
    }
}

#[test]
fn width_decrease_clears_last_output_to_force_rerender() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(bordered, "Test Content"),
        Options::default(),
        &terminal,
    );
    let mut screen = Screen::new(&terminal, 100, 24);
    let initial = screen.rows();
    screen.resize(50, 24);
    app.resize(50, 24);
    app.advance(Duration::from_millis(100));
    screen.update(&terminal);
    assert_ne!(screen.rows(), initial);
    assert!(screen.rows()[1].contains("Test Content"));
    app.rerender(
        View::new()
            .border_style(BorderStyle::Round)
            .child(Text::new("Updated Content")),
    );
    app.advance(Duration::from_millis(100));
    screen.update(&terminal);
    assert!(screen.rows()[1].contains("Updated Content"));
}

fn letters(cx: &mut Scope, (suffix, cursor): &(&'static str, Option<usize>)) -> Element {
    cx.use_cursor(cursor.map(|y| CursorPosition { x: 0, y }));
    View::new()
        .flex_direction(FlexDirection::Column)
        .children(
            ["A", "B", "C", "D", "E", "F"]
                .map(|letter| Text::new(format!("{letter}{suffix}")).key(letter)),
        )
        .into()
}

fn shell_frame(element: Element, columns: u16, rows: u16) -> (TestTerminal, TestApp, Screen) {
    let terminal = TestTerminal::new(columns, rows);
    terminal.write("shell 0\r\nshell 1\r\nshell 2\r\n");
    let app = mount(element, Options::default(), &terminal);
    let screen = Screen::new(&terminal, columns, rows);
    (terminal, app, screen)
}

fn trimmed(screen: &mut Screen) -> Vec<String> {
    let mut lines = screen.lines();
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

fn cursor_shrink(cursor: usize, rows: u16) {
    let (terminal, mut app, mut screen) =
        shell_frame(component(letters, ("", Some(cursor))), 100, 10);
    screen.resize(100, rows);
    app.resize(100, rows);
    screen.update(&terminal);
    assert_eq!(
        trimmed(&mut screen),
        [
            "shell 0", "shell 1", "shell 2", "A", "B", "C", "D", "E", "F"
        ]
    );
    app.rerender(component(letters, ("!", Some(cursor))));
    app.advance(Duration::from_millis(34));
    screen.update(&terminal);
    assert_eq!(
        trimmed(&mut screen),
        [
            "shell 0", "shell 1", "shell 2", "A!", "B!", "C!", "D!", "E!", "F!"
        ]
    );
}

#[test]
fn rows_shrink_with_a_cursor_above_the_output_bottom_keeps_lines_above_the_frame_cursor_row_0_8_rows()
 {
    cursor_shrink(0, 8);
}

#[test]
fn rows_shrink_with_a_cursor_above_the_output_bottom_keeps_lines_above_the_frame_cursor_row_4_7_rows()
 {
    cursor_shrink(4, 7);
}

#[test]
fn width_and_height_shrinking_together_keeps_lines_above_the_frame() {
    let (terminal, mut app, mut screen) = shell_frame(component(letters, ("", Some(0))), 100, 10);
    screen.resize(50, 6);
    app.resize(50, 6);
    screen.update(&terminal);
    assert_eq!(
        trimmed(&mut screen),
        [
            "shell 0", "shell 1", "shell 2", "A", "B", "C", "D", "E", "F"
        ]
    );
}

#[test]
fn rows_shrink_right_after_a_commit_clears_the_cursor_keeps_lines_above_the_frame() {
    let (terminal, mut app, mut screen) = shell_frame(component(letters, ("", Some(0))), 100, 10);
    app.rerender(component(letters, ("", None)));
    screen.resize(100, 8);
    app.resize(100, 8);
    screen.update(&terminal);
    assert_eq!(
        trimmed(&mut screen),
        [
            "shell 0", "shell 1", "shell 2", "A", "B", "C", "D", "E", "F"
        ]
    );
    assert!(!screen.cursor_visible());
}

fn six_lines(_: &mut Scope, _: &()) -> Element {
    View::new()
        .flex_direction(FlexDirection::Column)
        .children((0..6).map(|i| Text::new(format!("line {i}")).key(i)))
        .into()
}

const LINES: [&str; 6] = ["line 0", "line 1", "line 2", "line 3", "line 4", "line 5"];

#[test]
fn standard_rendering_erases_and_rewrites_the_frame_when_the_terminal_height_shrinks_onto_it() {
    let terminal = TestTerminal::new(40, 10);
    let mut app = mount(component(six_lines, ()), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 40, 10);
    assert_eq!(&screen.rows()[..6], LINES);
    assert_eq!(screen.cursor(), (6, 0));
    screen.resize(40, 6);
    app.resize(40, 6);
    screen.update(&terminal);
    // Ink pins homeAndEraseDown + frame; retain its visible frame, history
    // and no-trailing-newline intent without pinning cell-diff bytes.
    assert_eq!(screen.rows(), LINES);
    assert_eq!(screen.scrollback(), ["line 0"]);
    assert_eq!(screen.cursor(), (5, 0));
}

#[test]
fn standard_rendering_keeps_content_above_the_frame_when_the_terminal_height_shrinks_onto_it_and_a_custom_cursor_sits_on_it()
 {
    fn example(cx: &mut Scope, _: &()) -> Element {
        cx.use_cursor(Some(CursorPosition { x: 0, y: 0 }));
        component(six_lines, ())
    }
    let (terminal, mut app, mut screen) = shell_frame(component(example, ()), 40, 10);
    screen.resize(40, 6);
    app.resize(40, 6);
    screen.update(&terminal);
    assert_eq!(
        trimmed(&mut screen),
        [
            "shell 0", "shell 1", "shell 2", "line 0", "line 1", "line 2", "line 3", "line 4",
            "line 5"
        ]
    );
}

#[test]
fn standard_rendering_writes_nothing_when_the_terminal_height_shrinks_and_the_frame_still_fits() {
    let terminal = TestTerminal::new(40, 10);
    let mut app = mount(component(six_lines, ()), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 40, 10);
    assert_eq!(&screen.rows()[..6], LINES);
    assert_eq!(screen.cursor(), (6, 0));
    let before = terminal.writes().len();
    screen.resize(40, 8);
    app.resize(40, 8);
    screen.update(&terminal);
    assert!(terminal.writes()[before..].is_empty());
}

#[test]
fn standard_rendering_leaves_fullscreen_without_clearing_when_the_terminal_height_grows() {
    let terminal = TestTerminal::new(40, 6);
    let mut app = mount(component(six_lines, ()), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 40, 6);
    assert_eq!(screen.rows(), LINES);
    assert_eq!(screen.cursor(), (5, 0));
    screen.resize(40, 10);
    app.resize(40, 10);
    screen.update(&terminal);
    // Ink pins eraseLines(6) + frame + newline. No scrollback-pull model on
    // growth: assert the frame and its rest cursor, not newly exposed rows.
    assert_eq!(&screen.rows()[..6], LINES);
    assert_eq!(screen.cursor(), (6, 0));
    assert!(screen.scrollback().is_empty());
    assert_no_full_clear(&terminal);
}
