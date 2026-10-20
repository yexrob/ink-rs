// Ink examples/chat.

use ink_rs::{Element, FlexDirection, Scope, Styled, Text, View, component};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Default)]
struct Chat {
    input: String,
    messages: Vec<String>,
}

pub fn app() -> Element {
    component(chat, ())
}

fn chat(cx: &mut Scope, _: &()) -> Element {
    let chat = cx.use_state(Chat::default);
    cx.use_input(true, move |input, key| {
        chat.update(|chat| {
            if key.enter {
                if !chat.input.is_empty() {
                    chat.messages
                        .push(format!("User: {}", std::mem::take(&mut chat.input)));
                }
            } else if key.backspace || key.delete {
                let end = chat
                    .input
                    .grapheme_indices(true)
                    .next_back()
                    .map_or(0, |(i, _)| i);
                chat.input.truncate(end);
            } else if !key.ctrl && !key.meta {
                chat.input.push_str(input);
            }
        });
    });
    let chat = chat.get();
    View::new()
        .flex_direction(FlexDirection::Column)
        .padding(1)
        .child(
            View::new().flex_direction(FlexDirection::Column).children(
                chat.messages
                    .into_iter()
                    .enumerate()
                    .map(|(i, text)| Text::new(text).key(i)),
            ),
        )
        .child(
            View::new()
                .margin_top(1)
                .child(Text::new(format!("Enter your message: {}", chat.input))),
        )
        .into()
}
