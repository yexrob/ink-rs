//! Ported from ink test/components.tsx @ 26d2c3f.

mod common;

use common::*;
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
fn ensure_wrap_ansi_doesnt_trim_leading_whitespace() {
    let output = render(Text::new(" ERROR ").color(Color::Red));
    assert_ansi_eq(&output, &styled(&["{fg=red} ERROR {/}"]));
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
