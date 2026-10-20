//! Ported from Ink test/render.tsx and test/alternate-screen-example.tsx
//! (Ink 8.0.0, commit 26d2c3f).

mod common;

#[path = "../examples/alternate-screen/app.rs"]
mod alternate_screen;

#[path = "../examples/aria/app.rs"]
mod aria;

#[path = "../examples/chat/app.rs"]
mod chat;

#[path = "../examples/cursor-ime/app.rs"]
mod cursor_ime;

#[path = "../examples/incremental-rendering/app.rs"]
mod incremental_rendering;

#[path = "../examples/scroll/app.rs"]
mod scroll;

#[cfg(feature = "tokio")]
#[path = "../examples/subprocess-output/app.rs"]
mod subprocess_output;

#[path = "../examples/table/app.rs"]
mod table;

#[path = "../examples/use-input/app.rs"]
mod use_input;

#[path = "../examples/use-stdout/app.rs"]
mod use_stdout;

use std::time::Duration;

use ink_rs::testing::{TestApp, TestTerminal, mount};
use ink_rs::{ColorLevel, Element, Options, RenderOptions, render_to_string, text_width};

use common::Screen;

fn flush(app: &mut TestApp) {
    app.advance(Duration::from_millis(34));
}

fn start(element: Element, rows: u16) -> (TestApp, TestTerminal, Screen) {
    let terminal = TestTerminal::new(100, rows);
    let app = mount(
        element,
        Options {
            interactive: Some(true),
            ..Options::default()
        },
        &terminal,
    );
    let screen = Screen::new(&terminal, 100, rows);
    (app, terminal, screen)
}

fn input(app: &mut TestApp, terminal: &TestTerminal, screen: &mut Screen, bytes: impl AsRef<[u8]>) {
    app.input(bytes);
    flush(app);
    screen.update(terminal);
}

fn contains(screen: &Screen, text: &str) -> bool {
    screen.rows().iter().any(|row| row.contains(text))
}

fn has_row(screen: &Screen, text: &str) -> bool {
    screen.rows().iter().any(|row| row.trim() == text)
}

#[cfg(feature = "tokio")]
async fn subprocess_lines(
    chunks: impl IntoIterator<Item = subprocess_output::Chunk>,
) -> Vec<String> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        subprocess_output::app(receiver),
        Options {
            interactive: Some(false),
            ..Options::default()
        },
        &terminal,
    );
    for chunk in chunks {
        sender.send(chunk).unwrap();
        tokio::task::yield_now().await;
        app.advance(Duration::ZERO);
    }
    drop(sender);
    tokio::task::yield_now().await;
    app.advance(Duration::ZERO);
    app.unmount();
    subprocess_output::strip_ansi(&terminal.writes().concat())
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

// render.tsx:355. The channel replaces the fixture's stubbed stdout stream.
#[cfg(feature = "tokio")]
#[tokio::test(flavor = "local", start_paused = true)]
async fn subprocess_example_retains_recent_lines_across_chunks() {
    let chunks = ["one\ntwo\nthree\nfour\nfi", "ve\nsix"]
        .map(|text| subprocess_output::Chunk::Bytes(text.as_bytes().to_vec()));
    assert_eq!(
        subprocess_lines(chunks).await,
        ["Command output:", "two", "three", "four", "five", "six"]
    );
}

// render.tsx:482.
#[cfg(feature = "tokio")]
#[tokio::test(flavor = "local", start_paused = true)]
async fn subprocess_example_strips_ansi_sequences_across_chunks() {
    let chunks = ["\x1b[3", "1mRed\x1b[0", "m text"]
        .map(|text| subprocess_output::Chunk::Bytes(text.as_bytes().to_vec()));
    assert_eq!(
        subprocess_lines(chunks).await,
        ["Command output:", "Red text"]
    );
}

// render.tsx:508.
#[cfg(feature = "tokio")]
#[tokio::test(flavor = "local", start_paused = true)]
async fn subprocess_example_displays_command_launch_errors() {
    let chunks = [subprocess_output::Chunk::Failed("spawn npm ENOENT".into())];
    assert_eq!(
        subprocess_lines(chunks).await,
        ["Command output:", "spawn npm ENOENT"]
    );
}

