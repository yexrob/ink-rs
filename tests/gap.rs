//! Ported from ink test/gap.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

#[test]
fn gap() {
    let output = render(
        View::new()
            .gap(1)
            .width(3)
            .flex_wrap(FlexWrap::Wrap)
            .child(Text::new("A"))
            .child(Text::new("B"))
            .child(Text::new("C")),
    );
    assert_eq!(output, lines(&["A B", "", "C",]));
}

#[test]
fn column_gap() {
    let output = render(
        View::new()
            .gap(1)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, "A B");
}

#[test]
fn row_gap() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .gap(1)
            .child(Text::new("A"))
            .child(Text::new("B")),
    );
    assert_eq!(output, lines(&["A", "", "B",]));
}

#[test]
fn removing_column_gap_restores_gap_on_rerender() {
    fn example(_: &mut Scope, value: &Option<u16>) -> Element {
        let mut view = View::new().gap(2).flex_direction(FlexDirection::Row);
        if let Some(gap) = value {
            view = view.column_gap(*gap);
        }
        view.child(Text::new("A")).child(Text::new("B")).into()
    }
    let mut instance = testing::render(component(example, Some(1)));
    assert_ansi_eq(instance.last_frame(), "A B");
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), "A  B");
    instance.rerender(component(example, Some(0)));
    assert_ansi_eq(instance.last_frame(), "AB");
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), "A  B");
}

#[test]
fn removing_row_gap_restores_gap_on_rerender() {
    fn example(_: &mut Scope, value: &Option<u16>) -> Element {
        let mut view = View::new().gap(2).flex_direction(FlexDirection::Column);
        if let Some(gap) = value {
            view = view.row_gap(*gap);
        }
        view.child(Text::new("A")).child(Text::new("B")).into()
    }
    let mut instance = testing::render(component(example, Some(1)));
    assert_ansi_eq(instance.last_frame(), &lines(&["A", "", "B"]));
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), &lines(&["A", "", "", "B"]));
    instance.rerender(component(example, Some(0)));
    assert_ansi_eq(instance.last_frame(), &lines(&["A", "B"]));
    instance.rerender(component(example, None));
    assert_ansi_eq(instance.last_frame(), &lines(&["A", "", "", "B"]));
}
