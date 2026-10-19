//! Ported from ink test/clip-wide-background.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn clipping_wide_text_preserves_its_background_nested_false_offset_1_width_3() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(1)
            .child(
                View::new()
                    .width(4)
                    .flex_shrink(0.0)
                    .child(Text::new("你好").background_color(Color::Blue)),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue} 好{/}"]));
}

#[test]
fn clipping_wide_text_preserves_its_background_nested_false_offset_0_width_3() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(0)
            .child(
                View::new()
                    .width(4)
                    .flex_shrink(0.0)
                    .child(Text::new("你好").background_color(Color::Blue)),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}你 {/}"]));
}

#[test]
fn clipping_wide_text_preserves_its_background_nested_false_offset_1_width_2() {
    let output = render_truecolor(
        View::new()
            .width(2)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(1)
            .child(
                View::new()
                    .width(4)
                    .flex_shrink(0.0)
                    .child(Text::new("你好").background_color(Color::Blue)),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}  {/}"]));
}

#[test]
fn clipping_wide_text_preserves_its_background_nested_false_offset_0_width_4() {
    let output = render_truecolor(
        View::new()
            .width(4)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(0)
            .child(
                View::new()
                    .width(4)
                    .flex_shrink(0.0)
                    .child(Text::new("你好").background_color(Color::Blue)),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}你好{/}"]));
}

#[test]
fn clipping_wide_text_preserves_its_background_nested_true_offset_1_width_3() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(1)
            .child(
                View::new()
                    .width(4)
                    .flex_shrink(0.0)
                    .child(Text::default().child(Text::new("你好").background_color(Color::Blue))),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue} 好{/}"]));
}

#[test]
fn clipping_wide_text_preserves_its_background_nested_true_offset_0_width_3() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(0)
            .child(
                View::new()
                    .width(4)
                    .flex_shrink(0.0)
                    .child(Text::default().child(Text::new("你好").background_color(Color::Blue))),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}你 {/}"]));
}

#[test]
fn clipping_wide_text_preserves_its_background_nested_true_offset_1_width_2() {
    let output = render_truecolor(
        View::new()
            .width(2)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(1)
            .child(
                View::new()
                    .width(4)
                    .flex_shrink(0.0)
                    .child(Text::default().child(Text::new("你好").background_color(Color::Blue))),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}  {/}"]));
}

#[test]
fn clipping_wide_text_preserves_its_background_nested_true_offset_0_width_4() {
    let output = render_truecolor(
        View::new()
            .width(4)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(0)
            .child(
                View::new()
                    .width(4)
                    .flex_shrink(0.0)
                    .child(Text::default().child(Text::new("你好").background_color(Color::Blue))),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}你好{/}"]));
}

#[test]
fn clipped_halves_retain_their_own_background_colors() {
    let output = render_truecolor(
        View::new()
            .width(2)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(1)
            .child(
                View::new().width(4).flex_shrink(0.0).child(
                    Text::default()
                        .child(Text::new("你").background_color(Color::Blue))
                        .child(Text::new("好").background_color(Color::Red)),
                ),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue} {bg=red} {/}"]));
}

#[test]
fn clipping_a_joined_emoji_preserves_its_background_and_adjacent_text() {
    let output = render_truecolor(
        View::new()
            .width(2)
            .height(1)
            .overflow_x(Overflow::Hidden)
            .content_offset_x(1)
            .child(
                View::new()
                    .width(3)
                    .flex_shrink(0.0)
                    .child(Text::default().child(Text::new("👩‍💻X").background_color(Color::Blue))),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue} X{/}"]));
}
