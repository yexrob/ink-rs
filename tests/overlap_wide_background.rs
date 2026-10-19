//! Ported from ink test/overlap-wide-background.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn overwriting_cjk_cell_0_preserves_the_other_cell_s_background() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .child(Text::new("你Z").background_color(Color::Blue))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(0)
                    .top(0)
                    .child(Text::new("X")),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["X{bg=blue} Z{/}"]));
}

#[test]
fn overwriting_cjk_cell_1_preserves_the_other_cell_s_background() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .child(Text::new("你Z").background_color(Color::Blue))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(1)
                    .top(0)
                    .child(Text::new("X")),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue} {/}X{bg=blue}Z{/}"]));
}

#[test]
fn overwriting_joined_emoji_cell_0_preserves_the_other_cell_s_background() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .child(Text::new("👩‍💻Z").background_color(Color::Blue))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(0)
                    .top(0)
                    .child(Text::new("X")),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["X{bg=blue} Z{/}"]));
}

#[test]
fn overwriting_joined_emoji_cell_1_preserves_the_other_cell_s_background() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .child(Text::new("👩‍💻Z").background_color(Color::Blue))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(1)
                    .top(0)
                    .child(Text::new("X")),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue} {/}X{bg=blue}Z{/}"]));
}

#[test]
fn fully_overwriting_a_wide_character_with_xy_replaces_its_styles() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .child(Text::new("你Z").background_color(Color::Blue))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(0)
                    .top(0)
                    .child(Text::new("XY").background_color(Color::Red)),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=red}XY{bg=blue}Z{/}"]));
}

#[test]
fn fully_overwriting_a_wide_character_with_cjk_replaces_its_styles() {
    let output = render_truecolor(
        View::new()
            .width(3)
            .height(1)
            .child(Text::new("你Z").background_color(Color::Blue))
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(0)
                    .top(0)
                    .child(Text::new("好").background_color(Color::Red)),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=red}好{bg=blue}Z{/}"]));
}

#[test]
fn an_overlay_crossing_two_wide_characters_preserves_both_exposed_backgrounds() {
    let output = render_truecolor(
        View::new()
            .width(4)
            .height(1)
            .child(
                Text::default()
                    .child(Text::new("你").background_color(Color::Blue))
                    .child(Text::new("好").background_color(Color::Green)),
            )
            .child(
                View::new()
                    .position(Position::Absolute)
                    .left(1)
                    .top(0)
                    .child(Text::new("XY").background_color(Color::Red)),
            ),
        80,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue} {bg=red}XY{bg=green} {/}"]));
}
