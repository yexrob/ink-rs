// Ink examples/use-stdout.

use std::time::Duration;

use ink_rs::{AnimationOptions, Element, FlexDirection, Scope, Styled, Text, View, component};

pub fn app() -> Element {
    component(dimensions, ())
}

fn dimensions(cx: &mut Scope, _: &()) -> Element {
    let writer = cx.use_stdout();
    let size = cx.use_window_size();
    let tick = cx.use_animation(AnimationOptions {
        interval: Duration::from_secs(1),
        ..AnimationOptions::default()
    });
    // One write per delivered frame; a late wake does not replay missed writes.
    cx.use_effect(tick.frame, move || {
        if tick.frame > 0 {
            writer.write("Hello from Ink to stdout\n");
        }
    });
    View::new()
        .flex_direction(FlexDirection::Column)
        .padding_x(2)
        .padding_y(1)
        .child(Text::new("Terminal dimensions:").bold(true).underline(true))
        .child(
            View::new()
                .margin_top(1)
                .child(Text::new("Width: ").child(Text::new(size.columns.to_string()).bold(true))),
        )
        .child(Text::new("Height: ").child(Text::new(size.rows.to_string()).bold(true)))
        .into()
}
