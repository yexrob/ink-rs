// Ink examples/border-backgrounds.

use ink_rs::{BorderStyle, Color, FlexDirection, Options, Text, View, render, rsx};

fn main() -> Result<(), ink_rs::Error> {
    let app = rsx! {
        View(flex_direction: FlexDirection::Column, gap: 1) {
            View(
                border_style: BorderStyle::Round,
                border_color: Color::White,
                border_background_color: Color::Blue,
                padding: 1,
            ) {
                Text { "Box with blue background on white border" }
            }
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Black,
                border_background_color: Color::Yellow,
                padding: 1,
            ) {
                Text { "Box with yellow background on black border" }
            }
            View(
                border_style: BorderStyle::Double,
                border_top_color: Color::Red,
                border_top_background_color: Color::Green,
                border_bottom_color: Color::Blue,
                border_bottom_background_color: Color::Yellow,
                border_left_color: Color::Cyan,
                border_left_background_color: Color::Magenta,
                border_right_color: Color::White,
                border_right_background_color: Color::Red,
                padding: 1,
            ) {
                Text { "Box with different colors per side" }
            }
            View(
                border_style: BorderStyle::Classic,
                border_dim_color: true,
                border_background_color: Color::Rgb(128, 0, 128),
                border_color: Color::White,
                padding: 1,
            ) {
                Text { "Box with dimmed RGB purple background on border" }
            }
            View(
                border_style: BorderStyle::Bold,
                border_color: "#FF00FF".parse::<Color>().expect("valid hex color"),
                border_background_color: "#00FF00".parse::<Color>().expect("valid hex color"),
                padding: 1,
            ) {
                Text { "Box with hex color backgrounds" }
            }
        }
    };
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
