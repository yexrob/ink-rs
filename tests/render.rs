//! Ported from ink test/render.tsx @ 26d2c3f.
mod common;

use common::{Screen, assert_no_full_clear};
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;
use std::cell::Cell;
use std::time::Duration;

fn column(lines: impl IntoIterator<Item = String>) -> Element {
    View::new()
        .flex_direction(FlexDirection::Column)
        .children(lines.into_iter().map(Text::new))
        .into()
}

fn history(items: Vec<&'static str>, live: usize, tick: u32) -> Element {
    Element::fragment::<[Element; 2]>([
        Static::new(items, |item, _| Text::new(*item).key(*item)).into(),
        column((0..live).map(|i| format!("live {i} tick {tick}"))),
    ])
}

fn flush(app: &mut testing::TestApp) {
    app.advance(Duration::from_millis(34));
}

fn nonempty(terminal: &TestTerminal, rows: u16) -> Vec<String> {
    Screen::new(terminal, 100, rows)
        .lines()
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

thread_local! { static PHASE: Cell<Option<State<u32>>> = const { Cell::new(None) }; }

fn erase(cx: &mut Scope, _: &()) -> Element {
    let phase = cx.use_state(|| 0);
    PHASE.set(Some(phase));
    column(if phase.get() == 0 {
        vec!["A".into(), "B".into(), "C".into()]
    } else {
        vec![]
    })
}

#[test]
fn do_not_erase_screen() {
    let terminal = TestTerminal::new(100, 4);
    let _app = mount(component(erase, ()), Options::default(), &terminal);
    assert_no_full_clear(&terminal);
    assert_eq!(nonempty(&terminal, 4), ["A", "B", "C"]);
}

#[test]
fn do_not_erase_screen_where_static_is_taller_than_viewport() {
    // Ink's fixture reads a missing argv[3]; use its unknown-size fallback.
    let terminal = TestTerminal::new(100, 24);
    let _app = mount(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["A", "B", "C"], |item, _| Text::new(*item).into()).into(),
            column(["D", "E", "F"].map(str::to_owned)),
        ]),
        Options::default(),
        &terminal,
    );
    assert_no_full_clear(&terminal);
    assert_eq!(nonempty(&terminal, 24), ["A", "B", "C", "D", "E", "F"]);
}

fn accounting(cx: &mut Scope, _: &()) -> Element {
    let phase = cx.use_state(|| 0);
    PHASE.set(Some(phase));
    let count = if phase.get() == 1 { 5 } else { 1 };
    let label = if phase.get() == 3 { "LIVE" } else { "live" };
    Element::fragment::<[Element; 2]>([
        Static::new(vec!["A", "B", "C", "D", "E", "F"], |item, _| {
            Text::new(*item).into()
        })
        .into(),
        column((0..count).map(|i| format!("{label}-{i}"))),
    ])
}

#[test]
fn last_line_of_static_survives_a_full_clear_accounting_frame_related_to_issue_973() {
    let terminal = TestTerminal::new(100, 4);
    let mut app = mount(component(accounting, ()), Options::default(), &terminal);
    for phase in 1..=3 {
        PHASE.get().unwrap().set(phase);
        flush(&mut app);
    }
    let lines = nonempty(&terminal, 4);
    assert!(lines.contains(&"F".into()));
    assert!(lines.contains(&"LIVE-0".into()));
}

#[test]
fn erase_screen_where_state_changes() {
    let terminal = TestTerminal::new(100, 4);
    let mut app = mount(component(erase, ()), Options::default(), &terminal);
    PHASE.get().unwrap().set(1);
    flush(&mut app);
    assert!(nonempty(&terminal, 4).is_empty());
}

fn fullscreen(top: &str, bottom: &str) -> Element {
    View::new()
        .width(100)
        .height(5)
        .flex_direction(FlexDirection::Column)
        .child(View::new().flex_grow(1.0).child(Text::new(top)))
        .child(Text::new(bottom))
        .into()
}

