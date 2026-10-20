//! Ported from Ink test/use-box-metrics.tsx (8.0.0, commit 26d2c3f).
//! Flushes, delays and act() use settle(); resize uses the live test driver.

mod common;

use std::cell::Cell;
use std::time::Duration;

use common::Screen;
use ink_rs::{
    Element, FlexDirection, NodeRef, Options, Overflow, Scope, State, Styled, Text, View,
    component, testing,
};

thread_local! {
    static TRACK_SECOND: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static MOUNTED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static SIBLING_TEXT: Cell<Option<State<&'static str>>> = const { Cell::new(None) };
    static ITEMS: Cell<Option<State<Vec<String>>>> = const { Cell::new(None) };
    static RENDERS: Cell<usize> = const { Cell::new(0) };
}

#[test]
fn returns_correct_size_on_first_render() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let metrics = cx.use_box_metrics(node);
        View::new()
            .node_ref(node)
            .child(Text::new(format!("{}x{}", metrics.width, metrics.height)))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    // ink-rs checks after ref attachment and before paint, so no zero frame is written.
    assert!(instance.frames()[0].contains("100x1"));
    instance.settle();
    assert!(instance.last_frame().contains("100x1"));
}

#[test]
fn returns_correct_position() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let metrics = cx.use_box_metrics(node);
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("first line"))
            .child(
                View::new()
                    .node_ref(node)
                    .margin_left(5)
                    .child(Text::new(format!("{},{}", metrics.left, metrics.top))),
            )
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("5,1"));
}

#[test]
fn updates_when_terminal_is_resized() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let metrics = cx.use_box_metrics(node);
        View::new()
            .node_ref(node)
            .child(Text::new(format!("Width: {}", metrics.width)))
            .into()
    }
    let terminal = testing::TestTerminal::new(100, 24);
    let mut app = testing::mount(component(example, ()), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 100, 24);
    assert!(screen.rows()[0].contains("Width: 100"));
    screen.resize(60, 24);
    app.resize(60, 24);
    app.advance(Duration::from_millis(200));
    screen.update(&terminal);
    assert!(screen.rows()[0].contains("Width: 60"));
}

#[test]
fn uses_latest_tracked_ref_when_terminal_is_resized() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let first = cx.use_node_ref();
        let second = cx.use_node_ref();
        let track_second = cx.use_state(|| false);
        TRACK_SECOND.set(Some(track_second));
        let metrics = cx.use_box_metrics(if track_second.get() { second } else { first });
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(View::new().node_ref(first).child(Text::new("short")))
            .child(View::new().node_ref(second).child(Text::new(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            )))
            .child(Text::new(format!("Tracked height: {}", metrics.height)))
            .into()
    }
    let terminal = testing::TestTerminal::new(100, 24);
    let mut app = testing::mount(component(example, ()), Options::default(), &terminal);
    let mut screen = Screen::new(&terminal, 100, 24);
    assert!(screen.rows().join("\n").contains("Tracked height: 1"));
    TRACK_SECOND.get().unwrap().set(true);
    app.advance(Duration::from_millis(50));
    screen.update(&terminal);
    assert!(screen.rows().join("\n").contains("Tracked height: 1"));
    screen.resize(20, 24);
    app.resize(20, 24);
    app.advance(Duration::from_millis(200));
    screen.update(&terminal);
    assert!(screen.rows().join("\n").contains("Tracked height: 4"));
}

#[test]
fn updates_when_sibling_content_changes() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let text = cx.use_state(|| "short");
        SIBLING_TEXT.set(Some(text));
        let metrics = cx.use_box_metrics(node);
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .node_ref(node)
                    .flex_direction(FlexDirection::Column)
                    .child(Text::new(text.get())),
            )
            .child(Text::new(format!("Height: {}", metrics.height)))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Height: 1"));
    SIBLING_TEXT.get().unwrap().set("line 1\nline 2\nline 3");
    instance.settle();
    assert!(instance.last_frame().contains("Height: 3"));
}

