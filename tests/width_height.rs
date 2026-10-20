//! Ported from ink test/width-height.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

#[test]
fn set_width() {
    let output = render(
        View::new()
            .child(View::new().width(5).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A    B");
}

#[test]
fn set_width_in_percent() {
    let output = render(
        View::new()
            .width(10)
            .child(View::new().width(Percent(50.0)).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A    B");
}

#[test]
fn set_width_in_fractional_percent() {
    let output = render_cols(
        View::new()
            .width(200)
            .child(View::new().width(Percent(12.5)).child(Text::new("A")))
            .child(Text::new("B")),
        200,
    );
    assert_eq!(output, "A                        B");
}

#[test]
fn set_min_width() {
    let smaller_output = render(
        View::new()
            .child(View::new().min_width(5).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(smaller_output, "A    B");
    let larger_output = render(
        View::new()
            .child(View::new().min_width(2).child(Text::new("AAAAA")))
            .child(Text::new("B")),
    );
    assert_eq!(larger_output, "AAAAAB");
}

#[test]
fn set_min_width_in_percent() {
    let output = render(
        View::new()
            .width(10)
            .child(View::new().min_width(Percent(50.0)).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A    B");
}

#[test]
fn set_width_to_zero() {
    let output = render(
        View::new()
            .child(View::new().width(0).child(Text::new("hello")))
            .child(Text::new("|")),
    );
    assert_eq!(output, "|ello");
}

#[test]
fn set_width_to_zero_with_text_below() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .child(View::new().width(0).child(Text::new("hello")))
                    .child(Text::new("|")),
            )
            .child(Text::new("next")),
    );
    assert_eq!(output, lines(&["|ello", "next",]));
}

#[test]
fn padding_leaves_no_room_for_text() {
    let output = render(
        View::new()
            .child(View::new().width(2).padding_x(1).child(Text::new("hello")))
            .child(Text::new("|")),
    );
    assert_eq!(output, " h|llo");
}

#[test]
fn shrink_text_to_zero_width() {
    let output = render(
        View::new()
            .width(5)
            .child(
                View::new()
                    .flex_basis(0)
                    .flex_shrink(1.0)
                    .child(Text::new("hello")),
            )
            .child(View::new().width(5).child(Text::new("world"))),
    );
    assert_eq!(output, "world");
}

#[test]
fn set_height() {
    let output = render(
        View::new()
            .height(4)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["AB", "", "", "",]));
}

#[test]
fn set_height_in_percent() {
    let output = render(
        View::new()
            .height(6)
            .flex_direction(FlexDirection::Column)
            .child(View::new().height(Percent(50.0)).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "", "", "B", "", "",]));
}

#[test]
fn cut_text_over_the_set_height() {
    let output = render_cols(View::new().height(2).child(Text::new("AAAABBBBCCCC")), 4);
    assert_eq!(output, lines(&["AAAA", "BBBB",]));
}

#[test]
fn set_min_height() {
    let smaller_output = render(View::new().min_height(4).child(Text::new("A")));
    assert_eq!(smaller_output, lines(&["A", "", "", "",]));
    let larger_output = render(
        View::new()
            .min_height(2)
            .child(View::new().height(4).child(Text::new("A"))),
    );
    assert_eq!(larger_output, lines(&["A", "", "", "",]));
}

#[test]
fn set_min_height_in_percent() {
    let output = render(
        View::new()
            .height(6)
            .flex_direction(FlexDirection::Column)
            .child(View::new().min_height(Percent(50.0)).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "", "", "B", "", "",]));
}

#[test]
fn set_max_width() {
    let constrained_output = render_cols(
        View::new()
            .child(View::new().max_width(3).child(Text::new("AAAAA")))
            .child(Text::new("B")),
        10,
    );
    assert_eq!(constrained_output, lines(&["AAAB", "AA",]));
    let unconstrained_output = render(
        View::new()
            .child(View::new().max_width(10).child(Text::new("AAA")))
            .child(Text::new("B")),
    );
    assert_eq!(unconstrained_output, "AAAB");
}

#[test]
fn clears_max_width_on_rerender() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let mut child = View::new();
        if *value {
            child = child.max_width(3);
        }
        View::new()
            .child(child.child(Text::new("AAAAA")))
            .child(Text::new("B"))
            .into()
    }
    let mut instance = testing::render(component(example, true));
    assert_ansi_eq(instance.last_frame(), &lines(&["AAAB", "AA"]));
    instance.rerender(component(example, false));
    assert_ansi_eq(instance.last_frame(), "AAAAAB");
}

