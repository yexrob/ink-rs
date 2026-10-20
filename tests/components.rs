//! Ported from ink test/components.tsx @ 26d2c3f.

mod common;

use std::time::Duration;

use common::{render, *};
use ink_rs::*;

// The OSC cases assert stripAnsi(output), not the escape sequence encoding.
fn strip_ansi(output: &str) -> String {
    let mut chars = output.chars().peekable();
    let mut plain = String::new();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            plain.push(c);
            continue;
        }
        match chars.next() {
            Some(']') => {
                while let Some(c) = chars.next() {
                    if c == '\u{7}' {
                        break;
                    }
                    if c == '\u{1b}' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            }
            Some('[') => {
                for c in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    plain
}

#[test]
fn text() {
    let output = render(Text::new("Hello World"));
    assert_eq!(output, "Hello World");
}

#[test]
fn text_with_variable() {
    let output = render(Text::new("Count: ").child(1.to_string()));
    assert_eq!(output, "Count: 1");
}

#[test]
fn multiple_text_nodes() {
    let output = render(Text::new("Hello").child(" World"));
    assert_eq!(output, "Hello World");
}

#[test]
fn text_with_component() {
    fn world() -> Text {
        Text::new("World")
    }
    let output = render(Text::new("Hello ").child(world()));
    assert_eq!(output, "Hello World");
}

#[test]
fn text_with_fragment() {
    let output = render(
        Text::new("Hello ")
            .child(Element::fragment([Text::new("World")]))
            .child(" "),
    );
    assert_eq!(output, "Hello World");
}

#[test]
fn wrap_text() {
    let output = render(
        View::new()
            .width(7)
            .child(Text::new("Hello World").wrap(TextWrap::Wrap)),
    );
    assert_eq!(output, lines(&["Hello", "World"]));
}

#[test]
fn dont_wrap_text_if_there_is_enough_space() {
    let output = render(
        View::new()
            .width(20)
            .child(Text::new("Hello World").wrap(TextWrap::Wrap)),
    );
    assert_eq!(output, "Hello World");
}

#[test]
fn hard_wrap_text() {
    let output = render(
        View::new()
            .width(7)
            .child(Text::new("Hello World").wrap(TextWrap::Hard)),
    );
    assert_eq!(output, lines(&["Hello W", "orld"]));
}

#[test]
fn hard_wrap_with_long_word() {
    let output = render(
        View::new()
            .width(5)
            .child(Text::new("aaaaaaaaaa").wrap(TextWrap::Hard)),
    );
    assert_eq!(output, lines(&["aaaaa", "aaaaa"]));
}

#[test]
fn dont_hard_wrap_text_if_there_is_enough_space() {
    let output = render(
        View::new()
            .width(20)
            .child(Text::new("Hello World").wrap(TextWrap::Hard)),
    );
    assert_eq!(output, "Hello World");
}

#[test]
fn truncate_text_in_the_end() {
    let output = render(
        View::new()
            .width(7)
            .child(Text::new("Hello World").wrap(TextWrap::TruncateEnd)),
    );
    assert_eq!(output, "Hello …");
}

#[test]
fn truncate_text_in_the_middle() {
    let output = render(
        View::new()
            .width(7)
            .child(Text::new("Hello World").wrap(TextWrap::TruncateMiddle)),
    );
    assert_eq!(output, "Hel…rld");
}

#[test]
fn truncate_text_in_the_beginning() {
    let output = render(
        View::new()
            .width(7)
            .child(Text::new("Hello World").wrap(TextWrap::TruncateStart)),
    );
    assert_eq!(output, "… World");
}

#[test]
fn do_not_wrap_text_with_bel_terminated_osc_hyperlinks() {
    let hyperlink = "\u{1b}]8;;https://example.com\u{7}Click here\u{1b}]8;;\u{7}";
    let output = render(
        View::new()
            .width(20)
            .child(Text::new(hyperlink).wrap(TextWrap::Wrap)),
    );
    assert_eq!(strip_ansi(&output), "Click here");
}

#[test]
fn do_not_wrap_text_with_st_terminated_osc_hyperlinks() {
    let hyperlink = "\u{1b}]8;;https://example.com\u{1b}\\Click here\u{1b}]8;;\u{1b}\\";
    let output = render(
        View::new()
            .width(20)
            .child(Text::new(hyperlink).wrap(TextWrap::Wrap)),
    );
    assert_eq!(strip_ansi(&output), "Click here");
}

#[test]
fn do_not_wrap_text_with_non_hyperlink_osc_sequences() {
    let text = "\u{1b}]0;My Title\u{7}Some text";
    let output = render(
        View::new()
            .width(20)
            .child(Text::new(text).wrap(TextWrap::Wrap)),
    );
    assert_eq!(strip_ansi(&output), "Some text");
}

#[test]
fn hard_wrap_single_word_bel_terminated_osc_hyperlink() {
    let hyperlink = "\u{1b}]8;;https://example.com\u{7}abcdefghij\u{1b}]8;;\u{7}";
    let output = render(
        View::new()
            .width(5)
            .child(Text::new(hyperlink).wrap(TextWrap::Wrap)),
    );
    assert_eq!(strip_ansi(&output), lines(&["abcde", "fghij"]));
}

#[test]
fn hard_wrap_single_word_st_terminated_osc_hyperlink() {
    let hyperlink = "\u{1b}]8;;https://example.com\u{1b}\\abcdefghij\u{1b}]8;;\u{1b}\\";
    let output = render(
        View::new()
            .width(5)
            .child(Text::new(hyperlink).wrap(TextWrap::Wrap)),
    );
    assert_eq!(strip_ansi(&output), lines(&["abcde", "fghij"]));
}

#[test]
fn ignore_empty_text_node() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(View::new().child(Text::new("Hello World")))
            .child(Text::new("")),
    );
    assert_eq!(output, "Hello World");
}

