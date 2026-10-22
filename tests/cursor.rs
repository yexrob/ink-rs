//! Ported from ink test/cursor.tsx @ 26d2c3f.
//! Cursor suffix and hide/show byte assertions use vt100 position/visibility.
mod common;

use std::cell::Cell;
use std::time::Duration;

use common::{Screen, assert_no_full_clear};
use ink_rs::testing::{TestTerminal, mount, render};
use ink_rs::*;

thread_local! {
    static CURSOR_RENDERS: Cell<usize> = const { Cell::new(0) };
}

fn cursor(cx: &mut Scope, _: &()) -> Element {
    CURSOR_RENDERS.set(CURSOR_RENDERS.get() + 1);
    cx.use_cursor(Some(CursorPosition { x: 2, y: 0 }));
    Text::new("Input").into()
}

fn siblings(_: &mut Scope, status: &&str) -> Element {
    View::new()
        .flex_direction(FlexDirection::Column)
        .child(component(cursor, ()))
        .child(Text::new(*status))
        .into()
}

fn flush(app: &mut testing::TestApp) {
    app.advance(Duration::from_millis(34));
}

#[test]
fn node_cursor_follows_ancestor_movement_in_the_same_paint() {
    fn prompt(cx: &mut Scope, _: &()) -> Element {
        let expanded = cx.use_state(|| false);
        let node = cx.use_node_ref();
        cx.use_input(true, move |_, _| expanded.set(true));
        cx.use_node_cursor(node, Some(CursorPosition { x: 2, y: 2 }));
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Static::new(vec![()], |_, _| Text::new("History").into()))
            .child(Text::new(if expanded.get() { "A\nB" } else { "A" }))
            .child(
                View::new().padding_left(3).child(
                    View::new()
                        .node_ref(node)
                        .border_style(BorderStyle::Single)
                        .padding(1)
                        .child(Text::new("Prompt")),
                ),
            )
            .into()
    }

    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(prompt, ()),
        Options {
            max_fps: 0,
            ..Options::default()
        },
        &terminal,
    );
    assert_cursor(&terminal, 24, 4, 5);
    let before = terminal.writes().len();
    app.input("x");
    let writes = terminal.writes();
    assert_eq!(writes.len(), before + 1);
    assert!(writes[before].contains("\x1b[6G"));
    assert_cursor(&terminal, 24, 5, 5);
    assert!(Screen::new(&terminal, 100, 24).rows()[5].contains("Prompt"));
}

#[test]
fn node_cursor_hides_when_node_unmounts_but_declarer_stays() {
    fn prompt(cx: &mut Scope, _: &()) -> Element {
        let show = cx.use_state(|| true);
        let node = cx.use_node_ref();
        cx.use_input(true, move |_, _| show.set(false));
        cx.use_node_cursor(node, Some(CursorPosition { x: 2, y: 0 }));
        View::new()
            .child(if show.get() {
                Element::from(View::new().node_ref(node).child(Text::new("Prompt")))
            } else {
                Text::new("Waiting").into()
            })
            .into()
    }

    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(prompt, ()),
        Options {
            max_fps: 0,
            ..Options::default()
        },
        &terminal,
    );
    assert_cursor(&terminal, 24, 0, 2);
    let before = terminal.writes().len();
    app.input("x");
    let output = terminal.writes()[before..].join("");
    assert!(output.contains("\x1b[?25l"));
    assert!(!output.contains("\x1b[?25h"));
    let screen = Screen::new(&terminal, 100, 24);
    assert!(!screen.cursor_visible());
    assert_eq!(screen.rows()[0], "Waiting");
}

fn assert_cursor(terminal: &TestTerminal, rows: u16, y: u16, x: u16) {
    let screen = Screen::new(terminal, 100, rows);
    assert_eq!(screen.cursor(), (y, x));
    assert!(screen.cursor_visible());
}

fn input_app(cx: &mut Scope, multiline: &bool) -> Element {
    let text = cx.use_state(String::new);
    cx.use_input(true, move |input, key| {
        if key.backspace || key.delete {
            text.update(|text| {
                text.pop();
            });
        } else if !input.is_empty() && !key.ctrl && !key.meta {
            text.update(|text| text.push_str(input));
        }
    });
    let value = text.get();
    cx.use_cursor(Some(CursorPosition {
        x: 2 + value.len(),
        y: usize::from(*multiline),
    }));
    let mut view = View::new().flex_direction(FlexDirection::Column);
    if *multiline {
        view = view.child(Text::new("Header"));
    }
    view.child(Text::new(format!("> {value}"))).into()
}

#[test]
fn cursor_is_shown_at_specified_position_after_render() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(input_app, false), Options::default(), &terminal);
    app.advance(Duration::from_millis(50));
    assert_cursor(&terminal, 24, 0, 2);
}

