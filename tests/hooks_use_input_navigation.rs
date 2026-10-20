//! Ported from ink test/hooks-use-input-navigation.tsx @ 26d2c3f.
//! PTY exited text/status becomes exit_result; fixture mismatches still panic.

use ink_rs::testing::{TestTerminal, mount};
use ink_rs::*;

fn fixture(mode: &'static str, bytes: &str) {
    let terminal = TestTerminal::new(100, 24);
    let mut app = mount(
        component(
            |cx: &mut Scope, mode: &&str| {
                let mode = *mode;
                let exit = cx.use_app();
                let mut down_count = 0;
                cx.use_input(true, move |input, key| {
                    if mode == "rapidArrowsEnter" {
                        if key.down_arrow {
                            down_count += 1;
                            return;
                        }
                        if key.enter && down_count == 3 {
                            exit.exit();
                            return;
                        }
                        panic!("Expected 3 down arrows then enter: {input:?} {key:?}");
                    }
                    let matched = match mode {
                        "upArrow" => key.up_arrow && !key.meta,
                        "downArrow" => key.down_arrow && !key.meta,
                        "leftArrow" => key.left_arrow && !key.meta,
                        "rightArrow" => key.right_arrow && !key.meta,
                        "upArrowMeta" => key.up_arrow && key.meta,
                        "downArrowMeta" => key.down_arrow && key.meta,
                        "leftArrowMeta" => key.left_arrow && key.meta,
                        "rightArrowMeta" => key.right_arrow && key.meta,
                        "upArrowCtrl" => key.up_arrow && key.ctrl,
                        "downArrowCtrl" => key.down_arrow && key.ctrl,
                        "leftArrowCtrl" => key.left_arrow && key.ctrl,
                        "rightArrowCtrl" => key.right_arrow && key.ctrl,
                        "pageDown" => key.page_down && !key.meta,
                        "pageUp" => key.page_up && !key.meta,
                        "home" => key.home && !key.meta,
                        "end" => key.end && !key.meta,
                        _ => false,
                    };
                    if matched {
                        exit.exit();
                    } else {
                        panic!("Unexpected input: {input:?} {key:?}");
                    }
                });
                Element::fragment(Vec::<Element>::new())
            },
            mode,
        ),
        Options::default(),
        &terminal,
    );
    app.input(bytes);
    assert!(matches!(app.exit_result(), Some(Ok(None))));
}

#[test]
fn use_input_handle_up_arrow() {
    fixture("upArrow", "\x1b[A");
}

#[test]
fn use_input_handle_down_arrow() {
    fixture("downArrow", "\x1b[B");
}

#[test]
fn use_input_handle_left_arrow() {
    fixture("leftArrow", "\x1b[D");
}

#[test]
fn use_input_handle_right_arrow() {
    fixture("rightArrow", "\x1b[C");
}

#[test]
fn use_input_handles_rapid_arrows_and_enter_in_one_chunk() {
    fixture("rapidArrowsEnter", "\x1b[B\x1b[B\x1b[B\r");
}

#[test]
fn use_input_handle_meta_up_arrow() {
    fixture("upArrowMeta", "\x1b\x1b[A");
}

#[test]
fn use_input_handle_meta_down_arrow() {
    fixture("downArrowMeta", "\x1b\x1b[B");
}

#[test]
fn use_input_handle_meta_left_arrow() {
    fixture("leftArrowMeta", "\x1b\x1b[D");
}

#[test]
fn use_input_handle_meta_right_arrow() {
    fixture("rightArrowMeta", "\x1b\x1b[C");
}

#[test]
fn use_input_handle_ctrl_up_arrow() {
    fixture("upArrowCtrl", "\x1b[1;5A");
}

#[test]
fn use_input_handle_ss3_ctrl_up_arrow() {
    fixture("upArrowCtrl", "\x1bO1;5A");
}

#[test]
fn use_input_handle_ctrl_down_arrow() {
    fixture("downArrowCtrl", "\x1b[1;5B");
}

#[test]
fn use_input_handle_ctrl_left_arrow() {
    fixture("leftArrowCtrl", "\x1b[1;5D");
}

#[test]
fn use_input_handle_ctrl_right_arrow() {
    fixture("rightArrowCtrl", "\x1b[1;5C");
}

#[test]
fn use_input_handle_page_down() {
    fixture("pageDown", "\x1b[6~");
}

#[test]
fn use_input_handle_page_up() {
    fixture("pageUp", "\x1b[5~");
}

#[test]
fn use_input_handle_home() {
    fixture("home", "\x1b[H");
}

#[test]
fn use_input_handle_end() {
    fixture("end", "\x1b[F");
}