#[test]
fn render_a_single_empty_text_node() {
    let output = render(Text::new(""));
    assert_eq!(output, "");
}

#[test]
fn number() {
    let output = render(Text::new(1.to_string()));
    assert_eq!(output, "1");
}

#[test]
#[should_panic(expected = "<Box> can’t be nested inside <Text> component")]
fn fail_when_box_is_inside_text_component() {
    render(Text::new("Hello World").child(Element::from(View::new())));
}

#[test]
fn remeasure_text_dimensions_on_text_change() {
    let mut instance = testing::render(View::new().child(Text::new("Hello")));
    assert_eq!(instance.last_frame(), "Hello");
    instance.rerender(View::new().child(Text::new("Hello World")));
    assert_eq!(instance.last_frame(), "Hello World");
}

#[test]
fn fragment() {
    let output = render(Element::fragment([Text::new("Hello World")]));
    assert_eq!(output, "Hello World");
}

#[test]
fn transform_children() {
    let output = render(
        Transform::new(|string, index| format!("[{index}: {string}]")).child(Text::new(
            Transform::new(|string, index| format!("{{{index}: {string}}}"))
                .child(Text::new("test")),
        )),
    );
    assert_eq!(output, "[0: {0: test}]");
}

#[test]
fn squash_multiple_text_nodes() {
    let output = render(
        Transform::new(|string, index| format!("[{index}: {string}]")).child(Text::new(
            Transform::new(|string, index| format!("{{{index}: {string}}}"))
                .child(Text::new("hello").child(" ").child("world")),
        )),
    );
    assert_eq!(output, "[0: {0: hello world}]");
}

#[test]
fn transform_with_multiple_lines() {
    let output = render(
        Transform::new(|string, index| format!("[{index}: {string}]"))
            .child(Text::new("hello").children([" ", "world", "\n", "goodbye", " ", "world"])),
    );
    assert_eq!(output, lines(&["[0: hello world]", "[1: goodbye world]"]));
}

#[test]
fn squash_multiple_nested_text_nodes() {
    let output = render(
        Transform::new(|string, index| format!("[{index}: {string}]")).child(Text::new(
            Transform::new(|string, index| format!("{{{index}: {string}}}"))
                .child("hello")
                .child(Text::new(" world")),
        )),
    );
    assert_eq!(output, "[0: {0: hello world}]");
}

#[test]
fn squash_empty_text_nodes() {
    let output = render(
        Transform::new(|string, _| format!("[{string}]")).child(Text::new(
            Transform::new(|string, _| format!("{{{string}}}"))
                .child(Text::default().children(Vec::<TextChild>::new())),
        )),
    );
    assert_eq!(output, "");
}

#[test]
fn transform_with_undefined_children() {
    let output = render(Transform::new(|children, _| children.to_owned()));
    assert_eq!(output, "");
}

#[test]
fn transform_with_null_children() {
    let output = render(Transform::new(|children, _| children.to_owned()));
    assert_eq!(output, "");
}

