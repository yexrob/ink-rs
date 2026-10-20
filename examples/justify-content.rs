// Ink examples/justify-content.

use ink_rs::{FlexDirection, JustifyContent, Options, Text, View, render, rsx};

fn main() -> Result<(), ink_rs::Error> {
    let modes = [
        ("flex-start", JustifyContent::FlexStart),
        ("flex-end", JustifyContent::FlexEnd),
        ("center", JustifyContent::Center),
        ("space-around", JustifyContent::SpaceAround),
        ("space-between", JustifyContent::SpaceBetween),
        ("space-evenly", JustifyContent::SpaceEvenly),
    ];
    let app = rsx! {
        View(flex_direction: FlexDirection::Column) {
            for (name, mode) in modes {
                View {
                    Text { "[" }
                    View(justify_content: mode, width: 20, height: 1) {
                        Text { "X" }
                        Text { "Y" }
                    }
                    Text { "] {name}" }
                }
            }
        }
    };
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