#[test]
fn fullscreen_mode_should_not_add_extra_newline_at_the_bottom() {
    let terminal = TestTerminal::new(100, 5);
    let _app = mount(
        fullscreen("Full-screen: top", "Bottom line (should be usable)"),
        Options::default(),
        &terminal,
    );
    let mut screen = Screen::new(&terminal, 100, 5);
    assert_eq!(screen.rows()[4], "Bottom line (should be usable)");
    assert!(screen.scrollback().is_empty());
}

#[test]
fn issue_442_full_terminal_size_box_should_not_add_an_extra_scroll_line() {
    let terminal = TestTerminal::new(100, 5);
    let _app = mount(
        fullscreen("#442 top", "#442 bottom"),
        Options::default(),
        &terminal,
    );
    let mut screen = Screen::new(&terminal, 100, 5);
    assert_eq!(screen.rows()[4], "#442 bottom");
    assert!(screen.scrollback().is_empty());
}

#[test]
fn issue_450_initial_overflowing_frame_should_not_clear_terminal() {
    let terminal = TestTerminal::new(100, 3);
    let _app = mount(
        column((1..=4).map(|n| format!("#450 overflow line {n}"))),
        Options::default(),
        &terminal,
    );
    assert_no_full_clear(&terminal);
}

#[test]
fn issue_450_initial_full_height_frame_should_not_clear_terminal() {
    let terminal = TestTerminal::new(100, 3);
    let _app = mount(
        column((1..=3).map(|n| format!("#450 fullscreen line {n}"))),
        Options::default(),
        &terminal,
    );
    assert_no_full_clear(&terminal);
}

fn fixture(cx: &mut Scope, mode: &u32) -> Element {
    let frame = cx.use_state(|| 0);
    PHASE.set(Some(frame));
    let height = match mode {
        0 => 5,
        1 => 6,
        2 => {
            if frame.get() == 0 {
                5
            } else {
                6
            }
        }
        _ => 7,
    };
    let label = if *mode == 3 && frame.get() == 7 {
        "frame 7 STALE-ROW".into()
    } else {
        format!("frame {}", frame.get())
    };
    View::new()
        .height(height)
        .flex_direction(FlexDirection::Column)
        .child(Text::new("#450 top"))
        .child(View::new().flex_grow(1.0).child(Text::new(label)))
        .child(Text::new("#450 bottom"))
        .into()
}

fn run_fixture(terminal: &TestTerminal, mode: u32) -> testing::TestApp {
    let mut app = mount(component(fixture, mode), Options::default(), terminal);
    for frame in 1..=8 {
        PHASE.get().unwrap().set(frame);
        app.advance(Duration::from_millis(100));
    }
    app
}

#[test]
fn issue_450_control_rows_minus_1_rerenders_should_avoid_a_full_clear() {
    let terminal = TestTerminal::new(100, 6);
    let _app = run_fixture(&terminal, 0);
    assert_no_full_clear(&terminal);
}

#[test]
fn issue_935_overflowing_rerenders_must_not_erase_terminal_scrollback() {
    let terminal = TestTerminal::new(100, 6);
    for i in 0..6 {
        terminal.write(&format!("#935 scrollback {i}\r\n"));
    }
    let _app = run_fixture(&terminal, 3);
    let mut screen = Screen::new(&terminal, 100, 6);
    let history = screen.scrollback();
    for i in 0..6 {
        assert!(history.contains(&format!("#935 scrollback {i}")));
    }
    assert!(screen.rows().iter().any(|s| s.contains("frame 8")));
    assert!(screen.rows().iter().all(|s| !s.contains("STALE-ROW")));
    assert_no_full_clear(&terminal);
}

#[test]
fn issue_450_full_height_rerenders_should_not_clear_before_unmount() {
    let terminal = TestTerminal::new(100, 6);
    let _app = run_fixture(&terminal, 1);
    assert_no_full_clear(&terminal);
}

#[test]
fn issue_450_grow_from_rows_minus_1_to_full_height_should_not_clear_before_unmount() {
    let terminal = TestTerminal::new(100, 6);
    let _app = run_fixture(&terminal, 2);
    assert_no_full_clear(&terminal);
}

