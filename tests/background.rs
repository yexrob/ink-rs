//! Ported from ink test/background.tsx @ 26d2c3f.

mod common;

use common::*;
use ink_rs::*;

#[test]
fn static_background_color_is_inherited_by_its_text() {
    let output = render_truecolor(
        Static::new(vec!["A"], |item, _| Text::new(*item).into())
            .width(3)
            .background_color(Color::Blue),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}A  {/}", ""]));
}

#[test]
fn text_inherits_parent_box_background_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Green)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello World")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=green}Hello World{/}"]));
}

#[test]
fn text_explicit_background_color_overrides_inherited() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Red)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello World").background_color(Color::Blue)),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}Hello World{/}"]));
}

#[test]
fn nested_box_background_inheritance() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Red)
            .align_self(AlignSelf::FlexStart)
            .child(
                View::new()
                    .background_color(Color::Blue)
                    .child(Text::new("Hello World")),
            ),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}Hello World{/}"]));
}

#[test]
fn nested_text_inherits_the_nearest_text_background_instead_of_the_box_background() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Red)
            .align_self(AlignSelf::FlexStart)
            .child(
                Text::new("Hello ")
                    .background_color(Color::Blue)
                    .child(Text::new("World")),
            ),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=blue}Hello World{/}"]));
}

#[test]
fn text_without_parent_box_background_has_no_inheritance() {
    let output = render_truecolor(
        View::new()
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello World")),
        100,
    );
    assert_eq!(output, "Hello World");
}

#[test]
fn multiple_text_elements_inherit_same_background() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Yellow)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello "))
            .child(Text::new("World")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=yellow}Hello World{/}"]));
}

#[test]
fn complex_nested_structure_with_background_inheritance() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Yellow)
            .align_self(AlignSelf::FlexStart)
            .child(
                View::new().child(Text::new("Outer: ")).child(
                    View::new()
                        .background_color(Color::Blue)
                        .child(Text::new("Inner: "))
                        .child(Text::new("Explicit").background_color(Color::Red)),
                ),
            ),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&["{bg=yellow}Outer: {bg=blue}Inner: {bg=red}Explicit{/}"]),
    );
}

#[test]
fn box_background_with_standard_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Red)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=red}Hello{/}"]));
}

#[test]
fn box_background_with_hex_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Rgb(255, 0, 0))
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=#ff0000}Hello{/}"]));
}

#[test]
fn box_background_with_rgb_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Rgb(255, 0, 0))
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=#ff0000}Hello{/}"]));
}

#[test]
fn box_background_with_ansi256_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Ansi256(9))
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=9}Hello{/}"]));
}

#[test]
fn box_background_with_wide_characters() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Yellow)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("こんにちは")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=yellow}こんにちは{/}"]));
}

#[test]
fn box_background_with_emojis() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Red)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("🎉🎊")),
        100,
    );
    assert_ansi_eq(&output, &styled(&["{bg=red}🎉🎊{/}"]));
}

#[test]
fn box_background_fills_entire_area_with_standard_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Red)
            .width(10)
            .height(3)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=red}Hello     {/}",
            "{bg=red}          {/}",
            "{bg=red}          {/}",
        ]),
    );
}

#[test]
fn box_background_fills_with_hex_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Rgb(255, 0, 0))
            .width(10)
            .height(3)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=#ff0000}Hello     {/}",
            "{bg=#ff0000}          {/}",
            "{bg=#ff0000}          {/}",
        ]),
    );
}

#[test]
fn box_background_fills_with_rgb_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Rgb(255, 0, 0))
            .width(10)
            .height(3)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=#ff0000}Hello     {/}",
            "{bg=#ff0000}          {/}",
            "{bg=#ff0000}          {/}",
        ]),
    );
}

#[test]
fn box_background_fills_with_ansi256_color() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Ansi256(9))
            .width(10)
            .height(3)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=9}Hello     {/}",
            "{bg=9}          {/}",
            "{bg=9}          {/}",
        ]),
    );
}

#[test]
fn box_background_with_border_fills_content_area() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Cyan)
            .border_style(BorderStyle::Round)
            .width(10)
            .height(5)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hi")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "╭────────╮",
            "│{bg=cyan}Hi      {/}│",
            "│{bg=cyan}        {/}│",
            "│{bg=cyan}        {/}│",
            "╰────────╯",
        ]),
    );
}

#[test]
fn box_background_with_padding_fills_entire_padded_area() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Magenta)
            .padding(1)
            .width(10)
            .height(5)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hi")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=magenta}          {/}",
            "{bg=magenta} Hi       {/}",
            "{bg=magenta}          {/}",
            "{bg=magenta}          {/}",
            "{bg=magenta}          {/}",
        ]),
    );
}

#[test]
fn box_background_with_center_alignment_fills_entire_area() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Blue)
            .width(10)
            .height(3)
            .justify_content(JustifyContent::Center)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hi")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=blue}    Hi    {/}",
            "{bg=blue}          {/}",
            "{bg=blue}          {/}",
        ]),
    );
}

#[test]
fn box_background_with_column_layout_fills_entire_area() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Green)
            .flex_direction(FlexDirection::Column)
            .width(10)
            .height(5)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Line 1"))
            .child(Text::new("Line 2")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{bg=green}Line 1    {/}",
            "{bg=green}Line 2    {/}",
            "{bg=green}          {/}",
            "{bg=green}          {/}",
            "{bg=green}          {/}",
        ]),
    );
}

#[test]
fn box_background_color_fills_full_width_on_every_line_when_text_wraps() {
    let output = render_truecolor(
        View::new()
            .background_color(Color::Red)
            .width(10)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello World!!")),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&["{bg=red}Hello     {/}", "{bg=red}World!!   {/}"]),
    );
}

#[test]
fn text_only_background_color_colors_text_content_but_does_not_fill_box_width() {
    let output = render_truecolor(
        View::new()
            .width(10)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello World!!").background_color(Color::Red)),
        100,
    );
    assert_ansi_eq(
        &output,
        &styled(&["{bg=red}Hello {/}", "{bg=red}World!!{/}"]),
    );
}