// render.tsx:523.
#[cfg(feature = "tokio")]
#[tokio::test(flavor = "local", start_paused = true)]
async fn subprocess_example_decodes_utf8_across_chunks() {
    let chunks = "\u{d55c}\u{1f642}"
        .bytes()
        .map(|byte| subprocess_output::Chunk::Bytes(vec![byte]));
    assert_eq!(
        subprocess_lines(chunks).await,
        ["Command output:", "\u{d55c}\u{1f642}"]
    );
}

// render.tsx:271. Screen rows/cursor replace eraseLines and cursorTo byte checks.
#[test]
fn ime_example_deletes_whole_graphemes_with_backspace() {
    let (mut app, terminal, mut screen) = start(cursor_ime::app(), 24);
    input(&mut app, &terminal, &mut screen, "A");
    for grapheme in ["\u{1f600}", "\u{1f469}\u{200d}\u{1f4bb}", "e\u{301}"] {
        input(&mut app, &terminal, &mut screen, grapheme);
        input(&mut app, &terminal, &mut screen, b"\x7f");
        assert_eq!(screen.rows()[1], "> A");
        assert!(!contains(&screen, "\u{fffd}"));
        assert_eq!(screen.cursor(), (1, 3));
        assert!(screen.cursor_visible());
    }
}

// render.tsx:299. Read the resulting row and cursor, not redraw escape sequences.
#[test]
fn ime_example_ignores_enter_when_editing_text() {
    let (mut app, terminal, mut screen) = start(cursor_ime::app(), 24);
    input(&mut app, &terminal, &mut screen, "\u{d55c}\u{ae00}");
    input(&mut app, &terminal, &mut screen, b"\r\x1b[127u");
    assert_eq!(screen.rows()[1], "> \u{d55c}");
    assert!(!contains(&screen, "> \u{d55c}\u{ae00}"));
    assert_eq!(screen.cursor(), (1, 4));
}

// render.tsx:320. Cursor coordinates/visibility replace cursorUp/cursorTo bytes.
#[test]
fn ime_example_places_the_cursor_after_wrapped_input() {
    let (mut app, terminal, mut screen) = start(cursor_ime::app(), 24);
    input(&mut app, &terminal, &mut screen, "\u{d55c}".repeat(52));
    assert_eq!(screen.rows()[2], "\u{d55c}".repeat(3));
    assert_eq!(screen.cursor(), (2, 6));
    input(&mut app, &terminal, &mut screen, b"\x7f\x7f\x7f");
    assert_eq!(screen.rows()[1], format!("> {}", "\u{d55c}".repeat(49)));
    assert_eq!(screen.cursor(), (2, 0));
    assert!(screen.cursor_visible());
}

// render.tsx:344. Pure rendering replaces a CI subprocess; column width replaces JS length.
#[test]
fn table_example_keeps_all_columns_visible_in_narrow_terminals() {
    let output = render_to_string(
        table::app(),
        RenderOptions {
            columns: 40,
            color_level: ColorLevel::None,
        },
    );
    let header = output.lines().next().unwrap();
    assert_eq!(
        header.split_whitespace().collect::<Vec<_>>(),
        ["ID", "Name", "Email"]
    );
    assert!(text_width(header) <= 40);
}

// render.tsx:392. Resize and screen content replace PTY output polling.
#[test]
fn stdout_example_updates_displayed_dimensions_after_resize() {
    let (mut app, terminal, mut screen) = start(use_stdout::app(), 24);
    assert!(contains(&screen, "Width: 100"));
    assert!(contains(&screen, "Height: 24"));
    screen.resize(60, 20);
    app.resize(60, 20);
    flush(&mut app);
    screen.update(&terminal);
    assert!(contains(&screen, "Width: 60"));
    assert!(contains(&screen, "Height: 20"));
}

fn toggles(count: usize) {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        aria::app(),
        Options {
            interactive: Some(false),
            ..Options::default()
        },
        &terminal,
    );
    // The fixture supplies all encoded spaces in one stdin read; CI emits at unmount.
    app.input("\x1b[32u".repeat(count));
    flush(&mut app);
    app.unmount();
    let screen = Screen::new(&terminal, 100, 24);
    assert!(contains(
        &screen,
        if count.is_multiple_of(2) {
            "[ ]"
        } else {
            "[x]"
        }
    ));
}

