// Ink examples/static.

use std::time::Duration;

use ink_rs::{
    AnimationOptions, Color, Element, Options, Scope, Static, Styled, Text, View, component, render,
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
    Element::fragment::<[Element; 2]>([
        Static::new(tests, |id, _| {
            Text::new(format!("\u{2714} Test #{id}"))
                .color(Color::Green)
                .key(*id)
        })
        .into(),
        View::new()
            .margin_top(1)
            .child(Text::new(format!("Completed tests: {count}")).dim_color(true))
            .into(),
    ])
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(completed, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
