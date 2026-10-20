//! Ported from Ink 8.0.0 test/errors.tsx, commit 26d2c3f.
mod common;

use std::panic::panic_any;
use std::sync::Arc;

use common::Screen;
use ink_rs::testing::{TestTerminal, mount};
use ink_rs::{Element, Error, Options, Scope, Text, component};

#[test]
fn catch_and_display_error() {
    let terminal = TestTerminal::new(100, 24);

    fn broken(_: &mut Scope, _: &()) -> Element {
        panic!("Oh no");
    }

    let app = mount(component(broken, ()), Options::default(), &terminal);
    let lines = Screen::new(&terminal, 100, 24).lines();
    assert_eq!(
        &lines[..12],
        [
            "",
            "  ERROR  Oh no",
            "",
            " tests/errors.rs:16:9",
            "",
            " 13:     let terminal = TestTerminal::new(100, 24);",
            " 14:",
            " 15:     fn broken(_: &mut Scope, _: &()) -> Element {",
            " 16:         panic!(\"Oh no\");",
            " 17:     }",
            " 18:",
            " 19:     let app = mount(component(broken, ()), Options::default(), &terminal);",
        ]
    );
    assert!(matches!(app.exit_result(), Some(Err(Error::Panic(_)))));
}

#[test]
fn error_boundary_catches_and_displays_nested_component_errors() {
    fn nested(_: &mut Scope, _: &()) -> Element {
        panic!("Nested component error");
    }
    fn parent(_: &mut Scope, _: &()) -> Element {
        Text::new("Before error")
            .child(component(nested, ()))
            .into()
    }
    let terminal = TestTerminal::new(100, 1000);
    let _app = mount(component(parent, ()), Options::default(), &terminal);
    let output = Screen::new(&terminal, 100, 1000).lines().join("\n");
    assert!(output.contains("ERROR"));
    assert!(output.contains("Nested component error"));
}

#[test]
fn clean_up_raw_mode_when_error_is_thrown() {
    fn broken(cx: &mut Scope, _: &()) -> Element {
        cx.use_input(true, |_, _| {});
        cx.use_effect((), || panic!("Error after raw mode enabled"));
        Text::new("Test").into()
    }
    let terminal = TestTerminal::new(100, 1000).with_stdin_tty(true);
    let app = mount(component(broken, ()), Options::default(), &terminal);
    assert_eq!(terminal.raw_mode_calls(), [true, false]);
    assert!(matches!(app.exit_result(), Some(Err(Error::Panic(_)))));
}

#[test]
fn display_thrown_strings_and_reject_wait_until_exit_with_the_original_message() {
    fn broken(_: &mut Scope, _: &()) -> Element {
        panic_any(String::from("Unable to load configuration"));
    }
    let terminal = TestTerminal::new(100, 1000);
    let app = mount(component(broken, ()), Options::default(), &terminal);
    let Some(Err(Error::Panic(error))) = app.exit_result() else {
        panic!("expected component panic");
    };
    assert_eq!(error.message(), "Unable to load configuration");
    let output = Screen::new(&terminal, 100, 1000).lines().join("\n");
    assert!(output.contains("ERROR"));
    assert!(output.contains("Unable to load configuration"));
}

#[test]
fn display_thrown_undefined_and_reject_wait_until_exit() {
    fn broken(_: &mut Scope, _: &()) -> Element {
        panic_any(());
    }
    let terminal = TestTerminal::new(100, 1000);
    let app = mount(component(broken, ()), Options::default(), &terminal);
    let Some(Err(Error::Panic(error))) = app.exit_result() else {
        panic!("expected component panic");
    };
    assert_eq!(error.message(), "Box<dyn Any>");
    assert!(
        Screen::new(&terminal, 100, 1000)
            .lines()
            .join("\n")
            .contains("ERROR  Box<dyn Any>")
    );
}

#[test]
fn wait_until_exit_preserves_the_original_component_error() {
    #[derive(Debug, PartialEq)]
    struct OriginalError;
    fn broken(_: &mut Scope, error: &Arc<OriginalError>) -> Element {
        panic_any(error.clone());
    }
    let error = Arc::new(OriginalError);
    let terminal = TestTerminal::new(100, 1000);
    let mut app = mount(
        component(broken, error.clone()),
        Options::default(),
        &terminal,
    );
    let Some(Err(Error::Panic(caught))) = app.take_exit_result() else {
        panic!("expected component panic");
    };
    let payload = caught
        .into_payload()
        .downcast::<Arc<OriginalError>>()
        .unwrap();
    assert!(Arc::ptr_eq(&error, &payload));
}
