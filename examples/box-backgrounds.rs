// Ink examples/box-backgrounds.

use ink_rs::{
    AlignSelf, BorderStyle, Color, FlexDirection, JustifyContent, Options, Text, View, render, rsx,
};

fn main() -> Result<(), ink_rs::Error> {
    let examples = [
        (
            "1. Standard red background (10x3):",
            rsx! {
                View(background_color: Color::Red, width: 10, height: 3) {
                    Text { "Hello" }
                }
            },
        ),
        (
            "2. Blue background with border (12x4):",
            rsx! {
                View(
                    background_color: Color::Blue,
                    border_style: BorderStyle::Round,
                    width: 12,
                    height: 4,
                ) {
                    Text { "Border" }
                }
            },
        ),
        (
            "3. Green background with padding (14x4):",
            rsx! {
                View(background_color: Color::Green, padding: 1, width: 14, height: 4) {
                    Text { "Padding" }
                }
            },
        ),
        (
            "4. Yellow background with center alignment (16x3):",
            rsx! {
                View(
                    background_color: Color::Yellow,
                    width: 16,
                    height: 3,
                    justify_content: JustifyContent::Center,
                ) {
                    Text { "Centered" }
                }
            },
        ),
        (
            "5. Magenta background, column layout (12x5):",
            rsx! {
                View(
                    background_color: Color::Magenta,
                    flex_direction: FlexDirection::Column,
                    width: 12,
                    height: 5,
                ) {
                    Text { "Line 1" }
                    Text { "Line 2" }
                }
            },
        ),
        (
            "6. Hex color background #FF8800 (10x3):",
            rsx! {
                View(
                    background_color: "#FF8800".parse::<Color>().expect("valid hex color"),
                    width: 10,
                    height: 3,
                ) {
                    Text { "Hex" }
                }
            },
        ),
        (
            "7. RGB background rgb(0,255,0) (10x3):",
            rsx! {
                View(background_color: Color::Rgb(0, 255, 0), width: 10, height: 3) {
                    Text { "RGB" }
                }
            },
        ),
        (
            "8. Text inheritance test:",
            rsx! {
                View(background_color: Color::Cyan) {
                    Text { "Inherited " }
                    Text(background_color: Color::Red) { "Override " }
                    Text { "Back to inherited" }
                }
            },
        ),
        (
            "9. Nested background inheritance:",
            rsx! {
                View(background_color: Color::Blue) {
                    Text { "Outer: " }
                    View(background_color: Color::Yellow) {
                        Text { "Inner: " }
                        Text(background_color: Color::Red) { "Deep" }
                    }
                }
            },
        ),
    ];
    let app = rsx! {
        View(flex_direction: FlexDirection::Column, gap: 1) {
            Text(bold: true) { "Box Background Examples:" }
            for (label, example) in examples {
                Text { {label} }
                {example.align_self(AlignSelf::FlexStart)}
            }
            View(margin_top: 1) {
                Text { "Press Ctrl+C to exit" }
            }
        }
    };
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