#[test]
fn issue_450_non_tty_full_height_rerenders_should_never_clear_terminal() {
    // Ink uses createStdout's TTY default despite this test's name.
    let terminal = TestTerminal::new(100, 6);
    let view = |frame| {
        View::new()
            .height(6)
            .child(Text::new(format!("frame {frame}")))
    };
    let mut app = mount(view(0), Options::default(), &terminal);
    app.rerender(view(1));
    app.rerender(view(2));
    assert_no_full_clear(&terminal);
}

#[test]
fn issue_450_non_tty_overflow_transitions_should_never_clear_terminal() {
    let terminal = TestTerminal::new(100, 3);
    let mut app = mount(
        column(["A", "B"].map(str::to_owned)),
        Options::default(),
        &terminal,
    );
    app.rerender(column(["A", "B", "C", "D"].map(str::to_owned)));
    assert_no_full_clear(&terminal);
}

#[test]
fn issue_450_viewport_shrink_into_overflow_should_clear_once() {
    let terminal = TestTerminal::new(100, 6);
    let mut app = mount(
        View::new()
            .height(6)
            .flex_direction(FlexDirection::Column)
            .child(Text::new("#450 top"))
            .child(View::new().flex_grow(1.0).child(Text::new("#450 middle")))
            .child(Text::new("#450 bottom")),
        Options::default(),
        &terminal,
    );
    let mut screen = Screen::new(&terminal, 100, 6);
    let before = terminal.writes().len();
    screen.resize(100, 5);
    app.resize(100, 5);
    app.advance(Duration::ZERO);
    screen.update(&terminal);
    let bytes = terminal.writes()[before..].join("");
    assert_eq!(bytes.matches("\x1b[1;1H\x1b[2K\x1b[C\x1b[J").count(), 1);
    assert!(!bytes.contains("\x1b[2J"));
    assert!(!bytes.contains("\x1b[3J"));
}

#[test]
fn issue_450_non_tty_grow_to_overflow_rerender_should_not_clear_terminal() {
    let terminal = TestTerminal::new(100, 6).with_tty(false);
    let mut app = mount(component(fixture, 0), Options::default(), &terminal);
    app.rerender(component(fixture, 3));
    flush(&mut app);
    app.unmount();
    assert_no_full_clear(&terminal);
}

#[test]
fn clear_output() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(component(erase, ()), Options::default(), &terminal);
    app.clear();
    app.unmount();
    assert!(nonempty(&terminal, 24).is_empty());
}

#[test]
fn preserve_terminal_history_when_rerendering_after_clear_standard() {
    let terminal = TestTerminal::new(100, 24);
    terminal.write("History\r\n");
    let mut app = mount(Text::new("First\nSecond"), Options::default(), &terminal);
    for _ in 0..2 {
        app.clear();
        assert_eq!(nonempty(&terminal, 24), ["History"]);
    }
    app.rerender(Text::new("Updated"));
    flush(&mut app);
    assert_eq!(nonempty(&terminal, 24), ["History", "Updated"]);
}

#[test]
fn full_clears_do_not_replay_static_output_into_scrollback_standard() {
    let terminal = TestTerminal::new(100, 4);
    terminal.write("History\r\n");
    let items = vec!["S1", "S2", "S3"];
    let mut app = mount(history(items.clone(), 6, 0), Options::default(), &terminal);
    for (live, tick) in [(6, 1), (6, 2), (1, 3)] {
        app.rerender(history(items.clone(), live, tick));
        flush(&mut app);
    }
    let mut screen = Screen::new(&terminal, 100, 4);
    assert_eq!(&screen.scrollback()[..4], ["History", "S1", "S2", "S3"]);
    for item in items {
        assert_eq!(
            screen.lines().iter().filter(|line| *line == item).count(),
            1
        );
    }
    assert_eq!(
        screen
            .rows()
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>(),
        ["live 0 tick 3"]
    );
}

