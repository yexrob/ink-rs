//! Ported from ink test/cursor-exit-position.tsx @ 26d2c3f.
mod common;

use common::Screen;
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn example(cx: &mut Scope, _: &()) -> Element {
    cx.use_cursor(Some(CursorPosition { x: 2, y: 0 }));
    Text::new("Header\nFooter").into()
}

#[test]
fn unmount_returns_a_positioned_cursor_below_the_frame_incremental_false() {
    let terminal = TestTerminal::new(100, 10);
    let app = mount(component(example, ()), Options::default(), &terminal);
    app.unmount();
    terminal.write("$ command");
    assert_eq!(
        &Screen::new(&terminal, 100, 10).rows()[..3],
        ["Header", "Footer", "$ command"]
    );
}

#[test]
fn alternate_screen_exit_does_not_reposition_the_primary_cursor_incremental_false() {
    let terminal = TestTerminal::new(100, 10);
    let before = Screen::new(&terminal, 100, 10);
    let app = mount(
        component(example, ()),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    );
    app.unmount();
    let after = Screen::new(&terminal, 100, 10);
    assert!(!after.alternate_screen());
    assert_eq!(after.rows(), before.rows());
    assert_eq!(after.cursor(), before.cursor());
    assert!(after.cursor_visible());
    let output = terminal.writes().join("");
    assert_eq!(output.split_once("\x1b[?1049l").unwrap().1, "\x1b[?25h");
}
