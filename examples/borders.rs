// Ink examples/borders.

use ink_rs::{BorderStyle, Element, FlexDirection, Options, Styled, Text, View, render};

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
    let app: Element = View::new()
        .flex_direction(FlexDirection::Column)
        .padding(2)
        .children(rows.into_iter().enumerate().map(|(i, row)| {
            let last = row.len() - 1;
            View::new()
                .margin_top(i as i16)
                .children(row.into_iter().enumerate().map(|(i, (label, border))| {
                    View::new()
                        .border_style(border)
                        .margin_right(if i == last { 0 } else { 2 })
                        .child(Text::new(label))
                }))
        }))
        .into();
    render(app, Options::default())
        .wait_until_exit()
        .map(|_| ())
}
