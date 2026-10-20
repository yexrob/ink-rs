//! Ported from ink test/flex-wrap.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

#[test]
fn row_no_wrap() {
    let output = render(
        View::new()
            .width(2)
            .child(Text::new("A"))
            .child(Text::new("BC")),
    );
    assert_eq!(output, lines(&["BC", "",]));
}

#[test]
fn column_no_wrap() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .height(2)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&["B", "C",]));
}

#[test]
fn row_wrap_content() {
    let output = render(
        View::new()
            .width(2)
            .flex_wrap(FlexWrap::Wrap)
            .child(Text::new("A"))
            .child(Text::new("BC")),
    );
    assert_eq!(output, lines(&["A", "BC",]));
}

#[test]
fn setting_wrap_to_undefined_restores_nowrap() {
    fn example(_: &mut Scope, value: &Option<FlexWrap>) -> Element {
        let mut view = View::new().width(3);
        if let Some(wrap) = value {
            view = view.flex_wrap(*wrap);
        }
        view.child(View::new().flex_shrink(0.0).child(Text::new("AB")))
            .child(View::new().flex_shrink(0.0).child(Text::new("CD")))
            .into()
    }
    let mut instance = testing::render(component(example, Some(FlexWrap::Wrap)));
    assert_ansi_eq(instance.last_frame(), &lines(&["AB", "CD"]));
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), "ABCD");
}

#[test]
fn setting_wrap_reverse_to_undefined_restores_nowrap() {
    fn example(_: &mut Scope, value: &Option<FlexWrap>) -> Element {
        let mut view = View::new().width(3);
        if let Some(wrap) = value {
            view = view.flex_wrap(*wrap);
        }
        view.child(View::new().flex_shrink(0.0).child(Text::new("AB")))
            .child(View::new().flex_shrink(0.0).child(Text::new("CD")))
            .into()
    }
    let mut instance = testing::render(component(example, Some(FlexWrap::WrapReverse)));
    assert_ansi_eq(instance.last_frame(), &lines(&["CD", "AB"]));
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), "ABCD");
}

#[test]
fn column_wrap_content() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .height(2)
            .flex_wrap(FlexWrap::Wrap)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&["AC", "B",]));
}

#[test]
fn column_wrap_content_reverse() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .height(2)
            .width(3)
            .flex_wrap(FlexWrap::WrapReverse)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&[" CA", "  B",]));
}

#[test]
fn row_wrap_content_reverse() {
    let output = render(
        View::new()
            .height(3)
            .width(2)
            .flex_wrap(FlexWrap::WrapReverse)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&["", "C", "AB",]));
}
