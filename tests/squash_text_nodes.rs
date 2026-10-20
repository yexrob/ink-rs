//! Ported from ink test/squash-text-nodes.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

#[test]
fn nested_transforms_receive_each_explicit_line_with_its_line_index() {
    let output = render(Text::new(
        Transform::new(|line, index| line.replace('x', &index.to_string()))
            .child(Text::new("xx\nxx")),
    ));
    assert_eq!(output, lines(&["00", "11",]));
}

#[test]
fn nested_transform_line_indices_do_not_depend_on_preceding_siblings() {
    let output = render(Text::new("prefix").child(
        Transform::new(|line, index| line.replace('x', &index.to_string())).child(Text::new("xx")),
    ));
    assert_eq!(output, "prefix00");
}
