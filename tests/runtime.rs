//! Rust-native acceptance tests for the stepping runtime.

use std::cell::Cell;

use ink_rs::{Element, Scope, State, Text, component, testing};

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
