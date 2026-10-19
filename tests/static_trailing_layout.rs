//! Ported from ink test/static-trailing-layout.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn rendertostring_preserves_0_trailing_dynamic_rows_after_static() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["A"], |item, _| Text::new(*item).into()).into(),
            View::new().height(0).into(),
        ]),
        80,
    );
    assert_eq!(output, "A");
}

#[test]
fn rendertostring_preserves_1_trailing_dynamic_rows_after_static() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["A"], |item, _| Text::new(*item).into()).into(),
            View::new().height(1).into(),
        ]),
        80,
    );
    assert_eq!(output, lines(&["A", "",]));
}

#[test]
fn rendertostring_preserves_2_trailing_dynamic_rows_after_static() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["A"], |item, _| Text::new(*item).into()).into(),
            View::new().height(2).into(),
        ]),
        80,
    );
    assert_eq!(output, lines(&["A", "", "",]));
}

#[test]
fn blank_dynamic_rows_agree_with_ordinary_layout_after_static() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["item"], |_, _| Text::new("").into()).into(),
            View::new().height(1).into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new(""))
            .child(View::new().height(1)),
        80,
    );
    assert_eq!(output, "");
    assert_eq!(output, ordinary);
}

#[test]
fn blank_dynamic_rows_agree_with_ordinary_layout_after_static_a() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["item"], |_, _| Text::new("A").into()).into(),
            View::new().height(1).into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("A"))
            .child(View::new().height(1)),
        80,
    );
    assert_eq!(output, lines(&["A", "",]));
    assert_eq!(output, ordinary);
}

#[test]
fn blank_dynamic_rows_agree_with_ordinary_layout_after_static_a_nb() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["item"], |_, _| Text::new("A\nB").into()).into(),
            View::new().height(1).into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("A\nB"))
            .child(View::new().height(1)),
        80,
    );
    assert_eq!(output, lines(&["A", "B", "",]));
    assert_eq!(output, ordinary);
}

#[test]
fn hidden_dynamic_content_does_not_add_a_separator_to_static_output() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["A"], |item, _| Text::new(*item).into()).into(),
            View::new()
                .display(Display::None)
                .height(1)
                .child(Text::new("Hidden"))
                .into(),
        ]),
        80,
    );
    assert_eq!(output, "A");
}
