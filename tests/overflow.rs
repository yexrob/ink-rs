//! Ported from ink test/overflow.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn vertical_clipping_preserves_transform_line_indices() {
    let output = render(
        View::new()
            .height(2)
            .overflow(Overflow::Hidden)
            .content_offset_y(1)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(
                        Transform::new(|line, index| line.replace('x', &index.to_string()))
                            .child(Text::new("xx\nxx\nxx")),
                    ),
            ),
    );
    assert_eq!(output, lines(&["11", "22",]));
}

#[test]
fn overflow_x_single_text_node_in_a_box_inside_overflow_container() {
    let output = render(
        View::new().width(6).overflow_x(Overflow::Hidden).child(
            View::new()
                .width(16)
                .flex_shrink(0.0)
                .child(Text::new("Hello World")),
        ),
    );
    assert_eq!(output, "Hello");
}

#[test]
fn overflow_x_single_text_node_in_a_zero_width_overflow_container() {
    let output = render(
        View::new()
            .child(
                View::new()
                    .width(0)
                    .overflow_x(Overflow::Hidden)
                    .child(Text::new("hello")),
            )
            .child(Text::new("|")),
    );
    assert_eq!(output, "|");
}

#[test]
fn overflow_x_single_text_node_inside_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(6)
            .overflow_x(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .width(16)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello World")),
            ),
    );
    assert_eq!(output, lines(&["╭────╮", "│Hell│", "╰────╯",]));
}

#[test]
fn overflow_x_single_text_node_in_a_box_with_border_inside_overflow_container() {
    let output = render(
        View::new().width(6).overflow_x(Overflow::Hidden).child(
            View::new()
                .width(16)
                .flex_shrink(0.0)
                .border_style(BorderStyle::Round)
                .child(Text::new("Hello World")),
        ),
    );
    assert_eq!(output, lines(&["╭─────", "│Hello", "╰─────",]));
}

#[test]
fn overflow_x_multiple_text_nodes_in_a_box_inside_overflow_container() {
    let output = render(
        View::new().width(6).overflow_x(Overflow::Hidden).child(
            View::new()
                .width(12)
                .flex_shrink(0.0)
                .child(Text::new("Hello "))
                .child(Text::new("World")),
        ),
    );
    assert_eq!(output, "Hello");
}

#[test]
fn overflow_x_multiple_text_nodes_in_a_box_inside_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(8)
            .overflow_x(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .width(12)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello "))
                    .child(Text::new("World")),
            ),
    );
    assert_eq!(output, lines(&["╭──────╮", "│Hello │", "╰──────╯",]));
}

#[test]
fn overflow_x_multiple_text_nodes_in_a_box_with_border_inside_overflow_container() {
    let output = render(
        View::new().width(8).overflow_x(Overflow::Hidden).child(
            View::new()
                .width(12)
                .flex_shrink(0.0)
                .border_style(BorderStyle::Round)
                .child(Text::new("Hello "))
                .child(Text::new("World")),
        ),
    );
    assert_eq!(output, lines(&["╭───────", "│HelloWo", "│", "╰───────",]));
}

#[test]
fn overflow_x_multiple_boxes_inside_overflow_container() {
    let output = render(
        View::new()
            .width(6)
            .overflow_x(Overflow::Hidden)
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello ")),
            )
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("World")),
            ),
    );
    assert_eq!(output, "Hello");
}

#[test]
fn overflow_x_multiple_boxes_inside_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(8)
            .overflow_x(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello ")),
            )
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("World")),
            ),
    );
    assert_eq!(output, lines(&["╭──────╮", "│Hello │", "╰──────╯",]));
}

#[test]
fn overflow_x_box_before_left_edge_of_overflow_container() {
    let output = render(
        View::new().width(6).overflow_x(Overflow::Hidden).child(
            View::new()
                .margin_left(-12)
                .width(6)
                .flex_shrink(0.0)
                .child(Text::new("Hello")),
        ),
    );
    assert_eq!(output, "");
}

#[test]
fn overflow_x_box_before_left_edge_of_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(6)
            .overflow_x(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .margin_left(-12)
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello")),
            ),
    );
    assert_eq!(output, lines(&["╭────╮", "│    │", "╰────╯",]));
}

