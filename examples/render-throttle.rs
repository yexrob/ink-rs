// Ink examples/render-throttle.

use std::time::Duration;

use ink_rs::{
    AnimationOptions, Element, FlexDirection, Options, Scope, Styled, Text, View, component, render,
};

fn counter(cx: &mut Scope, _: &()) -> Element {
    let tick = cx.use_animation(AnimationOptions {
        interval: Duration::from_millis(10),
        ..AnimationOptions::default()
    });
    // Count derives from frame, so throttled wakes skip to the elapsed count.
    View::new()
        .flex_direction(FlexDirection::Column)
        .padding(1)
        .child(Text::new(format!("Counter: {}", tick.frame)))
        .child(Text::new(
            "This updates every 10ms but renders are throttled",
        ))
        .child(Text::new("Press Ctrl+C to exit"))
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
