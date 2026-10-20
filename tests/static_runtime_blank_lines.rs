//! Ported from ink test/static-runtime-blank-lines.tsx @ 26d2c3f.

mod common;

use common::Screen;
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;
use std::time::Duration;

#[test]
fn runtime_static_preserves_0_blank_rows_in_debug_output() {
    let mut instance = testing::render(Element::fragment::<[Element; 2]>([
        Static::new(vec!["blank"], |item, _| View::new().height(0).key(*item)).into(),
        Text::new("after").into(),
    ]));
    instance.settle();
    assert_eq!(instance.last_frame(), "after");
}

#[test]
fn runtime_static_preserves_0_blank_rows_in_non_interactive_output() {
    let terminal = TestTerminal::new(100, 24).with_tty(false);
    let app = mount(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["blank"], |item, _| View::new().height(0).key(*item)).into(),
            Text::new("after").into(),
        ]),
        Options {
            interactive: Some(false),
            ..Options::default()
        },
        &terminal,
    );
    app.unmount();
    assert_eq!(terminal.writes().join(""), "after\n");
}

#[test]
fn runtime_static_preserves_1_blank_rows_in_debug_output() {
    let mut instance = testing::render(Element::fragment::<[Element; 2]>([
        Static::new(vec!["blank"], |item, _| View::new().height(1).key(*item)).into(),
        Text::new("after").into(),
    ]));
    instance.settle();
    assert_eq!(instance.last_frame(), "\nafter");
}

#[test]
fn runtime_static_preserves_1_blank_rows_in_non_interactive_output() {
    let terminal = TestTerminal::new(100, 24).with_tty(false);
    let app = mount(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["blank"], |item, _| View::new().height(1).key(*item)).into(),
            Text::new("after").into(),
        ]),
        Options {
            interactive: Some(false),
            ..Options::default()
        },
        &terminal,
    );
    app.unmount();
    assert_eq!(terminal.writes().join(""), "\nafter\n");
}

#[test]
fn runtime_static_preserves_2_blank_rows_in_debug_output() {
    let mut instance = testing::render(Element::fragment::<[Element; 2]>([
        Static::new(vec!["blank"], |item, _| View::new().height(2).key(*item)).into(),
        Text::new("after").into(),
    ]));
    instance.settle();
    assert_eq!(instance.last_frame(), "\n\nafter");
}

#[test]
fn runtime_static_preserves_2_blank_rows_in_non_interactive_output() {
    let terminal = TestTerminal::new(100, 24).with_tty(false);
    let app = mount(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["blank"], |item, _| View::new().height(2).key(*item)).into(),
            Text::new("after").into(),
        ]),
        Options {
            interactive: Some(false),
            ..Options::default()
        },
        &terminal,
    );
    app.unmount();
    assert_eq!(terminal.writes().join(""), "\n\nafter\n");
}

#[test]
fn a_blank_static_append_survives_a_later_live_update_incremental_false() {
    fn view(items: Vec<&'static str>, label: &str) -> Element {
        Element::fragment::<[Element; 2]>([
            Static::new(items, |item, _| -> Element {
                if *item == "blank" {
                    View::new().height(1).key(*item)
                } else {
                    Text::new(*item).key(*item)
                }
            })
            .into(),
            Text::new(label).into(),
        ])
    }
    let terminal = TestTerminal::new(100, 8);
    let mut app = mount(view(vec!["first"], "live"), Options::default(), &terminal);
    for (items, label) in [
        (vec!["first", "blank"], "live"),
        (vec!["first", "blank", "last"], "live"),
        (vec!["first", "blank", "last"], "updated"),
    ] {
        app.rerender(view(items, label));
        app.advance(Duration::from_millis(34));
    }
    assert_eq!(
        &Screen::new(&terminal, 100, 8).lines()[..4],
        ["first", "", "last", "updated"]
    );
}