#[test]
fn memoized_cursor_survives_sibling_updates_incremental_false() {
    CURSOR_RENDERS.set(0);
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(siblings, "Waiting"),
        Options::default(),
        &terminal,
    );
    assert_cursor(&terminal, 24, 0, 2);
    app.rerender(component(siblings, "Ready"));
    flush(&mut app);
    assert_eq!(CURSOR_RENDERS.get(), 1);
    assert_eq!(
        &Screen::new(&terminal, 100, 24).rows()[..2],
        ["Input", "Ready"]
    );
    assert_cursor(&terminal, 24, 0, 2);

    let before = terminal.writes().len();
    app.rerender(Text::new("Finished"));
    flush(&mut app);
    let output = terminal.writes()[before..].join("");
    assert_eq!(Screen::new(&terminal, 100, 24).rows()[0], "Finished");
    assert!(!Screen::new(&terminal, 100, 24).cursor_visible());
    assert!(output.contains("\x1b[?25l"));
    assert!(!output.contains("\x1b[?25h"));
}

#[test]
fn cursor_is_not_hidden_by_use_effect_after_first_render() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(input_app, false), Options::default(), &terminal);
    app.advance(Duration::from_millis(50));
    assert!(Screen::new(&terminal, 100, 24).cursor_visible());
}

#[test]
fn cursor_follows_text_input() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(input_app, false), Options::default(), &terminal);
    app.advance(Duration::from_millis(50));
    app.input("a");
    app.advance(Duration::from_millis(50));
    assert_cursor(&terminal, 24, 0, 3);
}

#[test]
fn cursor_moves_on_space_input_even_when_output_is_identical() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(input_app, false), Options::default(), &terminal);
    flush(&mut app);
    app.input("a");
    flush(&mut app);
    let after_a = terminal.writes().len();
    app.input(" ");
    flush(&mut app);
    assert!(terminal.writes().len() > after_a);
    assert_cursor(&terminal, 24, 0, 4);
}

#[test]
fn cursor_is_cleared_when_component_using_use_cursor_unmounts() {
    fn child(cx: &mut Scope, _: &()) -> Element {
        cx.use_cursor(Some(CursorPosition { x: 5, y: 0 }));
        Text::new("child").into()
    }
    fn parent(cx: &mut Scope, _: &()) -> Element {
        let show = cx.use_state(|| true);
        cx.use_input(true, move |_, key| {
            if key.enter {
                show.set(false);
            }
        });
        View::new()
            .child(if show.get() {
                component(child, ())
            } else {
                Text::new("no cursor").into()
            })
            .into()
    }
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(parent, ()), Options::default(), &terminal);
    app.advance(Duration::from_millis(50));
    assert!(Screen::new(&terminal, 100, 24).cursor_visible());
    app.input("\r");
    app.advance(Duration::from_millis(50));
    assert!(!Screen::new(&terminal, 100, 24).cursor_visible());
}

#[test]
fn screen_does_not_scroll_up_on_subsequent_renders() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(input_app, true), Options::default(), &terminal);
    app.advance(Duration::from_millis(50));
    let before = terminal.writes().len();
    app.input("x");
    app.advance(Duration::from_millis(50));
    // A final screen cannot show that the cursor was hidden during erasure.
    assert!(terminal.writes()[before..].join("").contains("\x1b[?25l"));
    let mut screen = Screen::new(&terminal, 100, 24);
    assert_eq!(&screen.rows()[..2], ["Header", "> x"]);
    assert!(screen.scrollback().is_empty());
    assert_cursor(&terminal, 24, 1, 3);
}

fn write_with_cursor(cx: &mut Scope, stream: &OutputStream) -> Element {
    cx.use_cursor(Some(CursorPosition { x: 2, y: 0 }));
    let writer = match stream {
        OutputStream::Stdout => cx.use_stdout(),
        OutputStream::Stderr => cx.use_stderr(),
    };
    let stream = *stream;
    cx.use_effect((), move || {
        writer.write(match stream {
            OutputStream::Stdout => "from stdout hook\n",
            OutputStream::Stderr => "from stderr hook\n",
        });
    });
    Text::new("Hello").into()
}

#[test]
fn cursor_remains_visible_after_use_stdout_write() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(write_with_cursor, OutputStream::Stdout),
        Options::default(),
        &terminal,
    );
    app.advance(Duration::from_millis(50));
    assert!(terminal.writes().join("").contains("from stdout hook"));
    assert_cursor(&terminal, 24, 1, 2);
}

#[test]
fn cursor_remains_visible_after_use_stderr_write() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(write_with_cursor, OutputStream::Stderr),
        Options::default(),
        &terminal,
    );
    app.advance(Duration::from_millis(50));
    assert_eq!(terminal.stderr(), ["from stderr hook\n"]);
    assert_cursor(&terminal, 24, 1, 2);
}

fn debug_write(cx: &mut Scope, &(stream, rerender): &(OutputStream, bool)) -> Element {
    let text = cx.use_state(move || if rerender { "Initial" } else { "Hello" });
    let writer = match stream {
        OutputStream::Stdout => cx.use_stdout(),
        OutputStream::Stderr => cx.use_stderr(),
    };
    cx.use_effect((), move || {
        if rerender {
            text.set("Updated");
        }
    });
    let value = text.get();
    cx.use_effect(value, move || {
        if !rerender || value == "Updated" {
            writer.write(match stream {
                OutputStream::Stdout => "from stdout hook\n",
                OutputStream::Stderr => "from stderr hook\n",
            });
        }
    });
    Text::new(value).into()
}

