//! Ported from Ink test/measure-element.tsx (8.0.0, commit 26d2c3f).
//! Whole-frame comparisons use the recorded expectations. The contains assertions
//! keep Ink's source literals because the recording lacks them.

mod common;

use ink_rs::Options;
use std::cell::Cell;
use std::time::Duration;

use ink_rs::{
    BorderStyle, Element, FlexDirection, Overflow, Scope, State, Static, Styled, Text, View,
    component, measure_element as measure, testing,
};

thread_local! {
    static ITEMS: Cell<Option<State<Vec<&'static str>>>> = const { Cell::new(None) };
}

#[test]
fn measure_element() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let width = cx.use_state(|| 0);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || width.set(measure(node_ref).width));
        View::new()
            .node_ref(node_ref)
            .child(Text::new(format!("Width: {}", width.get())))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    assert_eq!(instance.frames()[0], "Width: 0");
    instance.settle();
    assert_eq!(instance.last_frame(), "Width: 100");
}

fn measured_items(cx: &mut Scope, layout_effect: &bool) -> Element {
    let items = cx.use_state(Vec::<&'static str>::new);
    let height = cx.use_state(|| 0);
    let node_ref = cx.use_node_ref();
    ITEMS.set(Some(items));
    let values = items.get();
    let measure = move || height.set(measure(node_ref).height);
    if *layout_effect {
        cx.use_layout_effect(values.len(), measure);
    } else {
        cx.use_effect(values.len(), measure);
    }
    View::new()
        .flex_direction(FlexDirection::Column)
        .child(
            View::new()
                .node_ref(node_ref)
                .flex_direction(FlexDirection::Column)
                .children(values.iter().map(|item| Text::new(*item).key(*item))),
        )
        .child(Text::new(format!("Height: {}", height.get())))
        .into()
}

#[test]
fn measure_element_after_state_update() {
    let mut instance = testing::render(component(measured_items, false));
    instance.settle();
    ITEMS.get().unwrap().set(vec!["line 1", "line 2", "line 3"]);
    instance.settle();
    assert_eq!(
        instance.last_frame().trim(),
        "line 1\nline 2\nline 3\nHeight: 3"
    );
}

#[test]
fn measure_element_after_multiple_state_updates() {
    let mut instance = testing::render(component(measured_items, false));
    instance.settle();
    ITEMS.get().unwrap().set(vec!["line 1", "line 2", "line 3"]);
    instance.settle();
    ITEMS.get().unwrap().set(vec!["line 1"]);
    instance.settle();
    assert_eq!(instance.last_frame().trim(), "line 1\nHeight: 1");
}

#[test]
fn measure_element_in_use_layout_effect_after_state_update() {
    let mut instance = testing::render(component(measured_items, true));
    instance.settle();
    ITEMS.get().unwrap().set(vec!["line 1", "line 2", "line 3"]);
    instance.settle();
    assert_eq!(
        instance.last_frame().trim(),
        "line 1\nline 2\nline 3\nHeight: 3"
    );
}

#[test]
fn measure_position_of_nested_element_with_padding_offset() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let result = cx.use_state(String::new);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || {
            let metrics = measure(node_ref);
            result.set(format!("{},{}", metrics.x, metrics.y));
        });
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("Header"))
            .child(
                View::new().padding_left(4).child(
                    View::new()
                        .node_ref(node_ref)
                        .child(Text::new(format!("Nested: {}", result.get()))),
                ),
            )
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Nested: 4,1"));
}

#[test]
fn measure_position_of_deeply_nested_element_accumulates_offsets() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let result = cx.use_state(String::new);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || {
            let metrics = measure(node_ref);
            result.set(format!("{},{}", metrics.x, metrics.y));
        });
        View::new()
            .padding_left(2)
            .padding_top(1)
            .child(
                View::new().padding_left(3).padding_top(2).child(
                    View::new()
                        .node_ref(node_ref)
                        .child(Text::new(format!("Deep: {}", result.get()))),
                ),
            )
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Deep: 5,3"));
}

#[test]
fn measure_position_accounts_for_margin_offset() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let result = cx.use_state(String::new);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || {
            let metrics = measure(node_ref);
            result.set(format!("{},{}", metrics.x, metrics.y));
        });
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new().margin_left(5).margin_top(2).child(
                    View::new()
                        .node_ref(node_ref)
                        .child(Text::new(format!("Margin: {}", result.get()))),
                ),
            )
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Margin: 5,2"));
}

#[test]
fn measure_position_sibling_offset_gives_correct_y() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let result = cx.use_state(String::new);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || {
            let metrics = measure(node_ref);
            result.set(format!("{},{}", metrics.x, metrics.y));
        });
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("Line 1"))
            .child(Text::new("Line 2"))
            .child(
                View::new()
                    .node_ref(node_ref)
                    .child(Text::new(format!("Third: {}", result.get()))),
            )
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Third: 0,2"));
}

#[test]
fn static_does_not_affect_layout_tree_coordinates() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let result = cx.use_state(String::new);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || {
            let metrics = measure(node_ref);
            result.set(format!("{},{}", metrics.x, metrics.y));
        });
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Static::new(vec!["Static A", "Static B"], |item, _| {
                Text::new(*item).key(*item)
            }))
            .child(
                View::new()
                    .node_ref(node_ref)
                    .child(Text::new(format!("Live: {}", result.get()))),
            )
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Live: 0,0"));
}

#[test]
fn calculate_layout_while_rendering_is_throttled() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let width = cx.use_state(|| 0);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || width.set(measure(node_ref).width));
        View::new()
            .node_ref(node_ref)
            .child(Text::new(format!("Width: {}", width.get())))
            .into()
    }
    let terminal = testing::TestTerminal::new(100, 24);
    let mut app = testing::mount(
        Element::fragment(Vec::<Element>::new()),
        Options::default(),
        &terminal,
    );
    app.rerender(component(example, ()));
    app.advance(Duration::from_millis(50));
    assert_eq!(
        common::Screen::new(&terminal, 100, 24).rows()[0],
        "Width: 100"
    );
}

#[test]
fn measure_element_client_size_with_overflowing_content() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let result = cx.use_state(String::new);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || {
            let metrics = measure(node_ref);
            result.set(format!(
                "client:{}x{}",
                metrics.client_width, metrics.client_height
            ));
        });
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .node_ref(node_ref)
                    .width(12)
                    .height(2)
                    .overflow(Overflow::Hidden)
                    .flex_direction(FlexDirection::Column)
                    .child(
                        View::new()
                            .width(20)
                            .height(5)
                            .flex_shrink(0.0)
                            .flex_grow(0.0),
                    ),
            )
            .child(Text::new(result.get()))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("client:12x2"));
}

#[test]
fn measure_element_client_size_excludes_borders() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let result = cx.use_state(String::new);
        let node_ref = cx.use_node_ref();
        cx.use_effect((), move || {
            let metrics = measure(node_ref);
            result.set(format!(
                "{}x{} client:{}x{}",
                metrics.width, metrics.height, metrics.client_width, metrics.client_height
            ));
        });
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .node_ref(node_ref)
                    .width(12)
                    .height(4)
                    .border_style(BorderStyle::Round),
            )
            .child(Text::new(result.get()))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("12x4 client:10x2"));
}
