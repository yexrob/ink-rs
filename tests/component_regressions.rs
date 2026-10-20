//! Ported from ink test/component-regressions.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn changing_nested_transform_wrap_points_updates_layout_concurrent_false_deeply_nested_false() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let replace = *value;
        let content = Transform::new(move |text, _| {
            if replace {
                text.replace(' ', "x")
            } else {
                text.to_owned()
            }
        })
        .child(Text::new("ab cd ef"));
        let text = Text::new(content);
        View::new()
            .flex_direction(FlexDirection::Column)
            .width(4)
            .child(text)
            .child(Text::new("!"))
            .into()
    }
    let mut instance = testing::render(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["ab", "cd", "ef", "!"]));
    instance.rerender(component(example, true));
    assert_ansi_eq(instance.last_frame(), &lines(&["abxc", "dxef", "!"]));
    instance.rerender(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["ab", "cd", "ef", "!"]));
}

#[test]
fn changing_nested_transform_wrap_points_updates_layout_concurrent_false_deeply_nested_true() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let replace = *value;
        let content = Transform::new(move |text, _| {
            if replace {
                text.replace(' ', "x")
            } else {
                text.to_owned()
            }
        })
        .child(Text::new("ab cd ef"));
        let text = Text::new(Text::new(content));
        View::new()
            .flex_direction(FlexDirection::Column)
            .width(4)
            .child(text)
            .child(Text::new("!"))
            .into()
    }
    let mut instance = testing::render(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["ab", "cd", "ef", "!"]));
    instance.rerender(component(example, true));
    assert_ansi_eq(instance.last_frame(), &lines(&["abxc", "dxef", "!"]));
    instance.rerender(component(example, false));
    assert_ansi_eq(instance.last_frame(), &lines(&["ab", "cd", "ef", "!"]));
}
