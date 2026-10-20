// Ink examples/use-stderr.

use std::time::Duration;

use ink_rs::{AnimationOptions, Element, Options, Scope, Text, component, render, rsx};

fn example(cx: &mut Scope, _: &()) -> Element {
    let writer = cx.use_stderr();
    let tick = cx.use_animation(AnimationOptions {
        interval: Duration::from_secs(1),
        ..AnimationOptions::default()
    });
    // One write per delivered frame; a late wake does not replay missed writes.
    cx.use_effect(tick.frame, move || {
        if tick.frame > 0 {
            writer.write("Hello from Ink to stderr\n");
        }
    });
    rsx! {
        Text { "Hello World" }
    }
    .into()
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(example, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
