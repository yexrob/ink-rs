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
fn equal_children_props_skip_component_render() {
    thread_local! {
        static PARENT_RENDERS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
        static RENDERS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    #[derive(PartialEq)]
    struct LayoutProps {
        title: String,
        children: Element,
    }
    fn label(_: &mut Scope, text: &&str) -> Element {
        Text::new(*text).into()
    }
    fn layout(_: &mut Scope, props: &LayoutProps) -> Element {
        RENDERS.set(RENDERS.get() + 1);
        View::new()
            .child(Text::new(&props.title))
            .child(props.children.clone())
            .into()
    }
    fn parent(_: &mut Scope, props: &(usize, &'static str)) -> Element {
        PARENT_RENDERS.set(PARENT_RENDERS.get() + 1);
        component(
            layout,
            LayoutProps {
                title: "Title: ".to_owned(),
                children: View::new()
                    .child(Text::new("child "))
                    .child(component(label, props.1).key("label"))
                    .into(),
            },
        )
    }
    PARENT_RENDERS.set(0);
    RENDERS.set(0);
    let mut instance = testing::render(component(parent, (0, "A")));
    assert_eq!(RENDERS.get(), 1);
    assert_ansi_eq(instance.last_frame(), "Title: child A");

    instance.rerender(component(parent, (1, "A")));
    assert_eq!(PARENT_RENDERS.get(), 2);
    assert_eq!(RENDERS.get(), 1);
    assert_ansi_eq(instance.last_frame(), "Title: child A");

    instance.rerender(component(parent, (2, "B")));
    assert_eq!(PARENT_RENDERS.get(), 3);
    assert_eq!(RENDERS.get(), 2);
    assert_ansi_eq(instance.last_frame(), "Title: child B");
}

#[test]
fn use_callback_skips_child_and_calls_latest_closure() {
    thread_local! {
        static RENDERS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
        static HANDLE: std::cell::RefCell<Option<Callback<(), usize>>> = const { std::cell::RefCell::new(None) };
    }
    #[derive(PartialEq)]
    struct ChildProps {
        on_change: Callback<(), usize>,
    }
    fn child(_: &mut Scope, props: &ChildProps) -> Element {
        RENDERS.set(RENDERS.get() + 1);
        HANDLE.with(|handle| *handle.borrow_mut() = Some(props.on_change.clone()));
        Text::new("value: ").into()
    }
    fn owner(cx: &mut Scope, value: &usize) -> Element {
        let shown = cx.use_state(|| 0);
        let value = *value;
        let on_change = cx.use_callback(move |()| {
            shown.set(value);
            value
        });
        View::new()
            .child(component(child, ChildProps { on_change }))
            .child(Text::new(shown.get().to_string()))
            .into()
    }
    RENDERS.set(0);
    let mut instance = testing::render(component(owner, 1));
    let callback = HANDLE.with(|handle| handle.borrow_mut().take().unwrap());
    assert_eq!(RENDERS.get(), 1);
    assert_ansi_eq(instance.last_frame(), "value: 0");
    assert_eq!(callback.call(()), 1);
    instance.settle();
    assert_eq!(RENDERS.get(), 1);
    assert_ansi_eq(instance.last_frame(), "value: 1");

    instance.rerender(component(owner, 2));
    assert_eq!(RENDERS.get(), 1);
    assert_ansi_eq(instance.last_frame(), "value: 1");
    assert_eq!(callback.call(()), 2);
    instance.settle();
    assert_eq!(RENDERS.get(), 1);
    assert_ansi_eq(instance.last_frame(), "value: 2");

    instance.unmount();
    assert_eq!(callback.call(()), 2);
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
