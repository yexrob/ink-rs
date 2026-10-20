//! Ported from Ink test/suspension-resize.tsx (Ink 8.0.0, commit 26d2c3f).
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::{Options, Text};

fn resize(columns: u16) {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(Text::new("hello"), Options::default(), &terminal);
    app.suspend_terminal().unwrap();
    let count = terminal.writes().len();
    app.resize(columns, 24);
    assert_eq!(terminal.writes().len(), count);
    app.resume_terminal();
    assert!(terminal.writes()[count..].concat().contains("hello"));
}

#[test]
fn resize_to_80_columns_does_not_write_while_suspended() {
    resize(80);
}

#[test]
fn resize_to_120_columns_does_not_write_while_suspended() {
    resize(120);
}
