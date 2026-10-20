//! Ported from ink test/reconciler.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn update_child() {
    fn example(_: &mut Scope, update: &bool) -> Element {
        Text::new(if *update { "B" } else { "A" }).into()
    }
    let mut actual = testing::render(component(example, false));
    let mut expected = testing::render(Text::new("A"));
    assert_eq!(actual.last_frame(), expected.last_frame());
    actual.rerender(component(example, true));
    expected.rerender(Text::new("B"));
    assert_eq!(actual.last_frame(), expected.last_frame());
}

#[test]
fn update_text_node() {
    fn example(_: &mut Scope, update: &bool) -> Element {
        let labels: &[&str] = if *update {
            &["Hello ", "B"]
        } else {
            &["Hello ", "A"]
        };
        View::new()
            .flex_direction(FlexDirection::Row)
            .children(labels.iter().map(|label| {
                let element: Element = Text::new(*label).into();
                element
            }))
            .into()
    }
    let mut actual = testing::render(component(example, false));
    let mut expected = testing::render(Text::new("Hello A"));
    assert_eq!(actual.last_frame(), expected.last_frame());
    actual.rerender(component(example, true));
    expected.rerender(Text::new("Hello B"));
    assert_eq!(actual.last_frame(), expected.last_frame());
}

#[test]
fn remove_style_prop_from_intrinsic_node() {
    fn example(_: &mut Scope, value: &bool) -> Element {
        let mut view = View::new();
        if *value {
            view = view.margin_left(1);
        }
        view.child(Text::new("X")).into()
    }
    let mut instance = testing::render(component(example, true));
    assert_ansi_eq(instance.last_frame(), " X");
    instance.rerender(component(example, false));
    assert_ansi_eq(instance.last_frame(), "X");
}

#[test]
fn append_child() {
    fn example(_: &mut Scope, update: &bool) -> Element {
        let labels: &[&str] = if *update { &["A", "B"] } else { &["A"] };
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(labels.iter().map(|label| {
                let element: Element = Text::new(*label).into();
                element
            }))
            .into()
    }
    let mut actual = testing::render(component(example, false));
    let mut expected = testing::render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(["A"].map(Text::new)),
    );
    assert_eq!(actual.last_frame(), expected.last_frame());
    actual.rerender(component(example, true));
    expected.rerender(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(["A", "B"].map(Text::new)),
    );
    assert_eq!(actual.last_frame(), expected.last_frame());
}

#[test]
fn insert_child_between_other_children() {
    fn example(_: &mut Scope, update: &bool) -> Element {
        let labels: &[&str] = if *update {
            &["A", "B", "C"]
        } else {
            &["A", "C"]
        };
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(labels.iter().map(|label| {
                let element: Element = Text::new(*label).into();
                element.key(*label)
            }))
            .into()
    }
    let mut actual = testing::render(component(example, false));
    let mut expected = testing::render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(["A", "C"].map(Text::new)),
    );
    assert_eq!(actual.last_frame(), expected.last_frame());
    actual.rerender(component(example, true));
    expected.rerender(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(["A", "B", "C"].map(Text::new)),
    );
    assert_eq!(actual.last_frame(), expected.last_frame());
}

#[test]
fn remove_child() {
    fn example(_: &mut Scope, update: &bool) -> Element {
        let labels: &[&str] = if *update { &["A"] } else { &["A", "B"] };
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(labels.iter().map(|label| {
                let element: Element = Text::new(*label).into();
                element
            }))
            .into()
    }
    let mut actual = testing::render(component(example, false));
    let mut expected = testing::render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(["A", "B"].map(Text::new)),
    );
    assert_eq!(actual.last_frame(), expected.last_frame());
    actual.rerender(component(example, true));
    expected.rerender(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(["A"].map(Text::new)),
    );
    assert_eq!(actual.last_frame(), expected.last_frame());
}

#[test]
fn reorder_children() {
    fn example(_: &mut Scope, update: &bool) -> Element {
        let labels: &[&str] = if *update { &["B", "A"] } else { &["A", "B"] };
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(labels.iter().map(|label| {
                let element: Element = Text::new(*label).into();
                element.key(*label)
            }))
            .into()
    }
    let mut actual = testing::render(component(example, false));
    let mut expected = testing::render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(["A", "B"].map(Text::new)),
    );
    assert_eq!(actual.last_frame(), expected.last_frame());
    actual.rerender(component(example, true));
    expected.rerender(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(["B", "A"].map(Text::new)),
    );
    assert_eq!(actual.last_frame(), expected.last_frame());
}
