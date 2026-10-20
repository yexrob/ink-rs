//! Ported from ink test/static-runtime-blank-lines.tsx @ 26d2c3f.

use ink_rs::*;

#[test]
fn runtime_static_preserves_0_blank_rows_in_debug_output() {
    let mut instance = testing::render(Element::fragment::<[Element; 2]>([
        Static::new(vec!["blank"], |item, _| View::new().height(0).key(*item)).into(),
        Text::new("after").into(),
    ]));
    instance.settle();
    assert_eq!(instance.last_frame(), "after");
}

#[test]
fn runtime_static_preserves_1_blank_rows_in_debug_output() {
    let mut instance = testing::render(Element::fragment::<[Element; 2]>([
        Static::new(vec!["blank"], |item, _| View::new().height(1).key(*item)).into(),
        Text::new("after").into(),
    ]));
    instance.settle();
    assert_eq!(instance.last_frame(), "\nafter");
}

#[test]
fn runtime_static_preserves_2_blank_rows_in_debug_output() {
    let mut instance = testing::render(Element::fragment::<[Element; 2]>([
        Static::new(vec!["blank"], |item, _| View::new().height(2).key(*item)).into(),
        Text::new("after").into(),
    ]));
    instance.settle();
    assert_eq!(instance.last_frame(), "\n\nafter");
}
