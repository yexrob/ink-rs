//! Ported from ink test/style-update-consistency.tsx @ 26d2c3f.

use ink_rs::*;

fn view(child: View) -> Element {
    View::new()
        .width(30)
        .height(12)
        .flex_direction(FlexDirection::Column)
        .child(child.child(Text::new("Alpha")).child(Text::new("Beta")))
        .child(Text::new("End"))
        .into()
}

fn check(before: View, after: View) {
    let options = RenderOptions {
        columns: 30,
        color_level: ColorLevel::Basic,
    };
    let mut instance = testing::render_with(view(before), options);
    instance.settle();
    instance.rerender(view(after.clone()));
    instance.settle();
    assert_eq!(
        instance.last_frame(),
        render_to_string(view(after), options)
    );
}

#[test]
fn rerender_matches_fresh_layout_after_column_gap_override_removal() {
    check(View::new().gap(2).column_gap(4), View::new().gap(2));
}

#[test]
fn rerender_matches_fresh_layout_after_row_gap_override_removal() {
    check(View::new().gap(2).row_gap(4), View::new().gap(2));
}

#[test]
fn rerender_matches_fresh_layout_after_gap_removal() {
    check(View::new().gap(3), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_position_offset_removal() {
    check(
        View::new().position(Position::Relative).left(2).top(1),
        View::new(),
    );
}

#[test]
fn rerender_matches_fresh_layout_after_absolute_to_flow() {
    check(
        View::new().position(Position::Absolute).left(2).top(1),
        View::new(),
    );
}

#[test]
fn rerender_matches_fresh_layout_after_border_removal() {
    check(View::new().border_style(BorderStyle::Single), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_border_side_restoration() {
    check(
        View::new()
            .border_style(BorderStyle::Single)
            .border_left(false),
        View::new().border_style(BorderStyle::Single),
    );
}

#[test]
fn rerender_matches_fresh_layout_after_align_self_removal() {
    check(View::new().align_self(AlignSelf::FlexEnd), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_flex_basis_removal() {
    check(View::new().flex_basis(12), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_max_height_removal() {
    check(View::new().max_height(1), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_height_removal() {
    check(View::new().height(4), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_width_removal() {
    check(View::new().width(12), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_min_width_removal() {
    check(View::new().min_width(12), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_max_width_removal() {
    check(View::new().max_width(4), View::new());
}

#[test]
fn rerender_matches_fresh_layout_after_aspect_ratio_removal() {
    check(
        View::new().aspect_ratio(2.0).width(12),
        View::new().width(12),
    );
}
