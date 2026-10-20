// Ink examples/subprocess-output.

use std::cell::RefCell;
use std::rc::Rc;

use ink_rs::{Element, FlexDirection, Scope, Text, View, component, rsx};
use tokio::sync::mpsc::UnboundedReceiver;

pub enum Chunk {
    Bytes(Vec<u8>),
    /// The command could not start; Ink shows `error.message`.
    Failed(String),
}

#[derive(Clone)]
struct Source(Rc<RefCell<Option<UnboundedReceiver<Chunk>>>>);

impl PartialEq for Source {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Default)]
struct Output {
    pending: Vec<u8>,
    text: String,
}

impl Output {
    fn push(&mut self, bytes: &[u8]) {
        let bytes = [self.pending.as_slice(), bytes].concat();
        self.pending.clear();
        let mut consumed = 0;
        for chunk in bytes.utf8_chunks() {
            consumed += chunk.valid().len() + chunk.invalid().len();
            self.text.push_str(chunk.valid());
            if !chunk.invalid().is_empty() {
                if consumed == bytes.len()
                    && std::str::from_utf8(chunk.invalid())
                        .expect_err("invalid UTF-8")
                        .error_len()
                        .is_none()
                {
                    self.pending.extend_from_slice(chunk.invalid());
                } else {
                    self.text.push('\u{fffd}');
                }
            }
        }
        let lines: Vec<_> = self.text.split('\n').collect();
        self.text = lines[lines.len().saturating_sub(5)..].join("\n");
    }

    fn finish(&mut self) {
        if !self.pending.is_empty() {
            self.pending.clear();
            self.push("\u{fffd}".as_bytes());
        }
    }
}

/// Renders the last lines of a byte stream.
pub fn app(source: UnboundedReceiver<Chunk>) -> Element {
    component(
        subprocess_output,
        Source(Rc::new(RefCell::new(Some(source)))),
    )
}

fn subprocess_output(cx: &mut Scope, source: &Source) -> Element {
    let output = cx.use_state(Output::default);
    let source = source.clone();
    cx.use_future((), move || async move {
        let Some(mut receiver) = source.0.borrow_mut().take() else {
            return;
        };
        while let Some(chunk) = receiver.recv().await {
            output.update(|output| match chunk {
                Chunk::Bytes(bytes) => output.push(&bytes),
                Chunk::Failed(message) => output.text = message,
            });
        }
        output.update(Output::finish);
    });
    rsx! {
        View(flex_direction: FlexDirection::Column, padding: 1) {
            Text { "Command output:" }
            View(margin_top: 1) {
                Text { {strip_ansi(&output.get().text)} }
            }
        }
    }
    .into()
}

/// Removes CSI, OSC and two-byte escape sequences. An unfinished sequence at
/// the end is hidden until a later chunk completes it.
pub fn strip_ansi(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        let kind = match ch {
            '\u{1b}' => chars.next(),
            '\u{9b}' => Some('['),
            _ => {
                output.push(ch);
                continue;
            }
        };
        let complete = match kind {
            Some('[') => chars.by_ref().any(|ch| ('\u{40}'..='\u{7e}').contains(&ch)),
            Some(']') => {
                let mut escape = false;
                chars.by_ref().any(|ch| {
                    let end = ch == '\u{7}' || (escape && ch == '\\');
                    escape = ch == '\u{1b}';
                    end
                })
            }
            Some(_) => true,
            None => false,
        };
        if !complete {
            break;
        }
    }
    output
}