#[test]
fn debug_mode_use_stdout_write_replays_latest_frame() {
    let app = render(component(debug_write, (OutputStream::Stdout, false)));
    assert!(
        app.frames()
            .iter()
            .any(|s| s.contains("from stdout hook\nHello"))
    );
    assert!(!app.frames().iter().any(String::is_empty));
}

#[test]
fn debug_mode_use_stdout_write_does_not_leak_into_stderr() {
    let app = render(component(debug_write, (OutputStream::Stdout, false)));
    assert!(
        app.frames()
            .iter()
            .any(|s| s.contains("from stdout hook\nHello"))
    );
    assert!(app.stderr().is_empty());
}

#[test]
fn debug_mode_use_stderr_write_replays_latest_frame_without_empty_writes() {
    let app = render(component(debug_write, (OutputStream::Stderr, false)));
    assert_eq!(app.stderr(), ["from stderr hook\n"]);
    assert!(app.frames().len() > 1);
    assert!(app.frames()[1..].iter().any(|s| s.contains("Hello")));
    assert!(
        !app.frames()[1..]
            .iter()
            .any(|s| s.contains("from stderr hook\n"))
    );
    assert!(!app.stderr().iter().any(|s| s.contains("Hello")));
    assert!(!app.frames().iter().any(String::is_empty));
    assert!(!app.stderr().iter().any(String::is_empty));
}

#[test]
fn debug_mode_use_stdout_write_replays_rerendered_frame() {
    let mut app = render(component(debug_write, (OutputStream::Stdout, true)));
    app.settle();
    assert!(
        app.frames()
            .iter()
            .any(|s| s.contains("from stdout hook\nUpdated"))
    );
    assert!(
        !app.frames()
            .iter()
            .any(|s| s.contains("from stdout hook\nInitial"))
    );
    assert!(!app.frames().iter().any(String::is_empty));
}

#[test]
fn debug_mode_use_stderr_write_replays_rerendered_frame() {
    let mut app = render(component(debug_write, (OutputStream::Stderr, true)));
    app.settle();
    assert_eq!(app.stderr(), ["from stderr hook\n"]);
    assert!(
        !app.stderr()
            .iter()
            .any(|s| s.contains("Updated") || s.contains("Initial"))
    );
    let later = &app.frames()[1..];
    assert!(later.iter().any(|s| s.contains("Updated")));
    assert!(
        !later
            .iter()
            .any(|s| s.contains("Initial") || s.contains("from stderr hook\n"))
    );
    assert!(!app.frames().iter().any(String::is_empty));
    assert!(!app.stderr().iter().any(String::is_empty));
}

fn fullscreen(cx: &mut Scope, &(lines, y, marker): &(usize, usize, &str)) -> Element {
    cx.use_cursor(Some(CursorPosition { x: 3, y }));
    View::new()
        .flex_direction(FlexDirection::Column)
        .children(
            (0..lines).map(|i| Text::new(format!("Line {i}{}", if i == 1 { marker } else { "" }))),
        )
        .into()
}

#[test]
fn standard_rendering_fullscreen_cursor_lands_on_the_requested_row_across_rerender_and_cursor_only_update()
 {
    let terminal = TestTerminal::new(100, 5);
    let mut app = mount(
        component(fullscreen, (5, 2, "")),
        Options::default(),
        &terminal,
    );
    assert_cursor(&terminal, 5, 2, 3);
    app.rerender(component(fullscreen, (5, 2, "!")));
    flush(&mut app);
    assert_eq!(
        Screen::new(&terminal, 100, 5).rows(),
        ["Line 0", "Line 1!", "Line 2", "Line 3", "Line 4"]
    );
    assert_cursor(&terminal, 5, 2, 3);
    let before = terminal.writes().len();
    app.rerender(component(fullscreen, (5, 0, "!")));
    flush(&mut app);
    assert_cursor(&terminal, 5, 0, 3);
    assert!(!terminal.writes()[before..].join("").contains("Line"));
    assert!(Screen::new(&terminal, 100, 5).scrollback().is_empty());
}

#[test]
fn standard_rendering_fullscreen_cursor_lands_on_the_requested_row_on_the_sync_path() {
    let terminal = TestTerminal::new(100, 5);
    let mut app = mount(
        component(fullscreen, (6, 2, "")),
        Options::default(),
        &terminal,
    );
    app.rerender(component(fullscreen, (6, 2, "!")));
    flush(&mut app);
    // design.md caps the frame at screen height: Line 0 is cut, so live y=2
    // lands at viewport y=1. Ink's sync path clears and rewrites the overflow.
    assert_no_full_clear(&terminal);
    assert_cursor(&terminal, 5, 1, 3);
    assert_eq!(
        Screen::new(&terminal, 100, 5).rows(),
        ["Line 1!", "Line 2", "Line 3", "Line 4", "Line 5"]
    );
    assert!(Screen::new(&terminal, 100, 5).scrollback().is_empty());
}
