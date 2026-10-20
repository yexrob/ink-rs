// Ink examples/borders.

use ink_rs::{BorderStyle, Element, FlexDirection, Options, Text, View, render, rsx};

fn main() -> Result<(), ink_rs::Error> {
    let rows = [
        vec![
            ("single", BorderStyle::Single),
            ("double", BorderStyle::Double),
            ("round", BorderStyle::Round),
            ("bold", BorderStyle::Bold),
        ],
        vec![
            ("singleDouble", BorderStyle::SingleDouble),
            ("doubleSingle", BorderStyle::DoubleSingle),
            ("classic", BorderStyle::Classic),
        ],
    ];
    let app: Element = rsx! {
        View(flex_direction: FlexDirection::Column, padding: 2) {
            for (i, row) in rows.iter().enumerate() {
                View(margin_top: i as i16) {
                    for (i, &(label, border)) in row.iter().enumerate() {
                        View(border_style: border, margin_right: if i + 1 == row.len() { 0 } else { 2 }) {
                            Text { {label} }
                        }
                    }
                }
            }
        }
    }
    .into();
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
