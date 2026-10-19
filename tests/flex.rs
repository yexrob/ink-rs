//! Ported from ink test/flex.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn grow_equally() {
    let output = render(
        View::new()
            .width(6)
            .child(View::new().flex_grow(1.0).child(Text::new("A")))
            .child(View::new().flex_grow(1.0).child(Text::new("B"))),
    );
    assert_eq!(output, "A  B");
}

#[test]
fn grow_one_element() {
    let output = render(
        View::new()
            .width(6)
            .child(View::new().flex_grow(1.0).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A    B");
}

#[test]
fn do_not_shrink() {
    let output = render(
        View::new()
            .width(16)
            .child(View::new().flex_shrink(0.0).width(6).child(Text::new("A")))
            .child(View::new().flex_shrink(0.0).width(6).child(Text::new("B")))
            .child(View::new().width(6).child(Text::new("C"))),
    );
    assert_eq!(output, "A     B     C");
}

#[test]
fn shrink_equally() {
    let output = render(
        View::new()
            .width(10)
            .child(View::new().flex_shrink(1.0).width(6).child(Text::new("A")))
            .child(View::new().flex_shrink(1.0).width(6).child(Text::new("B")))
            .child(Text::new("C")),
    );
    assert_eq!(output, "A    B   C");
}

#[test]
fn set_flex_basis_with_flex_direction_row_container() {
    let output = render(
        View::new()
            .width(6)
            .child(View::new().flex_basis(3).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A  B");
}

#[test]
fn set_flex_basis_in_percent_with_flex_direction_row_container() {
    let output = render(
        View::new()
            .width(6)
            .child(View::new().flex_basis(Percent(50.0)).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A  B");
}

#[test]
fn set_flex_basis_with_flex_direction_column_container() {
    let output = render(
        View::new()
            .height(6)
            .flex_direction(FlexDirection::Column)
            .child(View::new().flex_basis(3).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "", "", "B", "", "",]));
}

#[test]
fn set_flex_basis_in_percent_with_flex_direction_column_container() {
    let output = render(
        View::new()
            .height(6)
            .flex_direction(FlexDirection::Column)
            .child(View::new().flex_basis(Percent(50.0)).child(Text::new("A")))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "", "", "B", "", "",]));
}
