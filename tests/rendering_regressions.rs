//! Ported from ink test/rendering-regressions.tsx @ 26d2c3f.

mod common;

use common::render;
use ink_rs::*;

#[test]
fn clipping_a_wide_character_at_the_right_edge_preserves_the_adjacent_marker_column() {
    let output = render(
        View::new()
            .child(
                View::new()
                    .width(4)
                    .height(1)
                    .overflow_x(Overflow::Hidden)
                    .child(
                        View::new()
                            .width(7)
                            .flex_shrink(0.0)
                            .child(Text::new("X你好世")),
                    ),
            )
            .child(Text::new("|")),
    );
    assert_eq!(output, "X你 |");
}

#[test]
fn clipping_a_wide_character_at_the_left_edge_preserves_the_adjacent_marker_column() {
    let output = render(
        View::new()
            .child(
                View::new()
                    .width(3)
                    .height(1)
                    .overflow_x(Overflow::Hidden)
                    .content_offset_x(1)
                    .child(
                        View::new()
                            .width(4)
                            .flex_shrink(0.0)
                            .child(Text::new("你好")),
                    ),
            )
            .child(Text::new("|")),
    );
    assert_eq!(output, " 好|");
}
