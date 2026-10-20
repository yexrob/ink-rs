use super::{excerpt, frames};
use crate::{ColorLevel, RenderOptions, render_to_string};

// Ported from Ink 8.0.0 test/error-overview.tsx, commit 26d2c3f.
fn overview(message: &str, location: Option<(&str, u32, u32)>, stack: Option<&str>) -> String {
    render_to_string(
        super::build(message, location, stack),
        RenderOptions {
            columns: 100,
            color_level: ColorLevel::None,
        },
    )
}

#[test]
fn renders_source_excerpts_for_paths_containing_spaces() {
    let output = overview(
        "Oh no",
        Some(("tests/fixtures/error source.txt", 1, 1)),
        None,
    );
    assert!(output.contains("panic!(\"Source excerpt marker\");"));
    assert!(output.contains("error source.txt:1:1"));
}

#[test]
fn renders_the_original_error_when_its_source_path_cannot_be_read() {
    let output = overview("Oh no", Some(("tests/fixtures", 1, 1)), None);
    assert!(output.contains("Oh no"));
    assert!(output.contains("fixtures:1:1"));
    assert!(!output.contains(" 1:"));
}

#[test]
fn renders_the_original_error_when_its_source_file_is_missing() {
    let output = overview("Oh no", Some(("missing-source-file.rs", 1, 1)), None);
    assert!(output.contains("Oh no"));
    assert!(output.contains("missing-source-file.rs:1:1"));
    assert!(!output.contains(" 1:"));
}

#[test]
fn multiline_error_messages_are_not_treated_as_stack_frames() {
    let output = overview(
        "Validation failed\nRequired field is missing",
        Some(("tests/fixtures/error source.txt", 1, 1)),
        None,
    );
    assert!(output.contains("panic!(\"Source excerpt marker\");"));
    assert!(output.contains("Validation failed"));
    assert_eq!(output.matches("Required field is missing").count(), 1);
}

#[test]
fn renders_native_stack_frames_as_raw_lines() {
    let output = overview("Oh no", None, Some("0: native"));
    assert!(output.contains(" - native"));
    assert!(!output.contains("undefined"));
}

#[test]
fn renders_named_native_stack_frames_as_raw_lines() {
    let output = overview("Oh no", None, Some("0: foo (native)"));
    assert!(output.contains(" - foo (native)"));
    assert!(!output.contains("foo (::)"));
    assert!(!output.contains("undefined"));
}

#[test]
fn excerpt_clips_three_lines_on_each_side_and_rejects_invalid_lines() {
    assert_eq!(excerpt("1\n2\n3\n4\n5\n6\n7\n8", 4).len(), 7);
    assert_eq!(excerpt("1\n2\n3\n4\n5", 1).len(), 4);
    assert_eq!(excerpt("1\n2\n3\n4\n5", 5).len(), 4);
    assert!(excerpt("one", 0).is_empty());
    assert!(excerpt("one", 2).is_empty());
}

#[test]
fn excerpt_converts_leading_tabs_and_crlf_but_preserves_internal_tabs() {
    assert_eq!(excerpt("last\r", 1), [(1, "last\r".into())]);
    assert_eq!(
        excerpt("\t\tstart\tend\r\n \tindented", 1),
        [(1, "    start\tend".into()), (2, " \tindented".into())]
    );
}

#[test]
fn frames_drop_internals_and_stop_at_the_component_runtime_caller() {
    assert_eq!(
        frames(
            "0: std::backtrace::capture
                 at std.rs:1:1
             1: ink_rs::runtime::panic::install::{{closure}}
             2: ink_rs::driver::process::install_panic_hook::{{closure}}
             3: core::panicking::panic_fmt
             4: app::broken
                 at src/main.rs:9:5
             5: core::iter::traits::iterator::Iterator::for_each
             6: <core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next
             7: <alloc::vec::Vec<T> as core::ops::drop::Drop>::drop
             8: alloc::boxed::Box<T>::call
             9: <std::thread::Thread as core::clone::Clone>::clone
            10: panic_unwind::imp::panic
            11: __rustc::rust_begin_unwind
            12: app::component
                 at src/main.rs:20:3
            13: <ink_rs::runtime::TypedComponent<F,P> as ink_rs::runtime::RenderComponent>::render
            14: ink_rs::runtime::Runtime::commit
            15: std::rt::lang_start
            16: _main"
        ),
        [
            ("app::broken".into(), Some("src/main.rs:9:5".into())),
            ("app::component".into(), Some("src/main.rs:20:3".into()))
        ]
    );
}

#[test]
fn frames_keep_leading_ink_rs_api_frames() {
    assert_eq!(
        frames(
            "0: core::panicking::panic_fmt
             1: ink_rs::runtime::state::State<T>::get
                 at src/runtime/state.rs:10:5
             2: <ink_rs::runtime::state::State<T> as core::clone::Clone>::clone
             3: app::component
                 at src/main.rs:20:3
             4: ink_rs::runtime::Runtime::commit
             5: std::rt::lang_start
             6: _main"
        ),
        [
            (
                "ink_rs::runtime::state::State<T>::get".into(),
                Some("src/runtime/state.rs:10:5".into())
            ),
            (
                "<ink_rs::runtime::state::State<T> as core::clone::Clone>::clone".into(),
                None
            ),
            ("app::component".into(), Some("src/main.rs:20:3".into()))
        ]
    );
}

#[test]
fn incomplete_frame_locations_remain_raw_lines() {
    assert_eq!(
        frames("0: foo\n at native"),
        [("foo".into(), None), ("at native".into(), None)]
    );
    assert_eq!(
        frames("0: foo\n at file.rs:0:1"),
        [("foo".into(), None), ("at file.rs:0:1".into(), None)]
    );
}
