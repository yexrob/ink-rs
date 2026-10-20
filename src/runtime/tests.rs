use std::cell::Cell;
use std::time::Instant;

use super::Runtime;
use crate::{Element, RenderOptions, Scope, Text, component};

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