#[test]
fn overflow_x_box_intersecting_with_left_edge_of_overflow_container() {
    let output = render(
        View::new().width(6).overflow_x(Overflow::Hidden).child(
            View::new()
                .margin_left(-3)
                .width(12)
                .flex_shrink(0.0)
                .child(Text::new("Hello World")),
        ),
    );
    assert_eq!(output, "lo Wor");
}

#[test]
fn overflow_x_box_intersecting_with_left_edge_of_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(8)
            .overflow_x(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .margin_left(-3)
                    .width(12)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello World")),
            ),
    );
    assert_eq!(output, lines(&["╭──────╮", "│lo Wor│", "╰──────╯",]));
}

#[test]
fn overflow_x_box_after_right_edge_of_overflow_container() {
    let output = render(
        View::new().width(6).overflow_x(Overflow::Hidden).child(
            View::new()
                .margin_left(6)
                .width(6)
                .flex_shrink(0.0)
                .child(Text::new("Hello")),
        ),
    );
    assert_eq!(output, "");
}

#[test]
fn overflow_x_box_intersecting_with_right_edge_of_overflow_container() {
    let output = render(
        View::new().width(6).overflow_x(Overflow::Hidden).child(
            View::new()
                .margin_left(3)
                .width(6)
                .flex_shrink(0.0)
                .child(Text::new("Hello")),
        ),
    );
    assert_eq!(output, "   Hel");
}

#[test]
fn overflow_y_single_text_node_inside_overflow_container() {
    let output = render(
        View::new()
            .height(1)
            .overflow_y(Overflow::Hidden)
            .child(Text::new("Hello\nWorld")),
    );
    assert_eq!(output, "Hello");
}

#[test]
fn overflow_y_single_text_node_inside_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(20)
            .height(3)
            .overflow_y(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(Text::new("Hello\nWorld")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────╮",
            "│Hello             │",
            "╰──────────────────╯",
        ])
    );
}

#[test]
fn overflow_y_multiple_boxes_inside_overflow_container() {
    let output = render(
        View::new()
            .height(2)
            .overflow_y(Overflow::Hidden)
            .flex_direction(FlexDirection::Column)
            .child(View::new().flex_shrink(0.0).child(Text::new("Line #1")))
            .child(View::new().flex_shrink(0.0).child(Text::new("Line #2")))
            .child(View::new().flex_shrink(0.0).child(Text::new("Line #3")))
            .child(View::new().flex_shrink(0.0).child(Text::new("Line #4"))),
    );
    assert_eq!(output, lines(&["Line #1", "Line #2",]));
}

#[test]
fn overflow_y_multiple_boxes_inside_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(9)
            .height(4)
            .overflow_y(Overflow::Hidden)
            .flex_direction(FlexDirection::Column)
            .border_style(BorderStyle::Round)
            .child(View::new().flex_shrink(0.0).child(Text::new("Line #1")))
            .child(View::new().flex_shrink(0.0).child(Text::new("Line #2")))
            .child(View::new().flex_shrink(0.0).child(Text::new("Line #3")))
            .child(View::new().flex_shrink(0.0).child(Text::new("Line #4"))),
    );
    assert_eq!(
        output,
        lines(&["╭───────╮", "│Line #1│", "│Line #2│", "╰───────╯",])
    );
}

#[test]
fn overflow_y_box_above_top_edge_of_overflow_container() {
    let output = render(
        View::new().height(1).overflow_y(Overflow::Hidden).child(
            View::new()
                .margin_top(-2)
                .height(2)
                .flex_shrink(0.0)
                .child(Text::new("Hello\nWorld")),
        ),
    );
    assert_eq!(output, "");
}

#[test]
fn overflow_y_box_above_top_edge_of_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(7)
            .height(3)
            .overflow_y(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .margin_top(-3)
                    .height(2)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello\nWorld")),
            ),
    );
    assert_eq!(output, lines(&["╭─────╮", "│     │", "╰─────╯",]));
}

#[test]
fn overflow_y_box_intersecting_with_top_edge_of_overflow_container() {
    let output = render(
        View::new().height(1).overflow_y(Overflow::Hidden).child(
            View::new()
                .margin_top(-1)
                .height(2)
                .flex_shrink(0.0)
                .child(Text::new("Hello\nWorld")),
        ),
    );
    assert_eq!(output, "World");
}

#[test]
fn overflow_y_box_intersecting_with_top_edge_of_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(7)
            .height(3)
            .overflow_y(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .margin_top(-1)
                    .height(2)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello\nWorld")),
            ),
    );
    assert_eq!(output, lines(&["╭─────╮", "│World│", "╰─────╯",]));
}