#[test]
fn hooks() {
    fn with_hooks(cx: &mut Scope, _: &()) -> Element {
        let value = cx.use_state(|| "Hello");
        Text::new(value.get()).into()
    }
    assert_eq!(render(component(with_hooks, ())), "Hello");
}

#[test]
fn static_output() {
    let output = render(
        View::new()
            .child(
                Static::new(vec!["A", "B", "C"], |letter, _| Text::new(*letter).into())
                    .padding_bottom(1),
            )
            .child(View::new().margin_top(1).child(Text::new("X"))),
    );
    assert_eq!(output, lines(&["A", "B", "C", "", "", "X"]));
}

#[test]
fn static_padding_is_not_emitted_again_when_there_are_no_new_items() {
    fn example(_: &mut Scope, (status, items): &(&str, Vec<&'static str>)) -> Element {
        Element::fragment::<[Element; 2]>([
            Static::new(items.clone(), |item, _| Text::new(*item).key(*item))
                .padding(1)
                .into(),
            Text::new(*status).into(),
        ])
    }
    let mut instance = testing::render(component(example, ("Waiting", vec!["A"])));
    assert_eq!(instance.last_frame(), "\n A\n\nWaiting");
    instance.rerender(component(example, ("Ready", vec!["A"])));
    assert_eq!(instance.last_frame(), "\n A\n\nReady");
    instance.rerender(component(example, ("Done", vec!["A", "B"])));
    assert_eq!(instance.last_frame(), "\n A\n\n\n B\n\nDone");
}

#[test]
fn skip_previous_output_when_rendering_new_static_output() {
    fn dynamic(_: &mut Scope, (items,): &(Vec<&'static str>,)) -> Element {
        Static::new(items.clone(), |item, _| Text::new(*item).key(*item)).into()
    }
    let mut instance = testing::render(component(dynamic, (vec!["A"],)));
    assert_eq!(instance.last_frame(), "A\n");
    instance.rerender(component(dynamic, (vec!["A", "B"],)));
    assert_eq!(instance.last_frame(), "A\nB\n");
}

#[test]
fn static_output_stops_accumulating_after_static_unmounts_904() {
    fn app(_: &mut Scope, show: &bool) -> Element {
        View::new()
            .child(show.then(|| Static::new(vec!["A", "B"], |item, _| Text::new(*item).key(*item))))
            .child(Text::new("Dynamic"))
            .into()
    }
    let mut instance = testing::render(component(app, true));
    instance.rerender(component(app, false));
    let length = instance.last_frame().len();
    for _ in 0..10 {
        instance.rerender(component(app, false));
    }
    assert_eq!(instance.last_frame().len(), length);
    assert!(instance.last_frame().contains("Dynamic"));
}

#[test]
fn full_static_output_is_reset_when_static_unmounts_so_stale_items_are_not_replayed() {
    fn app(_: &mut Scope, (show, label): &(bool, &str)) -> Element {
        View::new()
            .child(show.then(|| {
                Static::new(vec!["HISTORY-A", "HISTORY-B"], |item, _| {
                    Text::new(*item).key(*item)
                })
            }))
            .child(Text::new(*label))
            .into()
    }
    let mut instance = testing::render(component(app, (true, "d1")));
    assert!(instance.last_frame().contains("HISTORY-A"));
    assert!(instance.last_frame().contains("HISTORY-B"));
    instance.rerender(component(app, (false, "d2")));
    assert!(!instance.last_frame().contains("HISTORY-A"));
    assert!(!instance.last_frame().contains("HISTORY-B"));
    assert!(instance.last_frame().contains("d2"));
}

#[test]
fn unmounting_an_ancestor_of_static_clears_static_node_and_does_not_crash_the_renderer() {
    fn wrapper(_: &mut Scope, _: &()) -> Element {
        View::new()
            .child(Static::new(vec!["HISTORY-X"], |item, _| {
                Text::new(*item).key(*item)
            }))
            .into()
    }
    fn app(_: &mut Scope, (show, label): &(bool, &str)) -> Element {
        View::new()
            .child(show.then(|| component(wrapper, ())))
            .child(Text::new(*label))
            .into()
    }
    let mut instance = testing::render(component(app, (true, "live-1")));
    assert!(instance.last_frame().contains("HISTORY-X"));
    instance.rerender(component(app, (false, "live-2")));
    assert!(instance.last_frame().contains("live-2"));
    assert!(!instance.last_frame().contains("HISTORY-X"));
    instance.rerender(component(app, (false, "live-3")));
    assert_eq!(instance.last_frame(), "live-3");
}

#[test]
fn removing_a_static_ancestor_that_is_a_direct_child_of_the_root_does_not_crash() {
    fn app(_: &mut Scope, (show, label): &(bool, &str)) -> Element {
        Element::fragment::<[Element; 2]>([
            show.then(|| {
                View::new().child(Static::new(vec!["ROOT-HISTORY"], |item, _| {
                    Text::new(*item).key(*item)
                }))
            })
            .into(),
            Text::new(*label).into(),
        ])
    }
    let mut instance = testing::render(component(app, (true, "root-1")));
    assert!(instance.last_frame().contains("ROOT-HISTORY"));
    instance.rerender(component(app, (false, "root-2")));
    assert!(instance.last_frame().contains("root-2"));
    assert!(!instance.last_frame().contains("ROOT-HISTORY"));
}

#[test]
fn separate_ink_instances_do_not_clobber_each_others_static_node() {
    fn app(_: &mut Scope, (show, label): &(bool, &str)) -> Element {
        View::new()
            .child(show.then(|| {
                View::new().child(Static::new(vec![format!("{label}-history")], |item, _| {
                    Text::new(item.clone()).key(item.clone())
                }))
            }))
            .child(Text::new(*label))
            .into()
    }
    let mut first = testing::render(component(app, (true, "first")));
    let mut second = testing::render(component(app, (true, "second")));
    first.rerender(component(app, (false, "first-2")));
    assert!(first.last_frame().contains("first-2"));
    assert!(!first.last_frame().contains("first-history"));
    second.rerender(component(app, (false, "second-2")));
    assert!(second.last_frame().contains("second-2"));
    first.unmount();
    second.unmount();
}

#[test]
fn updating_static_in_one_instance_after_another_instance_mounted_static_sets_the_dirty_flag_on_the_correct_root()
 {
    fn app(_: &mut Scope, (items, label): &(Vec<&'static str>, &str)) -> Element {
        View::new()
            .child(Static::new(items.clone(), |item, _| {
                Text::new(*item).key(*item)
            }))
            .child(Text::new(*label))
            .into()
    }
    let mut first = testing::render(component(app, (vec!["A"], "first")));
    let mut second = testing::render(component(app, (vec!["X"], "second")));
    first.rerender(component(app, (vec!["A", "B"], "first")));
    assert!(first.last_frame().contains('B'));
    second.rerender(component(app, (vec!["X", "Y"], "second")));
    assert!(second.last_frame().contains('Y'));
    first.unmount();
    second.unmount();
}

#[test]
fn remounting_static_via_key_change_emits_the_new_items_nested_under_box() {
    fn app(_: &mut Scope, session: &u32) -> Element {
        let items = if *session == 1 {
            vec!["old-A", "old-B"]
        } else {
            vec!["new-C", "new-D"]
        };
        View::new()
            .child(
                Static::new(items, |item, _| Text::new(*item).key(*item)).key(session.to_string()),
            )
            .child(Text::new("dynamic"))
            .into()
    }
    let mut instance = testing::render(component(app, 1));
    assert!(instance.last_frame().contains("old-A"));
    assert!(instance.last_frame().contains("old-B"));
    instance.rerender(component(app, 2));
    assert!(instance.last_frame().contains("new-C"));
    assert!(instance.last_frame().contains("new-D"));
}

#[test]
fn remounting_static_via_key_change_emits_the_new_items_root_level_remove_child_from_container() {
    fn app(_: &mut Scope, session: &u32) -> Element {
        let items = if *session == 1 {
            vec!["old-A", "old-B"]
        } else {
            vec!["new-C", "new-D"]
        };
        Static::new(items, |item, _| Text::new(*item).key(*item)).key(session.to_string())
    }
    let mut instance = testing::render(component(app, 1));
    assert!(instance.last_frame().contains("old-A"));
    assert!(instance.last_frame().contains("old-B"));
    instance.rerender(component(app, 2));
    assert!(instance.last_frame().contains("new-C"));
    assert!(instance.last_frame().contains("new-D"));
}

#[test]
fn render_only_new_items_in_static_output_on_final_render() {
    fn dynamic(_: &mut Scope, (items,): &(Vec<&'static str>,)) -> Element {
        Static::new(items.clone(), |item, _| Text::new(*item).key(*item)).into()
    }
    let mut instance = testing::render(component(dynamic, (Vec::<&str>::new(),)));
    assert_eq!(instance.last_frame(), "");
    instance.rerender(component(dynamic, (vec!["A"],)));
    assert_eq!(instance.last_frame(), "A\n");
    instance.rerender(component(dynamic, (vec!["A", "B"],)));
    // Ink #397: no render is pending and Static output was rendered, so unmount
    // writes no new content. Check that last content frame before consuming self.
    assert_eq!(instance.last_frame(), "A\nB\n");
    instance.unmount();
}

#[test]
fn ensure_wrap_ansi_doesnt_trim_leading_whitespace() {
    let output = render(Text::new(" ERROR ").color(Color::Red));
    assert_ansi_eq(&output, &styled(&["{fg=red} ERROR {/}"]));
}

#[test]
fn replace_child_node_with_text() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        if *value {
            Text::new("x").into()
        } else {
            Text::new(Text::new("test").color(Color::Green)).into()
        }
    }
    let mut instance = testing::render(component(example, false));
    assert_ansi_eq(instance.last_frame(), &styled(&["{fg=green}test{/}"]));
    instance.rerender(component(example, true));
    assert_ansi_eq(instance.last_frame(), "x");
}

thread_local! {
    static LIVE_COUNT: std::cell::Cell<Option<State<u32>>> = const { std::cell::Cell::new(None) };
}

fn live_counter(cx: &mut Scope, _: &()) -> Element {
    let count = cx.use_state(|| 0);
    LIVE_COUNT.set(Some(count));
    Text::new(format!("Count: {}", count.get())).into()
}

fn noninteractive_counter(tty: bool, interactive: Option<bool>) {
    let terminal = testing::TestTerminal::new(100, 24).with_tty(tty);
    let mut app = testing::mount(
        component(live_counter, ()),
        Options {
            interactive,
            ..Options::default()
        },
        &terminal,
    );
    for count in 1..=3 {
        LIVE_COUNT.get().unwrap().set(count);
        app.advance(Duration::from_millis(10));
    }
    app.unmount();
    for intermediate in ["Count: 0", "Count: 1", "Count: 2"] {
        assert!(!terminal.writes().join("").contains(intermediate));
    }
    assert!(!terminal.writes().join("").contains("\x1b["));
    assert!(terminal.writes().last().unwrap().contains("Count: 3"));
}

#[test]
fn render_only_last_frame_when_stdout_is_not_a_tty() {
    noninteractive_counter(false, None);
}

#[test]
fn render_all_frames_when_interactive_is_explicitly_true() {
    let terminal = testing::TestTerminal::new(100, 24).with_tty(false);
    let mut app = testing::mount(
        component(live_counter, ()),
        Options {
            interactive: Some(true),
            ..Options::default()
        },
        &terminal,
    );
    assert_eq!(Screen::new(&terminal, 100, 24).rows()[0], "Count: 0");
    for count in 1..=2 {
        LIVE_COUNT.get().unwrap().set(count);
        app.advance(Duration::from_millis(50));
        assert_eq!(
            Screen::new(&terminal, 100, 24).rows()[0],
            format!("Count: {count}")
        );
    }
    app.unmount();
    assert!(terminal.writes().len() > 1);
    assert!(!terminal.writes().join("").contains("\x1b[?2026"));
}

#[test]
fn interactive_option_overrides_tty_detection() {
    noninteractive_counter(true, Some(false));
}

#[test]
fn alternate_screen_enters_on_mount_and_exits_on_unmount() {
    let terminal = testing::TestTerminal::new(100, 24);
    testing::mount(
        Text::new("Hello"),
        Options {
            alternate_screen: true,
            interactive: Some(true),
            ..Options::default()
        },
        &terminal,
    )
    .unmount();
    let writes = terminal.writes();
    assert!(writes[0].starts_with("\x1b[?1049h"));
    let exit = writes
        .iter()
        .position(|s| s.contains("\x1b[?1049l"))
        .unwrap();
    assert!(exit > 0);
    assert!(writes.join("").ends_with("\x1b[?1049l\x1b[?25h"));
}

#[test]
fn alternate_screen_cleanup_exits_the_alternate_screen() {
    let terminal = testing::TestTerminal::new(100, 24);
    let app = testing::mount(
        Text::new("Hello"),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    );
    drop(app);
    assert!(terminal.writes().join("").contains("\x1b[?1049l"));
}

fn no_alternate_screen(tty: bool, options: Options) {
    let terminal = testing::TestTerminal::new(100, 24).with_tty(tty);
    testing::mount(Text::new("Hello"), options, &terminal).unmount();
    let output = terminal.writes().join("");
    assert!(!output.contains("\x1b[?1049h"));
    assert!(!output.contains("\x1b[?1049l"));
    if !tty || options.interactive == Some(false) {
        assert!(!output.contains("\x1b[?2026"));
    }
}

#[test]
fn alternate_screen_ignored_when_non_interactive() {
    no_alternate_screen(
        true,
        Options {
            alternate_screen: true,
            interactive: Some(false),
            ..Options::default()
        },
    );
}

#[test]
fn alternate_screen_disabled_by_default() {
    no_alternate_screen(
        true,
        Options {
            interactive: Some(true),
            ..Options::default()
        },
    );
}

#[test]
fn alternate_screen_content_is_rendered_between_enter_and_exit() {
    let terminal = testing::TestTerminal::new(100, 24);
    testing::mount(
        Text::new("Hello"),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
        &terminal,
    )
    .unmount();
    let writes = terminal.writes();
    let enter = writes
        .iter()
        .position(|s| s.contains("\x1b[?1049h"))
        .unwrap();
    let exit = writes
        .iter()
        .position(|s| s.contains("\x1b[?1049l"))
        .unwrap();
    assert!(enter < exit);
    assert!(writes[enter + 1..exit].iter().any(|s| s.contains("Hello")));
}

#[test]
fn alternate_screen_ignored_when_is_tty_is_false() {
    no_alternate_screen(
        false,
        Options {
            alternate_screen: true,
            ..Options::default()
        },
    );
}

#[test]
fn alternate_screen_ignored_when_is_tty_is_false_even_if_interactive_is_true() {
    no_alternate_screen(
        false,
        Options {
            alternate_screen: true,
            interactive: Some(true),
            ..Options::default()
        },
    );
}

#[test]
fn static_output_is_written_immediately_in_non_interactive_mode() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let count = cx.use_state(|| 0);
        LIVE_COUNT.set(Some(count));
        View::new()
            .child(Static::new(
                if count.get() == 0 {
                    vec!["A"]
                } else {
                    vec!["A", "B"]
                },
                |item, _| Text::new(*item).key(*item),
            ))
            .child(Text::new("Dynamic"))
            .into()
    }
    let terminal = testing::TestTerminal::new(100, 24).with_tty(false);
    let mut app = testing::mount(component(example, ()), Options::default(), &terminal);
    LIVE_COUNT.get().unwrap().set(1);
    app.advance(Duration::from_millis(200));
    let before = terminal.writes().join("");
    assert!(before.contains('A'));
    assert!(before.contains('B'));
    assert!(!before.contains("Dynamic"));
    app.unmount();
    assert!(terminal.writes().join("").contains("Dynamic"));
}