// render.tsx:497 (count 1).
#[test]
fn aria_example_applies_1_consecutive_checkbox_toggles() {
    toggles(1);
}

// render.tsx:497 (count 2).
#[test]
fn aria_example_applies_2_consecutive_checkbox_toggles() {
    toggles(2);
}

// render.tsx:497 (count 3).
#[test]
fn aria_example_applies_3_consecutive_checkbox_toggles() {
    toggles(3);
}

// render.tsx:538. A trimmed screen row replaces the output regex and CRLF matching.
#[test]
fn chat_example_submits_intact_text_after_deleting_an_emoji() {
    let (mut app, terminal, mut screen) = start(chat::app(), 24);
    input(
        &mut app,
        &terminal,
        &mut screen,
        "Hi \u{1f469}\u{200d}\u{1f4bb}",
    );
    input(&mut app, &terminal, &mut screen, b"\x7f");
    assert!(has_row(&screen, "Enter your message: Hi"));
    input(&mut app, &terminal, &mut screen, b"\r");
    assert!(has_row(&screen, "User: Hi"));
}

// render.tsx:561. Inspect the screen rather than a redraw chunk.
#[test]
fn chat_example_submits_pending_edits_from_the_same_input_chunk() {
    let (mut app, terminal, mut screen) = start(chat::app(), 24);
    input(&mut app, &terminal, &mut screen, "Hello");
    input(&mut app, &terminal, &mut screen, b"\x7f\r");
    assert!(has_row(&screen, "User: Hell"));
    input(&mut app, &terminal, &mut screen, "A");
    input(&mut app, &terminal, &mut screen, b"\x7f\r");
    assert!(!contains(&screen, "User: A"));
    assert!(has_row(&screen, "Enter your message:"));
}

// render.tsx:586. A screen row replaces the output regex.
#[test]
fn chat_example_does_not_insert_modified_shortcuts_into_messages() {
    let (mut app, terminal, mut screen) = start(chat::app(), 24);
    input(&mut app, &terminal, &mut screen, "Hello");
    input(&mut app, &terminal, &mut screen, b"\x01\x1bb!");
    input(&mut app, &terminal, &mut screen, b"\r");
    assert!(has_row(&screen, "User: Hello!"));
}

// render.tsx:606. Screen substrings replace regexes over redraw output.
#[test]
fn scroll_example_handles_consecutive_arrow_keys_in_one_input_chunk() {
    let (mut app, terminal, mut screen) = start(scroll::app(), 24);
    assert!(contains(&screen, "scrollTop=0/32"));
    for (keys, expected) in [
        ("\x1b[B\x1b[B", "scrollTop=2/32"),
        ("\x1b[C\x1b[C", "scrollLeft=4/42"),
        ("\x1b[A\x1b[A", "scrollTop=0/32"),
        ("\x1b[D\x1b[D", "scrollLeft=0/42"),
    ] {
        input(&mut app, &terminal, &mut screen, keys);
        assert!(contains(&screen, expected), "{:?}", screen.rows());
    }
}

// render.tsx:638. Physical rows replace line indices within Ink's redraw chunks.
#[test]
fn input_example_handles_consecutive_movement_keys() {
    let (mut app, terminal, mut screen) = start(use_input::app(), 24);
    input(
        &mut app,
        &terminal,
        &mut screen,
        b"\x1b[C\x1b[C\x1b[B\x1b[B",
    );
    assert_eq!(screen.rows()[4], "   ^_^");
    input(
        &mut app,
        &terminal,
        &mut screen,
        b"\x1b[D\x1b[D\x1b[A\x1b[A",
    );
    assert_eq!(screen.rows()[2], " ^_^");
}