#[test]
fn growing_into_an_overflowing_frame_keeps_static_rows_that_are_still_in_the_viewport_standard() {
    let terminal = TestTerminal::new(100, 3);
    terminal.write("History\r\n");
    let mut app = mount(
        history(vec!["S1", "S2", "S3"], 1, 0),
        Options::default(),
        &terminal,
    );
    app.rerender(history(vec!["S1", "S2", "S3"], 5, 1));
    flush(&mut app);
    assert_no_full_clear(&terminal);
    // design.md caps live rows: live 0 and live 1 never enter scrollback.
    assert_eq!(
        nonempty(&terminal, 3),
        [
            "History",
            "S1",
            "S2",
            "S3",
            "live 2 tick 1",
            "live 3 tick 1",
            "live 4 tick 1"
        ]
    );
}

#[test]
fn static_items_added_while_the_frame_overflows_are_written_once_standard() {
    let terminal = TestTerminal::new(100, 4);
    let mut app = mount(history(vec!["S1"], 6, 0), Options::default(), &terminal);
    for n in 2..=3 {
        app.rerender(history(
            vec!["S1", "S2", "S3"][..n].to_vec(),
            6,
            n as u32 - 1,
        ));
        flush(&mut app);
    }
    assert_eq!(
        nonempty(&terminal, 4)
            .into_iter()
            .filter(|s| s.starts_with('S'))
            .collect::<Vec<_>>(),
        ["S1", "S2", "S3"]
    );
}

#[test]
fn alternate_screen_replays_static_output_when_the_frame_shrinks_after_overflowing_standard() {
    let terminal = TestTerminal::new(100, 5);
    let mut app = mount(
        history(vec!["S1", "S2", "S3"], 1, 0),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    );
    for (live, tick) in [(7, 1), (1, 2)] {
        app.rerender(history(vec!["S1", "S2", "S3"], live, tick));
        flush(&mut app);
    }
    assert_eq!(
        Screen::new(&terminal, 100, 5).rows(),
        ["S1", "S2", "S3", "live 0 tick 2", ""]
    );
}

#[test]
fn alternate_screen_replays_static_items_added_while_the_frame_overflows_once_standard() {
    let terminal = TestTerminal::new(100, 5);
    let mut app = mount(
        history(vec!["S1"], 7, 0),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    );
    for n in 2..=3 {
        app.rerender(history(
            vec!["S1", "S2", "S3"][..n].to_vec(),
            7,
            n as u32 - 1,
        ));
        flush(&mut app);
        // The height cap hides all Static rows while live output fills the screen.
        assert!(
            Screen::new(&terminal, 100, 5)
                .rows()
                .iter()
                .all(|s| !s.starts_with('S'))
        );
    }
    app.rerender(history(vec!["S1", "S2", "S3"], 1, 3));
    flush(&mut app);
    assert_eq!(
        Screen::new(&terminal, 100, 5).rows(),
        ["S1", "S2", "S3", "live 0 tick 3", ""]
    );
}

#[test]
fn update_trailing_newline_when_unchanged_output_becomes_fullscreen() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(Text::new("First\nSecond"), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 100, 24);
    assert_eq!(&screen.rows()[..3], ["First", "Second", ""]);
    assert_eq!(screen.cursor(), (2, 0));
    screen.resize(100, 2);
    app.resize(100, 2);
    screen.update(&terminal);
    assert_eq!(screen.rows(), ["First", "Second"]);
    assert_eq!(screen.scrollback(), ["First"]);
    assert_eq!(screen.cursor(), (1, 0));
}

#[test]
fn rerender_on_resize() {
    fn example(_: &mut Scope, _: &()) -> Element {
        View::new()
            .border_style(BorderStyle::Round)
            .child(Text::new("Test"))
            .into()
    }
    let terminal = TestTerminal::new(10, 24);
    let mut app = mount(component(example, ()), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 10, 24);
    assert_eq!(
        &screen.rows()[..4],
        ["╭────────╮", "│Test    │", "╰────────╯", ""]
    );
    screen.resize(8, 24);
    app.resize(8, 24);
    app.advance(Duration::from_millis(100));
    screen.update(&terminal);
    assert_eq!(
        &screen.rows()[..4],
        ["╭──────╮", "│Test  │", "╰──────╯", ""]
    );
    // Ink counts EventEmitter listeners; the driver polls events and unmount
    // consumes TestApp, so its listener-count assertions have no counterpart.
    app.unmount();
}