#[test]
fn reset_prop_when_it_s_removed_from_the_element() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let mut view = View::new()
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::FlexEnd);
        if !*value {
            view = view.height(4);
        }
        view.child(Text::new("x")).into()
    }
    let mut instance = testing::render(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["", "", "", "x"]));
    instance.rerender(component(example, true));
    assert_ansi_eq(instance.last_frame(), "x");
}

#[test]
fn newline() {
    let output = render(Text::new("Hello").child(Newline::new()).child("World"));
    assert_eq!(output, lines(&["Hello", "World"]));
}

#[test]
fn multiple_newlines() {
    let output = render(Text::new("Hello").child(Newline::count(2)).child("World"));
    assert_eq!(output, lines(&["Hello", "", "World"]));
}

#[test]
fn horizontal_spacer() {
    let output = render(
        View::new()
            .width(20)
            .child(Text::new("Left"))
            .child(Spacer)
            .child(Text::new("Right")),
    );
    assert_eq!(output, "Left           Right");
}

#[test]
fn vertical_spacer() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .height(6)
            .child(Text::new("Top"))
            .child(Spacer)
            .child(Text::new("Bottom")),
    );
    assert_eq!(output, lines(&["Top", "", "", "", "", "Bottom"]));
}

#[test]
fn link_ansi_escapes_are_closed_properly() {
    let output = render(Text::new(
        "\u{1b}]8;;https://example.com\u{7}Example\u{1b}]8;;\u{7}",
    ));
    assert_eq!(
        output,
        "\u{1b}]8;;https://example.com\u{7}Example\u{1b}]8;;\u{7}"
    );
}
