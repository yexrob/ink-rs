//! Ported from ink test/borders.tsx @ 26d2c3f.

mod common;

use common::{render, *};
use ink_rs::*;

#[test]
fn single_node_full_width_box() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .child(Text::new("Hello World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│Hello World                                                                                       │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ])
    );
}

#[test]
fn single_node_full_width_box_with_colorful_border() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .border_color(Color::Green)
            .child(Text::new("Hello World")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{fg=green}╭──────────────────────────────────────────────────────────────────────────────────────────────────╮{/}",
            "{fg=green}│{/}Hello World                                                                                       {fg=green}│{/}",
            "{fg=green}╰──────────────────────────────────────────────────────────────────────────────────────────────────╯{/}",
        ]),
    );
}

#[test]
fn single_node_fit_content_box() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello World")),
    );
    assert_eq!(
        output,
        lines(&["╭───────────╮", "│Hello World│", "╰───────────╯",])
    );
}

#[test]
fn single_node_fit_content_box_with_wide_characters() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("こんにちは")),
    );
    assert_eq!(
        output,
        lines(&["╭──────────╮", "│こんにちは│", "╰──────────╯",])
    );
}

#[test]
fn single_node_fit_content_box_with_emojis() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("🌊🌊")),
    );
    assert_eq!(output, lines(&["╭────╮", "│🌊🌊│", "╰────╯",]));
}

#[test]
fn single_node_fit_content_box_with_variation_selector_emojis() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("🌡️⚠️✅")),
    );
    assert_eq!(output, lines(&["╭──────╮", "│🌡️⚠️✅│", "╰──────╯",]));
}

#[test]
fn single_node_fixed_width_box() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(20)
            .child(Text::new("Hello World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────╮",
            "│Hello World       │",
            "╰──────────────────╯",
        ])
    );
}

#[test]
fn single_node_fixed_width_and_height_box() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(20)
            .height(20)
            .child(Text::new("Hello World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────╮",
            "│Hello World       │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "╰──────────────────╯",
        ])
    );
}

#[test]
fn single_node_box_with_padding() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .padding(1)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭─────────────╮",
            "│             │",
            "│ Hello World │",
            "│             │",
            "╰─────────────╯",
        ])
    );
}

#[test]
fn single_node_box_with_horizontal_alignment() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(20)
            .justify_content(JustifyContent::Center)
            .child(Text::new("Hello World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────╮",
            "│   Hello World    │",
            "╰──────────────────╯",
        ])
    );
}

#[test]
fn single_node_box_with_vertical_alignment() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .height(20)
            .align_items(AlignItems::Center)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭───────────╮",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│Hello World│",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "╰───────────╯",
        ])
    );
}

#[test]
fn single_node_box_with_wrapping() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(10)
            .child(Text::new("Hello World")),
    );
    assert_eq!(
        output,
        lines(&["╭────────╮", "│Hello   │", "│World   │", "╰────────╯",])
    );
}

#[test]
fn multiple_nodes_full_width_box() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .child(Text::new("Hello ").child("World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│Hello World                                                                                       │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ])
    );
}

#[test]
fn multiple_nodes_full_width_box_with_colorful_border() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .border_color(Color::Green)
            .child(Text::new("Hello ").child("World")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{fg=green}╭──────────────────────────────────────────────────────────────────────────────────────────────────╮{/}",
            "{fg=green}│{/}Hello World                                                                                       {fg=green}│{/}",
            "{fg=green}╰──────────────────────────────────────────────────────────────────────────────────────────────────╯{/}",
        ]),
    );
}

#[test]
fn multiple_nodes_fit_content_box() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello ").child("World")),
    );
    assert_eq!(
        output,
        lines(&["╭───────────╮", "│Hello World│", "╰───────────╯",])
    );
}

#[test]
fn multiple_nodes_fixed_width_box() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(20)
            .child(Text::new("Hello ").child("World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────╮",
            "│Hello World       │",
            "╰──────────────────╯",
        ])
    );
}

#[test]
fn multiple_nodes_fixed_width_and_height_box() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(20)
            .height(20)
            .child(Text::new("Hello ").child("World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────╮",
            "│Hello World       │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "╰──────────────────╯",
        ])
    );
}

#[test]
fn multiple_nodes_box_with_padding() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .padding(1)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello ").child("World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭─────────────╮",
            "│             │",
            "│ Hello World │",
            "│             │",
            "╰─────────────╯",
        ])
    );
}

#[test]
fn multiple_nodes_box_with_horizontal_alignment() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(20)
            .justify_content(JustifyContent::Center)
            .child(Text::new("Hello ").child("World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────╮",
            "│   Hello World    │",
            "╰──────────────────╯",
        ])
    );
}

#[test]
fn multiple_nodes_box_with_vertical_alignment() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .height(20)
            .align_items(AlignItems::Center)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("Hello ").child("World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭───────────╮",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│Hello World│",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "╰───────────╯",
        ])
    );
}

#[test]
fn multiple_nodes_box_with_wrapping() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(10)
            .child(Text::new("Hello ").child("World")),
    );
    assert_eq!(
        output,
        lines(&["╭────────╮", "│Hello   │", "│World   │", "╰────────╯",])
    );
}