#[test]
fn throttle_renders_to_max_fps() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        Text::new("Hello"),
        Options {
            max_fps: 1,
            ..Options::default()
        },
        &terminal,
    );
    assert_eq!(terminal.writes().len(), 1);
    assert_eq!(nonempty(&terminal, 24), ["Hello"]);
    app.rerender(Text::new("World"));
    assert_eq!(terminal.writes().len(), 1);
    app.advance(Duration::from_millis(999));
    assert_eq!(terminal.writes().len(), 1);
    app.advance(Duration::from_millis(1));
    assert_eq!(terminal.writes().len(), 2);
    assert_eq!(nonempty(&terminal, 24), ["World"]);
}

#[test]
fn no_throttled_renders_after_unmount() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(Text::new("Foo"), Options::default(), &terminal);
    assert_eq!(terminal.writes().len(), 1);
    app.rerender(Text::new("Bar"));
    app.rerender(Text::new("Baz"));
    app.unmount();
    // unmount consumes TestApp; Ink's clock.tick after unmount has no counterpart.
    assert_eq!(nonempty(&terminal, 24), ["Baz"]);
}

#[test]
fn unmount_forces_pending_throttled_render() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        Text::new("Hello"),
        Options {
            max_fps: 1,
            ..Options::default()
        },
        &terminal,
    );
    assert_eq!(terminal.writes().len(), 1);
    app.rerender(Text::new("Final"));
    assert_eq!(terminal.writes().len(), 1);
    app.unmount();
    assert_eq!(nonempty(&terminal, 24), ["Final"]);
}

#[test]
fn wait_until_exit_resolves_first_exit_value_when_duplicate_exits_happen_during_teardown() {
    thread_local! {
        static HANDLE: Cell<Option<AppHandle>> = const { Cell::new(None) };
    }
    struct Cleanup(AppHandle);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            self.0.exit_with("second");
        }
    }
    fn example(cx: &mut Scope, _: &()) -> Element {
        let handle = cx.use_app();
        HANDLE.set(Some(handle.clone()));
        cx.use_effect((), move || {
            handle.exit_with("first");
            Cleanup(handle)
        });
        Text::new("Hello").into()
    }
    let terminal = TestTerminal::new(100, 24);
    let app = mount(component(example, ()), Options::default(), &terminal);
    // Rust writes have no Node callback barrier. Exercise a cleanup request
    // during teardown and an outside request after the exit has completed.
    HANDLE.take().unwrap().exit_with("second");
    let result = app.exit_result().unwrap().as_ref().unwrap();
    assert_eq!(
        result.as_ref().unwrap().downcast_ref::<&str>(),
        Some(&"first")
    );
}

#[test]
fn no_bsu_esu_when_output_is_unchanged() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        Text::new("Hello"),
        Options {
            max_fps: 1,
            ..Options::default()
        },
        &terminal,
    );
    assert!(terminal.writes()[0].starts_with("\x1b[?2026h"));
    app.rerender(Text::new("Hello"));
    app.advance(Duration::from_secs(1));
    assert_eq!(terminal.writes().len(), 1);
}

#[test]
fn no_bsu_esu_when_output_and_cursor_are_unchanged() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        cx.use_cursor(Some(CursorPosition { x: 0, y: 0 }));
        Text::new("Hello").into()
    }
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(example, ()),
        Options {
            max_fps: 1,
            ..Options::default()
        },
        &terminal,
    );
    assert!(terminal.writes()[0].starts_with("\x1b[?2026h"));
    app.rerender(component(example, ()));
    app.advance(Duration::from_secs(1));
    assert_eq!(terminal.writes().len(), 1);
}

#[test]
fn bsu_esu_wraps_throttled_log_trailing_call() {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        Text::new("Hello"),
        Options {
            max_fps: 1,
            ..Options::default()
        },
        &terminal,
    );
    app.rerender(Text::new("World"));
    assert_eq!(terminal.writes().len(), 1);
    app.advance(Duration::from_secs(1));
    assert_eq!(terminal.writes().len(), 2);
    for write in terminal.writes() {
        assert!(write.starts_with("\x1b[?2026h"));
        assert!(write.ends_with("\x1b[?2026l"));
    }
}
