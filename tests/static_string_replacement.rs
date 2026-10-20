//! Ported from ink test/static-string-replacement.tsx @ 26d2c3f.

mod common;

use std::cell::Cell;

use common::*;
use ink_rs::*;

fn example(cx: &mut Scope, should_replace: &bool) -> Element {
    let updated = cx.use_state(|| false);
    cx.use_layout_effect((), move || updated.set(true));
    let updated = updated.get();
    Element::fragment([
        if !updated || *should_replace {
            Static::new(vec![if updated { "New" } else { "Old" }], |item, _| {
                Text::new(*item).key(*item)
            })
            .key(if updated { "new" } else { "old" })
        } else {
            Element::empty()
        },
        Text::new("Live").into(),
    ])
}

#[test]
fn render_to_string_reflects_a_layout_effect_static_removal() {
    assert_eq!(render_cols(component(example, false), 80), "Live");
}

#[test]
fn render_to_string_reflects_a_layout_effect_static_replacement() {
    assert_eq!(render_cols(component(example, true), 80), "New\nLive");
}

#[test]
fn layout_effect_appends_to_the_same_static_retain_earlier_items() {
    fn example(cx: &mut Scope, _: &()) -> Element {
        let items = cx.use_state(|| vec!["First"]);
        cx.use_layout_effect((), move || items.set(vec!["First", "Second"]));
        Static::new(items.get(), |item, _| Text::new(*item).key(*item)).into()
    }
    assert_eq!(render_cols(component(example, ()), 80), "First\nSecond");
}

thread_local! {
    static CLEANUP_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[test]
fn replacement_static_survives_cleanup_in_static_only_output() {
    CLEANUP_COUNT.set(0);
    fn example(cx: &mut Scope, _: &()) -> Element {
        let updated = cx.use_state(|| false);
        cx.use_layout_effect((), move || {
            updated.set(true);
            Cleanup(|| CLEANUP_COUNT.set(CLEANUP_COUNT.get() + 1))
        });
        Static::new(
            vec![if updated.get() { "New" } else { "Old" }],
            |item, _| Text::new(*item).key(*item),
        )
        .key(updated.get().to_string())
    }
    assert_eq!(render_cols(component(example, ()), 80), "New");
    assert_eq!(CLEANUP_COUNT.get(), 1);
}