#[test]
fn set_max_height() {
    let constrained_output = render(
        View::new()
            .max_height(2)
            .child(View::new().height(4).child(Text::new("A"))),
    );
    assert_eq!(constrained_output, lines(&["A", "",]));
    let unconstrained_output = render(View::new().max_height(4).child(Text::new("A")));
    assert_eq!(unconstrained_output, "A");
}

#[test]
fn clears_max_height_on_rerender() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let mut view = View::new();
        if *value {
            view = view.max_height(2);
        }
        view.child(View::new().height(4).child(Text::new("A")))
            .into()
    }
    let mut instance = testing::render(component(example, true));
    assert_ansi_eq(instance.last_frame(), &lines(&["A", ""]));
    instance.rerender(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["A", "", "", ""]));
}

#[test]
fn set_aspect_ratio_with_width() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .width(8)
                    .aspect_ratio(2.0)
                    .border_style(BorderStyle::Single)
                    .child(Text::new("X")),
            )
            .child(Text::new("Y")),
    );
    assert_eq!(
        output,
        lines(&["┌──────┐", "│X     │", "│      │", "└──────┘", "Y",])
    );
}

#[test]
fn set_aspect_ratio_with_height() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .height(3)
                    .aspect_ratio(2.0)
                    .border_style(BorderStyle::Single)
                    .child(Text::new("X")),
            )
            .child(Text::new("Y")),
    );
    assert_eq!(output, lines(&["┌────┐", "│X   │", "└────┘", "Y",]));
}

#[test]
fn set_aspect_ratio_with_width_and_height() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .width(8)
                    .height(3)
                    .aspect_ratio(2.0)
                    .border_style(BorderStyle::Single)
                    .child(Text::new("X")),
            )
            .child(Text::new("Y")),
    );
    assert_eq!(output, lines(&["┌────┐", "│X   │", "└────┘", "Y",]));
}

#[test]
fn set_aspect_ratio_with_max_height_constraint() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .width(10)
                    .max_height(3)
                    .aspect_ratio(2.0)
                    .border_style(BorderStyle::Single)
                    .child(Text::new("X")),
            )
            .child(Text::new("Y")),
    );
    assert_eq!(output, lines(&["┌────┐", "│X   │", "└────┘", "Y",]));
}

#[test]
fn clears_aspect_ratio_on_rerender() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let mut child = View::new().width(8).border_style(BorderStyle::Single);
        if *value {
            child = child.aspect_ratio(2.0);
        }
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(child.child(Text::new("X")))
            .child(Text::new("Y"))
            .into()
    }
    let mut instance = testing::render(component(example, true));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&["┌──────┐", "│X     │", "│      │", "└──────┘", "Y"]),
    );
    instance.rerender(component(example, false));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&["┌──────┐", "│X     │", "└──────┘", "Y"]),
    );
}

#[test]
#[ignore = "skipped upstream: Yoga does not support percentage max width"]
fn set_max_width_in_percent() {
    let output = render(
        View::new()
            .width(10)
            .child(
                View::new()
                    .max_width(Percent(50.0))
                    .child(Text::new("AAAAAAAAAA")),
            )
            .child(Text::new("B")),
    );
    assert_eq!(output, "AAAAAB");
}

#[test]
fn set_max_height_in_percent() {
    let output = render(
        View::new()
            .height(6)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .max_height(Percent(50.0))
                    .child(View::new().height(6).child(Text::new("A"))),
            )
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "", "", "B", "", "",]));
}
