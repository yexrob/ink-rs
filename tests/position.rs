//! Ported from ink test/position.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

#[test]
fn absolute_position_with_top_and_left_offsets() {
    let output = render(
        View::new().width(5).height(3).child(
            View::new()
                .position(Position::Absolute)
                .top(1)
                .left(2)
                .child(Text::new("X")),
        ),
    );
    assert_eq!(output, lines(&["", "  X", "",]));
}

#[test]
fn absolute_position_with_bottom_and_right_offsets() {
    let output = render(
        View::new().width(6).height(4).child(
            View::new()
                .position(Position::Absolute)
                .bottom(1)
                .right(1)
                .child(Text::new("X")),
        ),
    );
    assert_eq!(output, lines(&["", "", "    X", "",]));
}

#[test]
fn renders_visible_lines_of_text_positioned_above_the_output_area() {
    let output = render(
        View::new().width(6).height(2).child(
            View::new()
                .position(Position::Absolute)
                .top(-1)
                .child(Text::new("Hidden\nFirst\nSecond")),
        ),
    );
    assert_eq!(output, lines(&["First", "Second",]));
}

#[test]
fn absolute_position_with_percentage_offsets() {
    let output = render(
        View::new().width(6).height(4).child(
            View::new()
                .position(Position::Absolute)
                .top(Percent(50.0))
                .left(Percent(50.0))
                .child(Text::new("X")),
        ),
    );
    assert_eq!(output, lines(&["", "", "   X", "",]));
}

#[test]
fn absolute_position_with_percentage_bottom_and_right_offsets() {
    let output = render(
        View::new().width(6).height(4).child(
            View::new()
                .position(Position::Absolute)
                .bottom(Percent(50.0))
                .right(Percent(50.0))
                .child(Text::new("X")),
        ),
    );
    assert_eq!(output, lines(&["", "  X", "", "",]));
}

#[test]
fn relative_position_offsets_visual_position_while_keeping_flow() {
    let output = render(
        View::new()
            .width(5)
            .child(
                View::new()
                    .position(Position::Relative)
                    .left(2)
                    .child(Text::new("A")),
            )
            .child(Text::new("B")),
    );
    assert_eq!(output, " BA");
}

#[test]
fn static_position_ignores_offsets() {
    let output = render(
        View::new()
            .width(5)
            .child(
                View::new()
                    .position(Position::Static)
                    .left(2)
                    .child(Text::new("A")),
            )
            .child(Text::new("B")),
    );
    assert_eq!(output, "AB");
}

#[test]
fn static_position_ignores_percentage_offsets() {
    let output = render(
        View::new()
            .width(5)
            .child(
                View::new()
                    .position(Position::Static)
                    .left(Percent(50.0))
                    .child(Text::new("A")),
            )
            .child(Text::new("B")),
    );
    assert_eq!(output, "AB");
}

#[test]
fn clears_top_offset_on_rerender() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let mut child = View::new().position(Position::Absolute).left(2);
        if *value {
            child = child.top(1);
        }
        View::new()
            .width(5)
            .height(3)
            .child(child.child(Text::new("X")))
            .into()
    }
    let mut instance = testing::render(component(example, true));
    assert_ansi_eq(instance.last_frame(), &lines(&["", "  X", ""]));
    instance.rerender(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["  X", "", ""]));
}

#[test]
fn clears_percentage_top_and_left_offsets_on_rerender() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let mut child = View::new().position(Position::Absolute);
        if *value {
            child = child.top(Percent(50.0)).left(Percent(50.0));
        }
        View::new()
            .width(6)
            .height(4)
            .child(child.child(Text::new("X")))
            .into()
    }
    let mut instance = testing::render(component(example, true));
    assert_ansi_eq(instance.last_frame(), &lines(&["", "", "   X", ""]));
    instance.rerender(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["X", "", "", ""]));
}

#[test]
fn clears_bottom_and_right_offsets_on_rerender() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let mut child = View::new().position(Position::Absolute);
        if *value {
            child = child.bottom(1).right(1);
        }
        View::new()
            .width(6)
            .height(4)
            .child(child.child(Text::new("X")))
            .into()
    }
    let mut instance = testing::render(component(example, true));
    assert_ansi_eq(instance.last_frame(), &lines(&["", "", "    X", ""]));
    instance.rerender(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["X", "", "", ""]));
}
