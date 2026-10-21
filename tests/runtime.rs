//! Rust-native acceptance tests for the stepping runtime.

use std::cell::Cell;

use ink_rs::{
    Element, FlexDirection, Scope, State, Static, Styled, Text, View, component, testing,
};

thread_local! {
    static RENDERS: Cell<usize> = const { Cell::new(0) };
    static CHILD_RENDERS: Cell<usize> = const { Cell::new(0) };
    static VALUE: Cell<Option<State<usize>>> = const { Cell::new(None) };
}

fn counter(cx: &mut Scope, _: &()) -> Element {
    RENDERS.set(RENDERS.get() + 1);
    let value = cx.use_state(|| 0);
    VALUE.set(Some(value));
    Text::new(value.get().to_string()).into()
}

#[test]
fn parent_update_skips_child_with_equal_props() {
    fn child(_: &mut Scope, _: &()) -> Element {
        CHILD_RENDERS.set(CHILD_RENDERS.get() + 1);
        Text::new("child").into()
    }
    fn parent(cx: &mut Scope, _: &()) -> Element {
        RENDERS.set(RENDERS.get() + 1);
        VALUE.set(Some(cx.use_state(|| 0)));
        component(child, ())
    }
    RENDERS.set(0);
    CHILD_RENDERS.set(0);
    let mut instance = testing::render(component(parent, ()));
    VALUE.get().unwrap().set(1);
    instance.settle();
    assert_eq!(RENDERS.get(), 2);
    assert_eq!(CHILD_RENDERS.get(), 1);
}

#[test]
fn equal_state_write_renders_and_presents_nothing() {
    RENDERS.set(0);
    let mut instance = testing::render(component(counter, ()));
    VALUE.get().unwrap().set(0);
    instance.settle();
    assert_eq!(RENDERS.get(), 1);
    assert_eq!(instance.frames().len(), 1);
}

#[test]
fn state_writes_before_settle_share_one_render_and_frame() {
    RENDERS.set(0);
    let mut instance = testing::render(component(counter, ()));
    let value = VALUE.get().unwrap();
    value.set(1);
    value.set(2);
    value.set(3);
    instance.settle();
    assert_eq!(RENDERS.get(), 2);
    assert_eq!(instance.frames().len(), 2);
    assert_eq!(instance.last_frame(), "3");
}

#[test]
fn layout_effect_state_update_presents_one_frame() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        RENDERS.set(RENDERS.get() + 1);
        let value = cx.use_state(|| 0);
        cx.use_layout_effect((), move || value.set(1));
        Text::new(value.get().to_string()).into()
    }
    RENDERS.set(0);
    let instance = testing::render(component(example, ()));
    assert_eq!(instance.frames().len(), 1);
    assert_eq!(instance.last_frame(), "1");
    assert_eq!(RENDERS.get(), 2);
}

#[test]
fn context_update_reaches_reader_below_equal_props_in_one_frame() {
    thread_local! {
        static READER_RENDERS: Cell<usize> = const { Cell::new(0) };
    }
    fn reader(cx: &mut Scope, _: &()) -> Element {
        READER_RENDERS.set(READER_RENDERS.get() + 1);
        Text::new(cx.use_context::<usize>().unwrap().to_string()).into()
    }
    fn middle(_: &mut Scope, _: &()) -> Element {
        CHILD_RENDERS.set(CHILD_RENDERS.get() + 1);
        View::new()
            .child(Text::new("").child(Element::fragment([component(reader, ())])))
            .into()
    }
    fn provider(cx: &mut Scope, _: &()) -> Element {
        RENDERS.set(RENDERS.get() + 1);
        let value = cx.use_state(|| 0);
        VALUE.set(Some(value));
        cx.provide(value.get() % 2);
        component(middle, ())
    }
    RENDERS.set(0);
    CHILD_RENDERS.set(0);
    READER_RENDERS.set(0);
    let mut instance = testing::render(component(provider, ()));
    assert_eq!(instance.frames(), ["0"]);
    VALUE.get().unwrap().set(1);
    instance.settle();
    assert_eq!(RENDERS.get(), 2);
    assert_eq!(CHILD_RENDERS.get(), 1);
    assert_eq!(READER_RENDERS.get(), 2);
    assert_eq!(instance.frames(), ["0", "1"]);

    VALUE.get().unwrap().set(3);
    instance.settle();
    assert_eq!(RENDERS.get(), 3);
    assert_eq!(CHILD_RENDERS.get(), 1);
    assert_eq!(READER_RENDERS.get(), 2);
    assert_eq!(instance.last_frame(), "1");
    let frames = instance.frames().len();
    instance.settle();
    assert_eq!(instance.frames().len(), frames);
}

#[test]
fn context_reads_ancestors_and_nearest_provider_wins() {
    thread_local! {
        static READER_RENDERS: Cell<usize> = const { Cell::new(0) };
    }
    fn reader(cx: &mut Scope, _: &()) -> Element {
        READER_RENDERS.set(READER_RENDERS.get() + 1);
        Text::new(cx.use_context::<usize>().unwrap().to_string()).into()
    }
    fn inner(cx: &mut Scope, _: &()) -> Element {
        let ancestor = cx.use_context::<usize>().unwrap();
        cx.provide(ancestor + 100);
        cx.provide(10_usize);
        assert_eq!(cx.use_context::<usize>(), Some(ancestor));
        Text::new(format!("{ancestor}/"))
            .child(component(reader, ()))
            .into()
    }
    fn outer(cx: &mut Scope, value: &usize) -> Element {
        cx.provide(*value);
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Static::new(vec![()], |_, _| component(reader, ())))
            .child(Element::fragment([
                component(inner, ()),
                component(reader, ()),
            ]))
            .into()
    }
    READER_RENDERS.set(0);
    let mut instance = testing::render(component(outer, 1_usize));
    assert_eq!(instance.frames(), ["1\n1/10\n1"]);
    assert_eq!(READER_RENDERS.get(), 3);
    instance.rerender(component(outer, 2_usize));
    assert_eq!(instance.frames(), ["1\n1/10\n1", "1\n2/10\n2"]);
    assert_eq!(READER_RENDERS.get(), 4);
}

#[test]
fn context_without_provider_returns_none() {
    fn reader(cx: &mut Scope, _: &()) -> Element {
        assert_eq!(cx.use_context::<String>(), None);
        Element::empty()
    }
    let instance = testing::render(component(reader, ()));
    assert_eq!(instance.last_frame(), "");
}