// render.tsx:672. Screen content replaces output polling after PTY resize.
#[test]
fn incremental_example_keeps_its_selection_visible_after_resize() {
    let (mut app, terminal, mut screen) = start(incremental_rendering::app(), 80);
    assert!(contains(&screen, "Selected: Server Authentication"));
    input(&mut app, &terminal, &mut screen, b"\x1b[A");
    assert!(contains(&screen, "Selected: Recommendation Engine"));
    screen.resize(100, 30);
    app.resize(100, 30);
    flush(&mut app);
    screen.update(&terminal);
    assert!(
        contains(&screen, "System Services Monitor (10 of 30 services)"),
        "{:?}",
        screen.rows()
    );
    assert!(contains(&screen, "Selected: WebSocket Connection Manager"));
    input(&mut app, &terminal, &mut screen, b"\x1b[B");
    assert!(contains(&screen, "Selected: Server Authentication"));
    assert!(contains(&screen, "> Server Authentication"));
}

// render.tsx:703. Screen content replaces output polling after PTY resize.
#[test]
fn incremental_example_grows_its_logs_after_terminal_resize() {
    let (mut app, terminal, mut screen) = start(incremental_rendering::app(), 30);
    assert!(contains(&screen, "Worker-3"));
    assert!(!contains(&screen, "Worker-12"));
    screen.resize(100, 60);
    app.resize(100, 60);
    flush(&mut app);
    screen.update(&terminal);
    assert!(contains(
        &screen,
        "System Services Monitor (30 of 30 services)"
    ));
    assert!(contains(&screen, "Worker-12"));
}

// render.tsx:727. Fake-clock tick and screen state replace waiting for a Score redraw.
#[test]
fn snake_ignores_a_reversal_queued_between_ticks() {
    let terminal = TestTerminal::new(100, 30);
    let mut app = mount(
        alternate_screen::app(),
        Options {
            alternate_screen: true,
            interactive: Some(true),
            ..Options::default()
        },
        &terminal,
    );
    let mut screen = Screen::new(&terminal, 100, 30);
    assert!(contains(&screen, "Arrow keys: move"));
    app.input(b"\x1b[A\x1b[D");
    app.advance(Duration::from_millis(150));
    flush(&mut app);
    screen.update(&terminal);
    assert!(!contains(&screen, "Game Over!"));
    assert!(contains(&screen, "Arrow keys: move"));
}

// alternate-screen-example.tsx:9.
#[test]
fn snake_can_move_into_the_tail_cell_when_the_tail_moves_away() {
    use alternate_screen::{Action, Direction, Game, Point, reduce};
    let mut game = Game {
        snake: vec![
            Point { x: 2, y: 1 },
            Point { x: 1, y: 1 },
            Point { x: 1, y: 2 },
            Point { x: 2, y: 2 },
        ],
        food: Point { x: 0, y: 0 },
        score: 3,
        game_over: false,
        won: false,
        frame: 10,
    };
    reduce(&mut game, Action::Tick(Direction::Down));
    assert!(!game.game_over);
    assert_eq!(
        game.snake,
        [
            Point { x: 2, y: 2 },
            Point { x: 2, y: 1 },
            Point { x: 1, y: 1 },
            Point { x: 1, y: 2 }
        ]
    );
    assert_eq!(game.score, 3);
}

// alternate-screen-example.tsx:39. A reducer call replaces subprocess JSON and its
// one-second kill timer; the test runner supervises completion of the win case.
#[test]
fn snake_ends_with_a_win_when_it_fills_the_board() {
    use alternate_screen::{Action, BOARD_HEIGHT, BOARD_WIDTH, Direction, Game, Point, reduce};
    let mut snake = Vec::new();
    for y in 0..BOARD_HEIGHT {
        for column in 0..BOARD_WIDTH {
            let x = if y % 2 == 0 {
                column
            } else {
                BOARD_WIDTH - 1 - column
            };
            if x != 0 || y != 0 {
                snake.push(Point { x, y });
            }
        }
    }
    let mut game = Game {
        score: snake.len() - 3,
        snake,
        food: Point { x: 0, y: 0 },
        game_over: false,
        won: false,
        frame: 42,
    };
    reduce(&mut game, Action::Tick(Direction::Left));
    assert!(game.game_over);
    assert!(game.won);
    assert_eq!(game.score, 297);
    assert_eq!(game.snake.len(), 300);
}
