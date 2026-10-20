use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

use super::Runtime;
use crate::{AnimationOptions, Element, RenderOptions, Scope, Text, component};

#[test]
fn a_panic_in_the_overview_commit_propagates() {
    super::panic::install();
    let mut runtime = Runtime::new(
        component(|_, _: &()| panic!("original panic"), ()),
        RenderOptions::default(),
    );
    runtime.catch = true;
    runtime.commit();
    assert_eq!(runtime.take_panic().unwrap().message(), "original panic");
    runtime.rerender(component(|_, _: &()| panic!("broken overview"), ()));
    let error = catch_unwind(AssertUnwindSafe(|| runtime.commit()))
        .err()
        .unwrap();
    assert_eq!(*error.downcast::<&str>().unwrap(), "broken overview");
}

thread_local! {
    static RENDERS: Cell<usize> = const { Cell::new(0) };
}

#[test]
fn idle_runtime_renders_and_presents_nothing() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        RENDERS.set(RENDERS.get() + 1);
        let value = cx.use_state(|| 0);
        cx.use_effect((), move || value.set(1));
        Text::new(value.get().to_string()).into()
    }
    RENDERS.set(0);
    let now = Instant::now();
    let mut runtime = Runtime::new(component(example, ()), RenderOptions::default());
    let mut frames = 0;
    assert_eq!(runtime.render_if_due(now, |_| frames += 1), Some(now));
    assert_eq!(runtime.render_if_due(now, |_| frames += 1), None);
    let renders = RENDERS.get();
    let presents = frames;
    assert_eq!(runtime.render_if_due(now, |_| frames += 1), None);
    assert_eq!(RENDERS.get(), renders);
    assert_eq!(frames, presents);
}

#[test]
fn animation_deadline_is_the_minimum_and_none_without_active_subscriptions() {
    fn example(cx: &mut Scope, active: &[bool; 2]) -> Element {
        let a = cx.use_animation(AnimationOptions {
            interval: Duration::from_millis(50),
            active: active[0],
        });
        let b = cx.use_animation(AnimationOptions {
            interval: Duration::from_millis(80),
            active: active[1],
        });
        Text::new(format!(
            "{},{},{},{}",
            a.frame,
            b.frame,
            a.time.as_millis(),
            a.delta.as_millis()
        ))
        .into()
    }
    let now = Instant::now();
    let ms = Duration::from_millis;
    let mut runtime = Runtime::new(component(example, [false; 2]), RenderOptions::default());
    assert_eq!(runtime.render_if_due(now, |_| {}), None);
    assert_eq!(runtime.animation_deadline(Duration::ZERO), None);
    runtime.rerender(component(example, [true; 2]));
    assert_eq!(runtime.render_if_due(now, |_| {}), Some(now + ms(50)));
    assert_eq!(runtime.animation_deadline(ms(101)), Some(now + ms(150)));

    // Ink's tickAsync test visits every slot. A late core step must also skip
    // missed slots and retain their elapsed delta rather than replaying ticks.
    let mut output = String::new();
    assert_eq!(
        runtime.render_if_due(now + ms(220), |frame| output = frame.serialize()),
        Some(now + ms(240))
    );
    assert_eq!(output, "4,2,220,220");
    assert_eq!(runtime.animation_deadline(ms(101)), Some(now + ms(350)));
    runtime.rerender(component(example, [false, true]));
    assert_eq!(
        runtime.render_if_due(now + ms(220), |_| {}),
        Some(now + ms(240))
    );
    runtime.rerender(component(example, [false; 2]));
    assert_eq!(runtime.render_if_due(now + ms(220), |_| {}), None);
    assert_eq!(runtime.animation_deadline(ms(101)), None);
    runtime.rerender(component(example, [true; 2]));
    runtime.render_if_due(now + ms(220), |_| {});
    runtime.rerender(Element::fragment(Vec::<Element>::new()));
    assert_eq!(runtime.render_if_due(now + ms(220), |_| {}), None);
}
