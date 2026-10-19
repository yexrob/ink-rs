//! Ported from ink test/content-offset.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn content_offset_y_scrolls_content_up() {
    let output = render(
        View::new()
            .height(2)
            .overflow_y(Overflow::Hidden)
            .content_offset_y(1)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(Text::new("Line 1"))
                    .child(Text::new("Line 2"))
                    .child(Text::new("Line 3")),
            ),
    );
    assert_eq!(output, lines(&["Line 2", "Line 3",]));
}

#[test]
fn content_offset_y_zero_offset_renders_content_unchanged() {
    let output = render(
        View::new()
            .height(2)
            .overflow_y(Overflow::Hidden)
            .content_offset_y(0)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(Text::new("Line 1"))
                    .child(Text::new("Line 2"))
                    .child(Text::new("Line 3")),
            ),
    );
    assert_eq!(output, lines(&["Line 1", "Line 2",]));
}

#[test]
fn content_offset_y_offset_beyond_content_renders_empty() {
    let output = render(
        View::new()
            .height(2)
            .overflow_y(Overflow::Hidden)
            .content_offset_y(5)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(Text::new("Line 1"))
                    .child(Text::new("Line 2")),
            ),
    );
    assert_eq!(output, lines(&["", "",]));
}

#[test]
fn content_offset_y_negative_offset_shifts_content_down() {
    let output = render(
        View::new()
            .height(2)
            .overflow_y(Overflow::Hidden)
            .content_offset_y(-1)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(Text::new("Line 1"))
                    .child(Text::new("Line 2")),
            ),
    );
    assert_eq!(output, lines(&["", "Line 1",]));
}

#[test]
fn content_offset_x_scrolls_content_left() {
    let output = render(
        View::new()
            .width(5)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(6)
            .child(
                View::new()
                    .width(11)
                    .flex_shrink(0.0)
                    .child(Text::new("Hello World")),
            ),
    );
    assert_eq!(output, "World");
}

#[test]
fn content_offset_x_and_content_offset_y_scroll_both_directions() {
    let output = render(
        View::new()
            .width(3)
            .height(2)
            .overflow(Overflow::Hidden)
            .content_offset_x(2)
            .content_offset_y(1)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .width(5)
                    .child(Text::new("aaaaa"))
                    .child(Text::new("bbbbb"))
                    .child(Text::new("ccccc")),
            ),
    );
    assert_eq!(output, lines(&["bbb", "ccc",]));
}

#[test]
fn content_offset_y_content_stays_inside_border() {
    let output = render(
        View::new()
            .height(4)
            .width(8)
            .overflow_y(Overflow::Hidden)
            .content_offset_y(1)
            .border_style(BorderStyle::Round)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(Text::new("Line 1"))
                    .child(Text::new("Line 2"))
                    .child(Text::new("Line 3")),
            ),
    );
    assert_eq!(
        output,
        lines(&["╭──────╮", "│Line 2│", "│Line 3│", "╰──────╯",])
    );
}

#[test]
fn content_offset_y_nested_offset_containers_compose() {
    let output = render(
        View::new()
            .height(2)
            .overflow_y(Overflow::Hidden)
            .content_offset_y(1)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .height(4)
                    .overflow_y(Overflow::Hidden)
                    .content_offset_y(1)
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(
                        View::new()
                            .flex_direction(FlexDirection::Column)
                            .flex_shrink(0.0)
                            .child(Text::new("Line 1"))
                            .child(Text::new("Line 2"))
                            .child(Text::new("Line 3"))
                            .child(Text::new("Line 4")),
                    ),
            ),
    );
    assert_eq!(output, lines(&["Line 3", "Line 4",]));
}

#[test]
fn nested_clipped_content_stays_inside_the_outer_clip() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("ABOVE"))
            .child(
                View::new()
                    .height(2)
                    .overflow_y(Overflow::Hidden)
                    .flex_direction(FlexDirection::Column)
                    .child(
                        View::new()
                            .height(4)
                            .margin_top(-2)
                            .overflow_y(Overflow::Hidden)
                            .flex_direction(FlexDirection::Column)
                            .flex_shrink(0.0)
                            .child(Text::new("Line 1"))
                            .child(Text::new("Line 2"))
                            .child(Text::new("Line 3"))
                            .child(Text::new("Line 4")),
                    ),
            ),
    );
    assert_eq!(output, lines(&["ABOVE", "Line 3", "Line 4",]));
}

#[test]
fn nested_clips_on_different_axes_both_apply() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("HEADER"))
            .child(
                View::new()
                    .height(2)
                    .overflow_y(Overflow::Hidden)
                    .flex_direction(FlexDirection::Column)
                    .child(
                        View::new()
                            .width(3)
                            .overflow_x(Overflow::Hidden)
                            .margin_top(-1)
                            .flex_direction(FlexDirection::Column)
                            .flex_shrink(0.0)
                            .child(Text::new("AAAAAA"))
                            .child(Text::new("BBBBBB"))
                            .child(Text::new("CCCCCC")),
                    ),
            ),
    );
    assert_eq!(output, lines(&["HEADER", "AAA", "BBB",]));
}

#[test]
fn disjoint_nested_clips_render_nothing() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("HEADER"))
            .child(
                View::new().width(5).overflow_x(Overflow::Hidden).child(
                    View::new()
                        .margin_left(10)
                        .width(10)
                        .overflow_x(Overflow::Hidden)
                        .flex_shrink(0.0)
                        .child(Text::new("XXXXXXXXXX")),
                ),
            ),
    );
    assert_eq!(output, lines(&["HEADER", "",]));
}