fn tracked_top(cx: &mut Scope, mounted: &bool) -> Element {
    let node = cx.use_node_ref();
    let metrics = cx.use_box_metrics(node);
    let text = Text::new(format!("Top: {}", metrics.top));
    if *mounted {
        View::new().node_ref(node).child(text).into()
    } else {
        text.into()
    }
}

#[test]
fn updates_when_sibling_content_changes_but_tracked_component_is_memoized() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let text = cx.use_state(|| "line 1");
        SIBLING_TEXT.set(Some(text));
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new(text.get()))
            .child(component(tracked_top, true))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Top: 1"));
    SIBLING_TEXT.get().unwrap().set("line 1\nline 2\nline 3");
    instance.settle();
    assert!(instance.last_frame().contains("Top: 3"));
}

#[test]
fn updates_when_tracked_ref_attaches_after_initial_render_and_component_is_memoized() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let text = cx.use_state(|| "line 1");
        let mounted = cx.use_state(|| false);
        SIBLING_TEXT.set(Some(text));
        MOUNTED.set(Some(mounted));
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new(text.get()))
            .child(component(tracked_top, mounted.get()))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Top: 0"));
    MOUNTED.get().unwrap().set(true);
    instance.settle();
    assert!(instance.last_frame().contains("Top: 1"));
    SIBLING_TEXT.get().unwrap().set("line 1\nline 2\nline 3");
    instance.settle();
    assert!(instance.last_frame().contains("Top: 3"));
}

#[test]
fn does_not_trigger_extra_re_renders_when_layout_is_unchanged() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        cx.use_box_metrics(node);
        RENDERS.set(RENDERS.get() + 1);
        View::new().node_ref(node).child(Text::new("Hello")).into()
    }
    RENDERS.set(0);
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    // Ink allows 2..=3 renders for passive delivery; ink-rs has two pre-paint
    // rounds: detached, then measured. The unchanged round marks nothing.
    assert_eq!(RENDERS.get(), 2);
    instance.advance(Duration::from_millis(100));
    assert_eq!(RENDERS.get(), 2);
}

// N/A: removes resize listener on unmount; does not crash when resize fires
// after unmount. These test Node's EventEmitter; unmount consumes the Rust app.

#[test]
fn returns_zeros_when_ref_is_not_attached() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let m = cx.use_box_metrics(node);
        View::new()
            .child(Text::new(format!(
                "{},{},{},{},{}",
                m.width, m.height, m.left, m.top, m.has_measured
            )))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("0,0,0,0,false"));
}

#[test]
fn has_measured_becomes_true_when_tracked_element_is_mounted_on_initial_render() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let m = cx.use_box_metrics(node);
        View::new()
            .node_ref(node)
            .child(Text::new(format!("Has measured: {}", m.has_measured)))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Has measured: true"));
}

#[test]
fn has_measured_resets_when_tracked_ref_switches_to_a_detached_element() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let first = cx.use_node_ref();
        let second = cx.use_node_ref();
        let tracked = cx.use_state(|| false);
        let mounted = cx.use_state(|| false);
        TRACK_SECOND.set(Some(tracked));
        MOUNTED.set(Some(mounted));
        let m = cx.use_box_metrics(if tracked.get() { second } else { first });
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(View::new().node_ref(first).child(Text::new("First")))
            .children(
                mounted
                    .get()
                    .then(|| View::new().node_ref(second).child(Text::new("Second"))),
            )
            .child(Text::new(format!("Has measured: {}", m.has_measured)))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Has measured: true"));
    TRACK_SECOND.get().unwrap().set(true);
    instance.settle();
    assert!(instance.last_frame().contains("Has measured: false"));
    MOUNTED.get().unwrap().set(true);
    instance.settle();
    assert!(instance.last_frame().contains("Has measured: true"));
}

#[test]
fn has_measured_becomes_true_after_the_tracked_element_is_measured() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let mounted = cx.use_state(|| false);
        MOUNTED.set(Some(mounted));
        let m = cx.use_box_metrics(node);
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(
                mounted
                    .get()
                    .then(|| View::new().node_ref(node).child(Text::new("Tracked"))),
            )
            .child(Text::new(format!("Has measured: {}", m.has_measured)))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Has measured: false"));
    MOUNTED.get().unwrap().set(true);
    instance.settle();
    assert!(instance.last_frame().contains("Has measured: true"));
}

