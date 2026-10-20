//! Ported from ink test/flex-justify-content.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

#[test]
fn row_align_text_to_center() {
    let output = render(
        View::new()
            .justify_content(JustifyContent::Center)
            .width(10)
            .child(Text::new("Test")),
    );
    assert_eq!(output, "   Test");
}

#[test]
fn row_align_multiple_text_nodes_to_center() {
    let output = render(
        View::new()
            .justify_content(JustifyContent::Center)
            .width(10)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "    AB");
}

#[test]
fn row_align_text_to_right() {
    let output = render(
        View::new()
            .justify_content(JustifyContent::FlexEnd)
            .width(10)
            .child(Text::new("Test")),
    );
    assert_eq!(output, "      Test");
}

#[test]
fn row_align_multiple_text_nodes_to_right() {
    let output = render(
        View::new()
            .justify_content(JustifyContent::FlexEnd)
            .width(10)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "        AB");
}

#[test]
fn row_align_two_text_nodes_on_the_edges() {
    let output = render(
        View::new()
            .justify_content(JustifyContent::SpaceBetween)
            .width(4)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A  B");
}

#[test]
fn row_space_evenly_two_text_nodes() {
    let output = render(
        View::new()
            .justify_content(JustifyContent::SpaceEvenly)
            .width(10)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "  A   B");
}

#[test]
#[ignore = "skipped upstream: Yoga bug with `space-around` and text nodes"]
fn row_align_two_text_nodes_with_equal_space_around_them() {
    let output = render(
        View::new()
            .justify_content(JustifyContent::SpaceAround)
            .width(5)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, " A B");
}

#[test]
fn row_align_colored_text_node_when_text_is_squashed() {
    let output = render(
        View::new()
            .justify_content(JustifyContent::FlexEnd)
            .width(5)
            .child(Text::new("X").color(Color::Green)),
    );
    assert_ansi_eq(&output, &styled(&["    {fg=green}X{/}"]));
}

#[test]
fn column_align_text_to_center() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .height(3)
            .child(Text::new("Test")),
    );
    assert_eq!(output, lines(&["", "Test", "",]));
}

#[test]
fn column_align_text_to_bottom() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::FlexEnd)
            .height(3)
            .child(Text::new("Test")),
    );
    assert_eq!(output, lines(&["", "", "Test",]));
}

#[test]
fn column_align_two_text_nodes_on_the_edges() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::SpaceBetween)
            .height(4)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "", "", "B",]));
}

#[test]
#[ignore = "skipped upstream: Yoga bug with `space-around` and text nodes"]
fn column_align_two_text_nodes_with_equal_space_around_them() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::SpaceAround)
            .height(5)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "\nA\n\nB\n");
}
