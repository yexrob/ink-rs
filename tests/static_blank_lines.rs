//! Ported from ink test/static-blank-lines.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn static_preserves_0_blank_rows_before_dynamic_output() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["blank"], |_, _| View::new().height(0).into()).into(),
            Text::new("after").into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(View::new().height(0))
            .child(Text::new("after")),
        80,
    );
    assert_eq!(ordinary, "after");
    assert_eq!(output, "after");
}

#[test]
fn static_preserves_1_blank_rows_before_dynamic_output() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["blank"], |_, _| View::new().height(1).into()).into(),
            Text::new("after").into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(View::new().height(1))
            .child(Text::new("after")),
        80,
    );
    assert_eq!(ordinary, lines(&["", "after",]));
    assert_eq!(output, lines(&["", "after",]));
}

#[test]
fn static_preserves_2_blank_rows_before_dynamic_output() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["blank"], |_, _| View::new().height(2).into()).into(),
            Text::new("after").into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(View::new().height(2))
            .child(Text::new("after")),
        80,
    );
    assert_eq!(ordinary, lines(&["", "", "after",]));
    assert_eq!(output, lines(&["", "", "after",]));
}

#[test]
fn static_preserves_3_blank_rows_before_dynamic_output() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["blank"], |_, _| View::new().height(3).into()).into(),
            Text::new("after").into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(View::new().height(3))
            .child(Text::new("after")),
        80,
    );
    assert_eq!(ordinary, lines(&["", "", "", "after",]));
    assert_eq!(output, lines(&["", "", "", "after",]));
}

#[test]
fn static_preserves_whitespace_content_undefined() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["item"], |_, _| {
                View::new().child(Text::default()).into()
            })
            .into(),
            Text::new("after").into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::default())
            .child(Text::new("after")),
        80,
    );
    assert_eq!(output, "after");
    assert_eq!(output, ordinary);
}

#[test]
fn static_preserves_whitespace_content_empty() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["item"], |_, _| View::new().child(Text::new("")).into()).into(),
            Text::new("after").into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new(""))
            .child(Text::new("after")),
        80,
    );
    assert_eq!(output, "after");
    assert_eq!(output, ordinary);
}

#[test]
fn static_preserves_whitespace_content_space() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["item"], |_, _| {
                View::new().child(Text::new(" ")).into()
            })
            .into(),
            Text::new("after").into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new(" "))
            .child(Text::new("after")),
        80,
    );
    assert_eq!(output, lines(&["", "after",]));
    assert_eq!(output, ordinary);
}

#[test]
fn static_preserves_whitespace_content_newline() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(vec!["item"], |_, _| {
                View::new().child(Text::new("\n")).into()
            })
            .into(),
            Text::new("after").into(),
        ]),
        80,
    );
    let ordinary = render_cols(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Text::new("\n"))
            .child(Text::new("after")),
        80,
    );
    assert_eq!(output, lines(&["", "", "after",]));
    assert_eq!(output, ordinary);
}

#[test]
fn empty_static_does_not_insert_a_blank_row() {
    let output = render_cols(
        Element::fragment::<[Element; 2]>([
            Static::new(Vec::<&str>::new(), |item, _| Text::new(*item).into()).into(),
            Text::new("after").into(),
        ]),
        80,
    );
    assert_eq!(output, "after");
}
