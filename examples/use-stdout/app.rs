// Ink examples/use-stdout.

use std::time::Duration;

use ink_rs::{AnimationOptions, Element, FlexDirection, Scope, Text, View, component, rsx};

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
    rsx! {
        View(flex_direction: FlexDirection::Column, padding_x: 2, padding_y: 1) {
            Text(bold: true, underline: true) { "Terminal dimensions:" }
            View(margin_top: 1) {
                Text {
                    "Width: "
                    Text(bold: true) { {size.columns.to_string()} }
                }
            }
            Text {
                "Height: "
                Text(bold: true) { {size.rows.to_string()} }
            }
        }
    }
    .into()
}
