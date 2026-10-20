// Ink examples/static.

use std::time::Duration;

use ink_rs::{
    AnimationOptions, Color, Element, Options, Scope, Static, Text, View, component, render, rsx,
};

fn completed(cx: &mut Scope, _: &()) -> Element {
    let tests = cx.use_state(Vec::<usize>::new);
    let tick = cx.use_animation(AnimationOptions {
        interval: Duration::from_millis(100),
        active: tests.get().len() < 10,
    });
    // One item per delivered frame, including frame zero as Ink's initial run.
    cx.use_effect(tick.frame, move || {
        tests.update(|tests| {
            if tests.len() < 10 {
                tests.push(tests.len() + 1);
            }
        });
    });
    let tests = tests.get();
    let count = tests.len();
    rsx! {
        Static(tests, |id, _| rsx! {
            Text(color: Color::Green, key: *id) { "\u{2714} Test #{id}" }
        })
        View(margin_top: 1) {
            Text(dim_color: true) { "Completed tests: {count}" }
        }
    }
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(completed, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
