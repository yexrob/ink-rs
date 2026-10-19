//! Ported from ink test/render-to-string.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn render_simple_text() {
    let output = render_cols(Text::new("Hello World"), 80);
    assert_eq!(output, "Hello World");
}

#[test]
fn skip_static_content_inside_a_hidden_ancestor() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            View::new()
                .display(Display::None)
                .child(Static::new(vec!["Hidden"], |item, _| {
                    View::new()
                        .border_style(BorderStyle::Single)
                        .child(Text::new(*item))
                        .into()
                }))
                .into(),
            Text::new("Visible").into(),
        ]),
        80,
    );
    assert_eq!(output, "Visible");
}

#[test]
fn render_text_with_variable() {
    let output = render_cols(Text::new(format!("Count: {}", 42)), 80);
    assert_eq!(output, "Count: 42");
}

#[test]
fn render_nested_text_components() {
    let output = render_cols(Text::new("Hello ").child(Text::new("World")), 80);
    assert_eq!(output, "Hello World");
}

#[test]
fn render_empty_fragment() {
    let output = render_cols(Element::fragment(Vec::<Element>::new()), 80);
    assert_eq!(output, "");
}

#[test]
fn render_null_children() {
    let output = render_cols(Text::new(Element::from(None::<Text>)), 80);
    assert_eq!(output, "");
}

#[test]
fn render_box_with_padding() {
    let output = render_cols(View::new().padding_left(2).child(Text::new("Padded")), 80);
    assert_eq!(output, "  Padded");
}

#[test]
fn render_box_with_flex_direction_row() {
    let output = render_cols(
        View::new().children([Text::new("A"), Text::new("B"), Text::new("C")]),
        80,
    );
    assert_eq!(output, "ABC");
}

#[test]
fn render_box_with_flex_direction_column() {
    let output = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children([Text::new("Line 1"), Text::new("Line 2")]),
        80,
    );
    assert_eq!(output, lines(&["Line 1", "Line 2",]));
}

#[test]
fn render_margin() {
    let output = render_cols(View::new().margin_left(2).child(Text::new("Margined")), 80);
    assert_eq!(output, "  Margined");
}

#[test]
fn render_gap_between_items() {
    let output = render_cols(
        View::new()
            .gap(1)
            .children([Text::new("A"), Text::new("B")]),
        80,
    );
    assert_eq!(output, "A B");
}

#[test]
fn render_box_with_fixed_width_and_height() {
    let output = render_cols(View::new().width(10).height(3).child(Text::new("Hi")), 80);
    assert_eq!(output.split('\n').count(), 3);
}

#[test]
fn render_spacer_pushes_content_apart() {
    let output = render_cols(
        View::new()
            .width(20)
            .child(Text::new("Left"))
            .child(Spacer)
            .child(Text::new("Right")),
        80,
    );
    assert_eq!(output, "Left           Right");
}

#[test]
fn render_newline_inserts_blank_line() {
    let output = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("Above"))
            .child(Newline::new())
            .child(Text::new("Below")),
        80,
    );
    assert_eq!(output, lines(&["Above", "", "", "Below",]));
}

#[test]
fn render_box_with_border() {
    let output = render_cols(
        View::new()
            .border_style(BorderStyle::Single)
            .width(20)
            .child(Text::new("Bordered")),
        20,
    );
    assert_eq!(
        output,
        lines(&[
            "┌──────────────────┐",
            "│Bordered          │",
            "└──────────────────┘",
        ])
    );
}

#[test]
fn render_colored_text() {
    let output = render_cols(Text::new("Green").color(Color::Green), 80);
    assert_ansi_eq(&output, &styled(&["{fg=green}Green{/}"]));
}

#[test]
fn render_bold_text() {
    let output = render_cols(Text::new("Bold").bold(true), 80);
    assert_ansi_eq(&output, &styled(&["{bold}Bold{/}"]));
}

#[test]
fn render_text_with_wrap() {
    let output = render_cols(
        View::new()
            .width(7)
            .child(Text::new("Hello World").wrap(TextWrap::Wrap)),
        80,
    );
    assert_eq!(output, lines(&["Hello", "World",]));
}

#[test]
fn render_text_with_truncate() {
    let output = render_cols(
        View::new()
            .width(7)
            .child(Text::new("Hello World").wrap(TextWrap::TruncateEnd)),
        80,
    );
    assert_eq!(output, "Hello …");
}

#[test]
fn default_columns_is_80() {
    let output = render_cols(Text::new("A".repeat(100)), 80);
    let rows: Vec<_> = output.split('\n').collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0],
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    );
    assert_eq!(rows[1], "AAAAAAAAAAAAAAAAAAAA");
}

#[test]
fn custom_columns_option() {
    let output = render_cols(Text::new("A".repeat(50)), 30);
    let rows: Vec<_> = output.split('\n').collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(rows[1], "AAAAAAAAAAAAAAAAAAAA");
}

#[test]
fn render_transform_component() {
    let output = render_cols(
        Transform::new(|output, _| output.to_uppercase()).child(Text::new("hello")),
        80,
    );
    assert_eq!(output, "HELLO");
}

#[test]
fn render_static_component_with_items() {
    let output = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Static::new(vec!["A", "B", "C"], |item, _| {
                Text::new(*item).into()
            }))
            .child(Text::new("Dynamic")),
        80,
    );
    assert_eq!(output, lines(&["A", "B", "C", "Dynamic",]));
}

#[test]
fn static_preserves_its_margins_and_all_items() {
    let output = render_cols(
        Static::new(vec!["A", "B"], |item, _| Text::new(*item).into())
            .margin_top(1)
            .margin_bottom(1)
            .margin_left(2),
        80,
    );
    assert_eq!(output, lines(&["", "  A", "  B", "",]));
}

#[test]
fn render_static_only_output_has_no_trailing_newline() {
    let output = render_cols(
        Static::new(vec!["A", "B"], |item, _| Text::new(*item).into()),
        80,
    );
    assert_eq!(output, lines(&["A", "B",]));
}

#[test]
fn render_static_dynamic_output_has_exactly_one_newline_between_parts() {
    let output = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Static::new(vec!["A", "B"], |item, _| {
                Text::new(*item).into()
            }))
            .child(Text::new("Dynamic")),
        80,
    );
    assert_eq!(output, lines(&["A", "B", "Dynamic",]));
}

#[test]
fn can_be_called_multiple_times_independently() {
    let output1 = render_cols(Text::new("First"), 80);
    let output2 = render_cols(Text::new("Second"), 80);
    assert_eq!(output1, "First");
    assert_eq!(output2, "Second");
}

#[test]
fn render_deeply_nested_component_tree() {
    let output = render_cols(
        View::new().flex_direction(FlexDirection::Column).child(
            View::new().padding_left(1).child(
                View::new().child(
                    Text::new("Nested ")
                        .bold(true)
                        .child(Text::new("deep").color(Color::Green)),
                ),
            ),
        ),
        80,
    );
    assert!(output.contains("Nested"));
    assert!(output.contains("deep"));
}