#[test]
fn overflow_y_box_below_bottom_edge_of_overflow_container() {
    let output = render(
        View::new().height(1).overflow_y(Overflow::Hidden).child(
            View::new()
                .margin_top(1)
                .height(2)
                .flex_shrink(0.0)
                .child(Text::new("Hello\nWorld")),
        ),
    );
    assert_eq!(output, "");
}

#[test]
fn overflow_y_box_below_bottom_edge_of_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(7)
            .height(3)
            .overflow_y(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .margin_top(2)
                    .height(2)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello\nWorld")),
            ),
    );
    assert_eq!(output, lines(&["╭─────╮", "│     │", "╰─────╯",]));
}

#[test]
fn overflow_y_box_intersecting_with_bottom_edge_of_overflow_container() {
    let output = render(
        View::new().height(1).overflow_y(Overflow::Hidden).child(
            View::new()
                .height(2)
                .flex_shrink(0.0)
                .child(Text::new("Hello\nWorld")),
        ),
    );
    assert_eq!(output, "Hello");
}

#[test]
fn overflow_y_box_intersecting_with_bottom_edge_of_overflow_container_with_border() {
    let output = render(
        View::new()
            .width(7)
            .height(3)
            .overflow_y(Overflow::Hidden)
            .border_style(BorderStyle::Round)
            .child(
                View::new()
                    .height(2)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello\nWorld")),
            ),
    );
    assert_eq!(output, lines(&["╭─────╮", "│Hello│", "╰─────╯",]));
}

#[test]
fn overflow_x_visible_overrides_overflow_hidden_while_preserving_vertical_clipping() {
    let output = render(
        View::new().width(6).height(2).child(
            View::new()
                .width(3)
                .height(1)
                .overflow(Overflow::Hidden)
                .overflow_x(Overflow::Visible)
                .child(
                    View::new()
                        .width(6)
                        .height(2)
                        .flex_shrink(0.0)
                        .child(Text::new("ABCDEF\nGHIJKL")),
                ),
        ),
    );
    assert_eq!(output, lines(&["ABCDEF", "",]));
}

#[test]
fn overflow_y_visible_overrides_overflow_hidden_while_preserving_horizontal_clipping() {
    let output = render(
        View::new().width(6).height(2).child(
            View::new()
                .width(3)
                .height(1)
                .overflow(Overflow::Hidden)
                .overflow_y(Overflow::Visible)
                .child(
                    View::new()
                        .width(6)
                        .height(2)
                        .flex_shrink(0.0)
                        .child(Text::new("ABCDEF\nGHIJKL")),
                ),
        ),
    );
    assert_eq!(output, lines(&["ABC", "GHI",]));
}

#[test]
fn overflow_single_text_node_inside_overflow_container() {
    let output = render(
        View::new().padding_bottom(1).child(
            View::new()
                .width(6)
                .height(1)
                .overflow(Overflow::Hidden)
                .child(
                    View::new()
                        .width(12)
                        .height(2)
                        .flex_shrink(0.0)
                        .child(Text::new("Hello\nWorld")),
                ),
        ),
    );
    assert_eq!(output, lines(&["Hello", "",]));
}

#[test]
fn overflow_single_text_node_inside_overflow_container_with_border() {
    let output = render(
        View::new().padding_bottom(1).child(
            View::new()
                .width(8)
                .height(3)
                .overflow(Overflow::Hidden)
                .border_style(BorderStyle::Round)
                .child(
                    View::new()
                        .width(12)
                        .height(2)
                        .flex_shrink(0.0)
                        .child(Text::new("Hello\nWorld")),
                ),
        ),
    );
    assert_eq!(output, lines(&["╭──────╮", "│Hello │", "╰──────╯", "",]));
}

#[test]
fn overflow_multiple_boxes_inside_overflow_container() {
    let output = render(
        View::new().padding_bottom(1).child(
            View::new()
                .width(4)
                .height(1)
                .overflow(Overflow::Hidden)
                .child(
                    View::new()
                        .width(2)
                        .height(2)
                        .flex_shrink(0.0)
                        .child(Text::new("TL\nBL")),
                )
                .child(
                    View::new()
                        .width(2)
                        .height(2)
                        .flex_shrink(0.0)
                        .child(Text::new("TR\nBR")),
                ),
        ),
    );
    assert_eq!(output, lines(&["TLTR", "",]));
}