#[test]
fn resets_metrics_when_tracked_element_unmounts() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let mounted = cx.use_state(|| true);
        MOUNTED.set(Some(mounted));
        let m = cx.use_box_metrics(node);
        View::new()
            .flex_direction(FlexDirection::Column)
            .children(mounted.get().then(|| {
                View::new()
                    .node_ref(node)
                    .width(10)
                    .child(Text::new("1234567890"))
            }))
            .child(Text::new(format!(
                "Metrics: {},{},{},{},{}",
                m.width, m.height, m.left, m.top, m.has_measured
            )))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("Metrics: 10,1,0,0,true"));
    MOUNTED.get().unwrap().set(false);
    instance.settle();
    assert!(instance.last_frame().contains("Metrics: 0,0,0,0,false"));
}

#[test]
fn returns_viewport_client_size_and_content_size_for_overflowing_content() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let content_node = cx.use_node_ref();
        let viewport = cx.use_box_metrics(node);
        let content = cx.use_box_metrics(content_node);
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .node_ref(node)
                    .width(12)
                    .height(2)
                    .overflow(Overflow::Hidden)
                    .flex_direction(FlexDirection::Column)
                    .child(
                        View::new()
                            .node_ref(content_node)
                            .width(20)
                            .height(5)
                            .flex_shrink(0.0)
                            .flex_grow(0.0),
                    ),
            )
            .child(Text::new(format!(
                "client:{}x{} content:{}x{}",
                viewport.client_width, viewport.client_height, content.width, content.height
            )))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("client:12x2 content:20x5"));
}

#[test]
fn content_size_updates_when_content_grows() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let items = cx.use_state(|| vec!["a".to_owned(), "b".to_owned()]);
        ITEMS.set(Some(items));
        let content = cx.use_box_metrics(node);
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .height(2)
                    .overflow_y(Overflow::Hidden)
                    .flex_direction(FlexDirection::Column)
                    .child(
                        View::new()
                            .node_ref(node)
                            .flex_direction(FlexDirection::Column)
                            .flex_shrink(0.0)
                            .children(
                                items
                                    .get()
                                    .iter()
                                    .map(|item| Text::new(item).key(item.clone())),
                            ),
                    ),
            )
            .child(Text::new(format!("contentHeight:{}", content.height)))
            .into()
    }
    let mut instance = testing::render(component(example, ()));
    instance.settle();
    assert!(instance.last_frame().contains("contentHeight:2"));
    ITEMS
        .get()
        .unwrap()
        .update(|items| items.push(format!("item {}", items.len())));
    instance.settle();
    assert!(instance.last_frame().contains("contentHeight:3"));
}

#[test]
fn tracks_a_box_mounted_by_child_state_concurrent_false() {
    fn child(cx: &mut Scope, node: &NodeRef) -> Element {
        let visible = cx.use_state(|| false);
        MOUNTED.set(Some(visible));
        if visible.get() {
            View::new()
                .node_ref(*node)
                .width(10)
                .child(Text::new("Tracked"))
                .into()
        } else {
            Element::fragment(Vec::<Element>::new())
        }
    }
    fn parent(cx: &mut Scope, _: &()) -> Element {
        let node = cx.use_node_ref();
        let m = cx.use_box_metrics(node);
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(component(child, node))
            .child(Text::new(format!("{}:{}", m.width, m.has_measured)))
            .into()
    }
    let mut instance = testing::render(component(parent, ()));
    instance.settle();
    assert_eq!(instance.last_frame(), "0:false");
    MOUNTED.get().unwrap().set(true);
    instance.settle();
    assert_eq!(instance.last_frame(), "Tracked\n10:true");
    MOUNTED.get().unwrap().set(false);
    instance.settle();
    assert_eq!(instance.last_frame(), "0:false");
}

// N/A: tracks a box mounted by child state - concurrent=true duplicates false.
