// Ink examples/cursor-ime: use the renderer's widths to anchor the IME.

use ink_rs::{
    CursorPosition, Element, FlexDirection, Scope, Text, TextWrap, View, component, rsx, text_width,
};
use unicode_segmentation::UnicodeSegmentation;

pub fn app() -> Element {
    component(editor, ())
}

fn editor(cx: &mut Scope, _: &()) -> Element {
    let text = cx.use_state(String::new);
    let columns = usize::from(cx.use_window_size().columns);
    cx.use_input(true, move |input, key| {
        text.update(|text| {
            if key.backspace || key.delete {
                let end = text
                    .grapheme_indices(true)
                    .next_back()
                    .map_or(0, |(i, _)| i);
                text.truncate(end);
            } else if !key.ctrl && !key.meta && !key.enter {
                text.push_str(input);
            }
        });
    });
    let heading = "Type Korean (Ctrl+C to exit):";
    let prompt = format!("> {}", text.get());
    // The extra cell moves a cursor at the right edge onto the following row.
    let mut cursor = CursorPosition { x: 0, y: 0 };
    for grapheme in format!("{heading}\n{prompt} ").graphemes(true) {
        if grapheme == "\n" {
            cursor.x = 0;
            cursor.y += 1;
            continue;
        }
        let width = text_width(grapheme);
        if cursor.x + width > columns {
            cursor.x = 0;
            cursor.y += 1;
        }
        cursor.x += width;
    }
    cursor.x = cursor.x.saturating_sub(1);
    cx.use_cursor(Some(cursor));
    rsx! {
        View(flex_direction: FlexDirection::Column) {
            Text(wrap: TextWrap::Hard) { {heading} }
            // Reserve the cursor cell in layout as well as in the calculation.
            Text(wrap: TextWrap::Hard) { "{prompt} " }
        }
    }
    .into()
}