#[test]
fn multiple_nodes_box_with_wrapping_and_long_first_node() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(10)
            .child(Text::new("Helloooooo").child(" World")),
    );
    assert_eq!(
        output,
        lines(&["╭────────╮", "│Helloooo│", "│oo World│", "╰────────╯",])
    );
}

#[test]
fn multiple_nodes_box_with_wrapping_and_very_long_first_node() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(10)
            .child(Text::new("Hellooooooooooooo").child(" World")),
    );
    assert_eq!(
        output,
        lines(&[
            "╭────────╮",
            "│Helloooo│",
            "│oooooooo│",
            "│o World │",
            "╰────────╯",
        ])
    );
}

#[test]
fn nested_boxes() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .width(40)
            .padding(1)
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .justify_content(JustifyContent::Center)
                    .padding(1)
                    .child(Text::new("Hello World")),
            ),
    );
    assert_eq!(
        output,
        lines(&[
            "╭──────────────────────────────────────╮",
            "│                                      │",
            "│ ╭─────────────╮                      │",
            "│ │             │                      │",
            "│ │ Hello World │                      │",
            "│ │             │                      │",
            "│ ╰─────────────╯                      │",
            "│                                      │",
            "╰──────────────────────────────────────╯",
        ])
    );
}

#[test]
fn nested_boxes_fit_content_box_with_wide_characters_on_flex_direction_row() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("ミスター")),
            )
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("スポック")),
            )
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("カーク船長")),
            ),
    );
    assert_eq!(
        output,
        lines(&[
            "╭────────────────────────────────╮",
            "│╭────────╮╭────────╮╭──────────╮│",
            "││ミスター││スポック││カーク船長││",
            "│╰────────╯╰────────╯╰──────────╯│",
            "╰────────────────────────────────╯",
        ])
    );
}

#[test]
fn nested_boxes_fit_content_box_with_emojis_on_flex_direction_row() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("🦾")),
            )
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("🌏")),
            )
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("😋")),
            ),
    );
    assert_eq!(
        output,
        lines(&[
            "╭────────────╮",
            "│╭──╮╭──╮╭──╮│",
            "││🦾││🌏││😋││",
            "│╰──╯╰──╯╰──╯│",
            "╰────────────╯",
        ])
    );
}

#[test]
fn nested_boxes_fit_content_box_with_wide_characters_on_flex_direction_column() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("ミスター")),
            )
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("スポック")),
            )
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("カーク船長")),
            ),
    );
    assert_eq!(
        output,
        lines(&[
            "╭────────────╮",
            "│╭──────────╮│",
            "││ミスター  ││",
            "│╰──────────╯│",
            "│╭──────────╮│",
            "││スポック  ││",
            "│╰──────────╯│",
            "│╭──────────╮│",
            "││カーク船長││",
            "│╰──────────╯│",
            "╰────────────╯",
        ])
    );
}

#[test]
fn nested_boxes_fit_content_box_with_emojis_on_flex_direction_column() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .flex_direction(FlexDirection::Column)
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("🦾")),
            )
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("🌏")),
            )
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .child(Text::new("😋")),
            ),
    );
    assert_eq!(
        output,
        lines(&[
            "╭────╮",
            "│╭──╮│",
            "││🦾││",
            "│╰──╯│",
            "│╭──╮│",
            "││🌏││",
            "│╰──╯│",
            "│╭──╮│",
            "││😋││",
            "│╰──╯│",
            "╰────╯",
        ])
    );
}

#[test]
fn render_border_after_update() {
    fn example(_: &mut Scope, value: &Option<Color>) -> Element {
        let mut view = View::new().border_style(BorderStyle::Round);
        if let Some(color) = value {
            view = view.border_color(*color);
        }
        view.child(Text::new("Hello World")).into()
    }
    let mut instance = testing::render(component(example, None));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&[
            "╭──────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│Hello World                                                                                       │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ]),
    );
    instance.rerender(component(example, Some(Color::Green)));
    assert_ansi_eq(
        instance.last_frame(),
        &styled(&[
            "{fg=green}╭──────────────────────────────────────────────────────────────────────────────────────────────────╮{/}",
            "{fg=green}│{/}Hello World                                                                                       {fg=green}│{/}",
            "{fg=green}╰──────────────────────────────────────────────────────────────────────────────────────────────────╯{/}",
        ]),
    );
    instance.rerender(component(example, None));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&[
            "╭──────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│Hello World                                                                                       │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ]),
    );
}

#[test]
fn render_border_edge_changes_after_update_when_border_style_is_unchanged() {
    fn example(_: &mut Scope, value: &Option<bool>) -> Element {
        let mut view = View::new()
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart);
        if let Some(top) = value {
            view = view.border_top(*top);
        }
        view.child(Text::new("Content")).into()
    }
    let mut instance = testing::render(component(example, None));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&["╭───────╮", "│Content│", "╰───────╯"]),
    );
    instance.rerender(component(example, Some(false)));
    assert_ansi_eq(instance.last_frame(), &lines(&["│Content│", "╰───────╯"]));
    instance.rerender(component(example, None));
    assert_ansi_eq(
        instance.last_frame(),
        &lines(&["╭───────╮", "│Content│", "╰───────╯"]),
    );
}

