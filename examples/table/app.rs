// Ink examples/table: fixed rows replace faker, without a data dependency.

use ink_rs::{Element, FlexDirection, Percent, Text, View, rsx};

fn row(id: &str, name: &str, email: &str) -> View {
    rsx! {
        View {
            for (width, text) in [(10.0, id), (50.0, name), (40.0, email)] {
                View(width: Percent(width)) {
                    Text { {text.to_owned()} }
                }
            }
        }
    }
}

pub fn app() -> Element {
    let names = [
        "ada", "grace", "linus", "margaret", "alan", "barbara", "donald", "frances", "ken", "radia",
    ];
    rsx! {
        View(flex_direction: FlexDirection::Column, width: Percent(100.0), max_width: 80) {
            {row("ID", "Name", "Email")}
            for (i, name) in names.into_iter().enumerate() {
                {row(&i.to_string(), name, &format!("{name}@example.com")).key(i)}
            }
        }
    }
    .into()
}
