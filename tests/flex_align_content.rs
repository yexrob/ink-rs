//! Ported from ink test/flex-align-content.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn align_content_flex_start() {
    let output = render(
        View::new()
            .width(2)
            .height(6)
            .flex_wrap(FlexWrap::Wrap)
            .align_content(AlignContent::FlexStart)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["AB", "CD", "", "", "", "",]));
}

#[test]
fn align_content_center() {
    let output = render(
        View::new()
            .width(2)
            .height(6)
            .flex_wrap(FlexWrap::Wrap)
            .align_content(AlignContent::Center)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["", "", "AB", "CD", "", "",]));
}

#[test]
fn align_content_flex_end() {
    let output = render(
        View::new()
            .width(2)
            .height(6)
            .flex_wrap(FlexWrap::Wrap)
            .align_content(AlignContent::FlexEnd)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["", "", "", "", "AB", "CD",]));
}

#[test]
fn align_content_space_between() {
    let output = render(
        View::new()
            .width(2)
            .height(6)
            .flex_wrap(FlexWrap::Wrap)
            .align_content(AlignContent::SpaceBetween)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["AB", "", "", "", "", "CD",]));
}

#[test]
fn align_content_space_around() {
    let output = render(
        View::new()
            .width(2)
            .height(6)
            .flex_wrap(FlexWrap::Wrap)
            .align_content(AlignContent::SpaceAround)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["", "AB", "", "", "CD", "",]));
}

#[test]
fn align_content_space_evenly() {
    let output = render(
        View::new()
            .width(2)
            .height(6)
            .flex_wrap(FlexWrap::Wrap)
            .align_content(AlignContent::SpaceEvenly)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["", "AB", "", "CD", "", "",]));
}

#[test]
fn align_content_stretch() {
    let output = render(
        View::new()
            .width(2)
            .height(6)
            .flex_wrap(FlexWrap::Wrap)
            .align_content(AlignContent::Stretch)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["AB", "", "", "CD", "", "",]));
}

#[test]
fn align_content_defaults_to_flex_start() {
    let output = render(
        View::new()
            .width(2)
            .height(6)
            .flex_wrap(FlexWrap::Wrap)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["AB", "CD", "", "", "", "",]));
}

#[test]
fn align_content_does_not_add_extra_spacing_when_there_is_no_free_cross_axis_space() {
    let output = render(
        View::new()
            .width(2)
            .height(2)
            .flex_wrap(FlexWrap::Wrap)
            .align_content(AlignContent::Center)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C"))
            .child(Text::new("D")),
    );
    assert_eq!(output, lines(&["AB", "CD",]));
}
#[test]
fn clears_align_content_on_rerender_to_default_flex_start() {
    fn example(_: &mut Scope, value: &Option<AlignContent>) -> Element {
        let mut view = View::new().width(2).height(6).flex_wrap(FlexWrap::Wrap);
        if let Some(align) = value {
            view = view.align_content(*align);
        }
        view.children(["A", "B", "C", "D"].map(Text::new)).into()
    }
    let mut instance = testing::render(component(example, Some(AlignContent::Center)));
    assert_ansi_eq(instance.last_frame(), &lines(&["", "", "AB", "CD", "", ""]));
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), &lines(&["AB", "CD", "", "", "", ""]));
}

#[test]
fn clears_align_content_from_stretch_on_rerender_to_default_flex_start() {
    fn example(_: &mut Scope, value: &Option<AlignContent>) -> Element {
        let mut view = View::new().width(2).height(6).flex_wrap(FlexWrap::Wrap);
        if let Some(align) = value {
            view = view.align_content(*align);
        }
        view.children(["A", "B", "C", "D"].map(Text::new)).into()
    }
    let mut instance = testing::render(component(example, Some(AlignContent::Stretch)));
    assert_ansi_eq(instance.last_frame(), &lines(&["AB", "", "", "CD", "", ""]));
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), &lines(&["AB", "CD", "", "", "", ""]));
}