#[test]
fn hide_top_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_top(false)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_eq!(
        output,
        lines(&["Above", "│Content│", "╰───────╯", "Below",])
    );
}

#[test]
fn hide_bottom_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_bottom(false)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_eq!(
        output,
        lines(&["Above", "╭───────╮", "│Content│", "Below",])
    );
}

#[test]
fn hide_top_and_bottom_borders() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_top(false)
                    .border_bottom(false)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_eq!(output, lines(&["Above", "│Content│", "Below",]));
}

#[test]
fn hide_left_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_left(false)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_eq!(
        output,
        lines(&["Above", "───────╮", "Content│", "───────╯", "Below",])
    );
}

#[test]
fn hide_right_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_right(false)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_eq!(
        output,
        lines(&["Above", "╭───────", "│Content", "╰───────", "Below",])
    );
}

#[test]
fn hide_left_and_right_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_left(false)
                    .border_right(false)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_eq!(
        output,
        lines(&["Above", "───────", "Content", "───────", "Below",])
    );
}

#[test]
fn hide_all_borders() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_top(false)
                    .border_bottom(false)
                    .border_left(false)
                    .border_right(false)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_eq!(output, lines(&["Above", "Content", "Below",]));
}

#[test]
fn change_color_of_top_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_top_color(Color::Green)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "Above",
            "{fg=green}╭───────╮{/}",
            "│Content│",
            "╰───────╯",
            "Below",
        ]),
    );
}

#[test]
fn change_color_of_bottom_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_bottom_color(Color::Green)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "Above",
            "╭───────╮",
            "│Content│",
            "{fg=green}╰───────╯{/}",
            "Below",
        ]),
    );
}

#[test]
fn change_color_of_left_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_left_color(Color::Green)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "Above",
            "╭───────╮",
            "{fg=green}│{/}Content│",
            "╰───────╯",
            "Below",
        ]),
    );
}

#[test]
fn change_color_of_right_border() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_style(BorderStyle::Round)
                    .border_right_color(Color::Green)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "Above",
            "╭───────╮",
            "│Content{fg=green}│{/}",
            "╰───────╯",
            "Below",
        ]),
    );
}

#[test]
fn custom_border_style() {
    let output = render(
        View::new()
            .border_style(BorderStyle::Custom(BoxChars {
                top_left: '↘',
                top: '↓',
                top_right: '↙',
                left: '→',
                bottom_left: '↗',
                bottom: '↑',
                bottom_right: '↖',
                right: '←',
            }))
            .child(Text::new("Content")),
    );
    assert_eq!(
        output,
        lines(&[
            "↘↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↙",
            "→Content                                                                                           ←",
            "↗↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↖",
        ])
    );
}

#[test]
fn dim_border_color() {
    let output = render(
        View::new()
            .border_dim_color(true)
            .border_style(BorderStyle::Round)
            .child(Text::new("Content")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{dim}╭──────────────────────────────────────────────────────────────────────────────────────────────────╮{/}",
            "{dim}│{/}Content                                                                                           {dim}│{/}",
            "{dim}╰──────────────────────────────────────────────────────────────────────────────────────────────────╯{/}",
        ]),
    );
}

#[test]
fn dim_top_border_color() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_top_dim_color(true)
                    .border_style(BorderStyle::Round)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "Above",
            "{dim}╭───────╮{/}",
            "│Content│",
            "╰───────╯",
            "Below",
        ]),
    );
}

#[test]
fn dim_bottom_border_color() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_bottom_dim_color(true)
                    .border_style(BorderStyle::Round)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "Above",
            "╭───────╮",
            "│Content│",
            "{dim}╰───────╯{/}",
            "Below",
        ]),
    );
}

#[test]
fn dim_left_border_color() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_left_dim_color(true)
                    .border_style(BorderStyle::Round)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "Above",
            "╭───────╮",
            "{dim}│{/}Content│",
            "╰───────╯",
            "Below",
        ]),
    );
}

#[test]
fn dim_right_border_color() {
    let output = render(
        View::new()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .child(Text::new("Above"))
            .child(
                View::new()
                    .border_right_dim_color(true)
                    .border_style(BorderStyle::Round)
                    .child(Text::new("Content")),
            )
            .child(Text::new("Below")),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "Above",
            "╭───────╮",
            "│Content{dim}│{/}",
            "╰───────╯",
            "Below",
        ]),
    );
}

#[test]
fn border_dim_color_does_not_dim_styled_child_text_touching_left_edge() {
    let output = render(
        View::new()
            .border_dim_color(true)
            .border_style(BorderStyle::Round)
            .align_self(AlignSelf::FlexStart)
            .child(Text::new("styled text").bold(true).color(Color::Blue)),
    );
    assert_ansi_eq(
        &output,
        &styled(&[
            "{dim}╭───────────╮{/}",
            "{dim}│{fg=blue,bold}styled text{dim}│{/}",
            "{dim}╰───────────╯{/}",
        ]),
    );
}
