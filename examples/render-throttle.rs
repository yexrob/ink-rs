// Ink examples/render-throttle.

use std::time::Duration;

use ink_rs::{
    AnimationOptions, Element, FlexDirection, Options, Scope, Text, View, component, render, rsx,
};

fn counter(cx: &mut Scope, _: &()) -> Element {
    let tick = cx.use_animation(AnimationOptions {
        interval: Duration::from_millis(10),
        ..AnimationOptions::default()
    });
    // Count derives from frame, so throttled wakes skip to the elapsed count.
    rsx! {
        View(flex_direction: FlexDirection::Column, padding: 1) {
            Text { {format!("Counter: {}", tick.frame)} }
            Text { "This updates every 10ms but renders are throttled" }
            Text { "Press Ctrl+C to exit" }
        }
    }
    .into()
}

fn main() -> Result<(), ink_rs::Error> {
    render(
        component(counter, ()),
        Options {
            max_fps: 10,
            ..Options::default()
        },
    )
    .wait_until_exit()
    .map(|_| ())
}