#[test]
fn disjoint_nested_clips_do_not_overwrite_content_when_a_wide_character_straddles_the_edge() {
    let output = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("XXXXXXXXXX"))
            .child(
                View::new()
                    .margin_top(-1)
                    .width(5)
                    .overflow_x(Overflow::Hidden)
                    .flex_shrink(0.0)
                    .child(
                        View::new()
                            .margin_left(6)
                            .width(5)
                            .overflow_x(Overflow::Hidden)
                            .content_offset_x(1)
                            .flex_shrink(0.0)
                            .child(View::new().flex_shrink(0.0).child(Text::new("你AAAA"))),
                    ),
            ),
        20,
    );
    assert_eq!(output, lines(&["XXXXXXXXXX", "",]));
}

#[test]
fn content_offset_y_nested_clipped_content_does_not_escape_the_outer_viewport() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("ABOVE"))
            .child(
                View::new()
                    .height(2)
                    .overflow_y(Overflow::Hidden)
                    .content_offset_y(1)
                    .flex_direction(FlexDirection::Column)
                    .child(
                        View::new()
                            .height(4)
                            .overflow_y(Overflow::Hidden)
                            .flex_direction(FlexDirection::Column)
                            .flex_shrink(0.0)
                            .child(Text::new("Line 1"))
                            .child(Text::new("Line 2"))
                            .child(Text::new("Line 3"))
                            .child(Text::new("Line 4")),
                    ),
            ),
    );
    assert_eq!(output, lines(&["ABOVE", "Line 2", "Line 3",]));
}

#[test]
fn content_offset_x_preserves_columns_when_offset_splits_a_wide_character() {
    let output = render(
        View::new()
            .width(2)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(1)
            .child(
                View::new()
                    .width(3)
                    .flex_shrink(0.0)
                    .child(Text::new("你A")),
            ),
    );
    assert_eq!(output, " A");
    let output = render(
        View::new()
            .width(2)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(2)
            .child(
                View::new()
                    .width(3)
                    .flex_shrink(0.0)
                    .child(Text::new("你A")),
            ),
    );
    assert_eq!(output, "A");
}

#[test]
fn content_offset_x_preserves_columns_across_a_line_of_wide_characters_offset_0() {
    let output = render(
        View::new()
            .width(4)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(0)
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("你好AB")),
            ),
    );
    assert_eq!(output, "你好");
}

#[test]
fn content_offset_x_preserves_columns_across_a_line_of_wide_characters_offset_1() {
    let output = render(
        View::new()
            .width(4)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(1)
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("你好AB")),
            ),
    );
    assert_eq!(output, " 好A");
}

#[test]
fn content_offset_x_preserves_columns_across_a_line_of_wide_characters_offset_2() {
    let output = render(
        View::new()
            .width(4)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(2)
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("你好AB")),
            ),
    );
    assert_eq!(output, "好AB");
}

#[test]
fn content_offset_x_preserves_columns_across_a_line_of_wide_characters_offset_3() {
    let output = render(
        View::new()
            .width(4)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(3)
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("你好AB")),
            ),
    );
    assert_eq!(output, " AB");
}

#[test]
fn content_offset_x_preserves_columns_across_a_line_of_wide_characters_offset_4() {
    let output = render(
        View::new()
            .width(4)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(4)
            .child(
                View::new()
                    .width(6)
                    .flex_shrink(0.0)
                    .child(Text::new("你好AB")),
            ),
    );
    assert_eq!(output, "AB");
}

#[test]
fn clipping_blanks_the_cell_of_a_wide_character_split_by_the_right_edge() {
    let output = render(
        View::new().child(Text::new("XY")).child(
            View::new()
                .position(Position::Absolute)
                .width(2)
                .overflow_x(Overflow::Hidden)
                .child(
                    View::new()
                        .width(3)
                        .flex_shrink(0.0)
                        .child(Text::new("A你")),
                ),
        ),
    );
    assert_eq!(output, "A");
}

#[test]
fn horizontal_scrolling_preserves_borders_beside_fully_clipped_rows() {
    let output = render(
        View::new()
            .width(4)
            .height(4)
            .border_style(BorderStyle::Single)
            .overflow(Overflow::Hidden)
            .content_offset_x(6)
            .child(
                View::new()
                    .width(10)
                    .flex_shrink(0.0)
                    .child(Text::new("A\nABCDEFGHIJ")),
            ),
    );
    assert_eq!(output, lines(&["┌──┐", "│  │", "│GH│", "└──┘",]));
}

#[test]
fn content_offset_y_keeps_the_visible_lines_of_a_multi_line_text_node() {
    let output = render(
        View::new()
            .height(2)
            .content_offset_y(1)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(Text::new("Line 1\nLine 2\nLine 3")),
            ),
    );
    assert_eq!(output, lines(&["Line 2", "Line 3",]));
}

#[test]
fn content_offset_y_keeps_the_visible_lines_of_a_multi_line_text_node_with_only_horizontal_clipping()
 {
    let output = render(
        View::new()
            .height(2)
            .overflow_x(Overflow::Hidden)
            .content_offset_y(1)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_shrink(0.0)
                    .child(Text::new("Line 1\nLine 2\nLine 3")),
            ),
    );
    assert_eq!(output, lines(&["Line 2", "Line 3",]));
}

#[test]
fn clipped_content_keeps_its_visible_lines_when_the_clip_starts_above_the_top() {
    let output = render(
        View::new().flex_direction(FlexDirection::Column).child(
            View::new()
                .margin_top(-1)
                .height(3)
                .overflow_y(Overflow::Hidden)
                .flex_direction(FlexDirection::Column)
                .child(
                    View::new()
                        .flex_direction(FlexDirection::Column)
                        .flex_shrink(0.0)
                        .child(Text::new("Line 1\nLine 2\nLine 3")),
                ),
        ),
    );
    assert_eq!(output, lines(&["Line 2", "Line 3",]));
}
