// Ink examples/box-backgrounds.

use ink_rs::{
    AlignSelf, BorderStyle, Color, FlexDirection, JustifyContent, Options, Styled, Text, View,
    render,
};

fn main() -> Result<(), ink_rs::Error> {
    let examples = [
        (
            "1. Standard red background (10x3):",
            View::new()
                .background_color(Color::Red)
                .width(10)
                .height(3)
                .child(Text::new("Hello")),
        ),
        (
            "2. Blue background with border (12x4):",
            View::new()
                .background_color(Color::Blue)
                .border_style(BorderStyle::Round)
                .width(12)
                .height(4)
                .child(Text::new("Border")),
        ),
        (
            "3. Green background with padding (14x4):",
            View::new()
                .background_color(Color::Green)
                .padding(1)
                .width(14)
                .height(4)
                .child(Text::new("Padding")),
        ),
        (
            "4. Yellow background with center alignment (16x3):",
            View::new()
                .background_color(Color::Yellow)
                .width(16)
                .height(3)
                .justify_content(JustifyContent::Center)
                .child(Text::new("Centered")),
        ),
        (
            "5. Magenta background, column layout (12x5):",
            View::new()
                .background_color(Color::Magenta)
                .flex_direction(FlexDirection::Column)
                .width(12)
                .height(5)
                .child(Text::new("Line 1"))
                .child(Text::new("Line 2")),
        ),
        (
            "6. Hex color background #FF8800 (10x3):",
            View::new()
                .background_color("#FF8800".parse::<Color>().expect("valid hex color"))
                .width(10)
                .height(3)
                .child(Text::new("Hex")),
        ),
        (
            "7. RGB background rgb(0,255,0) (10x3):",
            View::new()
                .background_color(Color::Rgb(0, 255, 0))
                .width(10)
                .height(3)
                .child(Text::new("RGB")),
        ),
        (
            "8. Text inheritance test:",
            View::new()
                .background_color(Color::Cyan)
                .child(Text::new("Inherited "))
                .child(Text::new("Override ").background_color(Color::Red))
                .child(Text::new("Back to inherited")),
        ),
        (
            "9. Nested background inheritance:",
            View::new()
                .background_color(Color::Blue)
                .child(Text::new("Outer: "))
                .child(
                    View::new()
                        .background_color(Color::Yellow)
                        .child(Text::new("Inner: "))
                        .child(Text::new("Deep").background_color(Color::Red)),
                ),
        ),
    ];
    let mut app = View::new()
        .flex_direction(FlexDirection::Column)
        .gap(1)
        .child(Text::new("Box Background Examples:").bold(true));
    for (label, example) in examples {
        app = app
            .child(Text::new(label))
            .child(example.align_self(AlignSelf::FlexStart));
    }
    app = app.child(
        View::new()
            .margin_top(1)
            .child(Text::new("Press Ctrl+C to exit")),
    );
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
