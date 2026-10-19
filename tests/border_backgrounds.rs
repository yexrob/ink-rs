//! Ported from ink test/border-backgrounds.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn border_with_background_color() {
    let output = render_truecolor(
        View::new()
            .border_style(BorderStyle::Single)
            .border_color(Color::White)
            .border_background_color(Color::Blue)
            .child(View::new().width(4).child(Text::new("Test"))),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{fg=white,bg=blue}┌──────────────────────────────────────────────────────────────────────────────────────────────────┐{/}",
            "{fg=white,bg=blue}│{/}Test                                                                                              {fg=white,bg=blue}│{/}",
            "{fg=white,bg=blue}└──────────────────────────────────────────────────────────────────────────────────────────────────┘{/}",
        ]),
    );
}

#[test]
fn border_with_different_background_colors_per_side() {
    let output = render_truecolor(
        View::new()
            .border_style(BorderStyle::Single)
            .border_top_background_color(Color::Red)
            .border_bottom_background_color(Color::Blue)
            .border_left_background_color(Color::Green)
            .border_right_background_color(Color::Yellow)
            .child(View::new().width(4).child(Text::new("Test"))),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=red}┌──────────────────────────────────────────────────────────────────────────────────────────────────┐{/}",
            "{bg=green}│{/}Test                                                                                              {bg=yellow}│{/}",
            "{bg=blue}└──────────────────────────────────────────────────────────────────────────────────────────────────┘{/}",
        ]),
    );
}

#[test]
fn border_background_color_fallback_to_general_border_background_color() {
    let output = render_truecolor(
        View::new()
            .border_style(BorderStyle::Single)
            .border_background_color(Color::Magenta)
            .border_top_background_color(Color::Cyan)
            .child(View::new().width(4).child(Text::new("Test"))),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=cyan}┌──────────────────────────────────────────────────────────────────────────────────────────────────┐{/}",
            "{bg=magenta}│{/}Test                                                                                              {bg=magenta}│{/}",
            "{bg=magenta}└──────────────────────────────────────────────────────────────────────────────────────────────────┘{/}",
        ]),
    );
}

#[test]
fn vertical_border_background_does_not_bleed_into_content_rows() {
    let output = render_truecolor(
        View::new()
            .border_style(BorderStyle::Classic)
            .border_background_color(Color::Cyan)
            .align_self(AlignSelf::FlexStart)
            .width(12)
            .child(Text::new(
                "Text longer than the Box width, so will definitely wrap.",
            )),
        100,
    );
    let rows: Vec<_> = output.lines().collect();
    assert_eq!(rows.len(), 9);
    assert_ansi_eq(rows[1], &styled(&["{bg=cyan}|{/}Text      {bg=cyan}|{/}"]));
    assert_ansi_eq(rows[2], &styled(&["{bg=cyan}|{/}longer    {bg=cyan}|{/}"]));
    assert_ansi_eq(rows[3], &styled(&["{bg=cyan}|{/}than the  {bg=cyan}|{/}"]));
    assert_ansi_eq(rows[4], &styled(&["{bg=cyan}|{/}Box width,{bg=cyan}|{/}"]));
    assert_ansi_eq(rows[5], &styled(&["{bg=cyan}|{/} so will  {bg=cyan}|{/}"]));
    assert_ansi_eq(rows[6], &styled(&["{bg=cyan}|{/}definitely{bg=cyan}|{/}"]));
    assert_ansi_eq(rows[7], &styled(&["{bg=cyan}|{/} wrap.    {bg=cyan}|{/}"]));
}

#[test]
fn foreground_background_and_dim_combine_correctly() {
    let output = render_truecolor(
        View::new()
            .border_top_dim_color(true)
            .border_style(BorderStyle::Single)
            .border_top_color(Color::Red)
            .border_top_background_color(Color::Cyan)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hi")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&["{fg=red,bg=cyan,dim}┌──┐{/}", "│Hi│", "└──┘"]),
    );
}