#[test]
fn overflow_multiple_boxes_inside_overflow_container_with_border() {
    let output = render(
        View::new().padding_bottom(1).child(
            View::new()
                .width(6)
                .height(3)
                .overflow(Overflow::Hidden)
                .border_style(BorderStyle::Round)
                .child(
                    View::new()
                        .width(2)
                        .height(2)
                        .flex_shrink(0.0)
                        .child(Text::new("TL\nBL")),
                )
                .child(
                    View::new()
                        .width(2)
                        .height(2)
                        .flex_shrink(0.0)
                        .child(Text::new("TR\nBR")),
                ),
        ),
    );
    assert_eq!(output, lines(&["╭────╮", "│TLTR│", "╰────╯", "",]));
}

#[test]
fn overflow_box_intersecting_with_top_left_edge_of_overflow_container() {
    let output = render(
        View::new()
            .width(4)
            .height(4)
            .overflow(Overflow::Hidden)
            .child(
                View::new()
                    .margin_top(-2)
                    .margin_left(-2)
                    .width(4)
                    .height(4)
                    .flex_shrink(0.0)
                    .child(Text::new("AAAA\nBBBB\nCCCC\nDDDD")),
            ),
    );
    assert_eq!(output, lines(&["CC", "DD", "", "",]));
}

#[test]
fn overflow_box_intersecting_with_top_right_edge_of_overflow_container() {
    let output = render(
        View::new()
            .width(4)
            .height(4)
            .overflow(Overflow::Hidden)
            .child(
                View::new()
                    .margin_top(-2)
                    .margin_left(2)
                    .width(4)
                    .height(4)
                    .flex_shrink(0.0)
                    .child(Text::new("AAAA\nBBBB\nCCCC\nDDDD")),
            ),
    );
    assert_eq!(output, lines(&["  CC", "  DD", "", "",]));
}

#[test]
fn overflow_box_intersecting_with_bottom_left_edge_of_overflow_container() {
    let output = render(
        View::new()
            .width(4)
            .height(4)
            .overflow(Overflow::Hidden)
            .child(
                View::new()
                    .margin_top(2)
                    .margin_left(-2)
                    .width(4)
                    .height(4)
                    .flex_shrink(0.0)
                    .child(Text::new("AAAA\nBBBB\nCCCC\nDDDD")),
            ),
    );
    assert_eq!(output, lines(&["", "", "AA", "BB",]));
}

#[test]
fn overflow_box_intersecting_with_bottom_right_edge_of_overflow_container() {
    let output = render(
        View::new()
            .width(4)
            .height(4)
            .overflow(Overflow::Hidden)
            .child(
                View::new()
                    .margin_top(2)
                    .margin_left(2)
                    .width(4)
                    .height(4)
                    .flex_shrink(0.0)
                    .child(Text::new("AAAA\nBBBB\nCCCC\nDDDD")),
            ),
    );
    assert_eq!(output, lines(&["", "", "  AA", "  BB",]));
}

#[test]
fn nested_overflow() {
    let output = render(
        View::new().padding_bottom(1).child(
            View::new()
                .width(4)
                .height(4)
                .overflow(Overflow::Hidden)
                .flex_direction(FlexDirection::Column)
                .child(
                    View::new()
                        .width(2)
                        .height(2)
                        .overflow(Overflow::Hidden)
                        .child(
                            View::new()
                                .width(4)
                                .height(4)
                                .flex_shrink(0.0)
                                .child(Text::new("AAAA\nBBBB\nCCCC\nDDDD")),
                        ),
                )
                .child(
                    View::new()
                        .width(4)
                        .height(3)
                        .child(Text::new("XXXX\nYYYY\nZZZZ")),
                ),
        ),
    );
    assert_eq!(output, lines(&["AA", "BB", "XXXX", "YYYY", "",]));
}

#[test]
fn out_of_bounds_writes_do_not_crash() {
    let output = render_cols(
        View::new()
            .width(12)
            .height(10)
            .border_style(BorderStyle::Round),
        10,
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────╮",
            "│         │",
            "│         │",
            "│         │",
            "│         │",
            "│         │",
            "│         │",
            "│         │",
            "│         │",
            "╰──────────╯",
        ])
    );
}
