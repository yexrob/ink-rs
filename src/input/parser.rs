//! Port of Ink src/input-parser.ts. UTF-8 carry replaces Node's
//! stdin.setEncoding('utf8') (src/components/App.tsx:397).

const PASTE_START: &str = "\x1b[200~";
const PASTE_END: &str = "\x1b[201~";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InputEvent {
    Text(String),
    Paste(String),
}

#[derive(Default)]
pub(crate) struct InputParser {
    pending: String,
    utf8_tail: Vec<u8>,
}

impl InputParser {
    /// Decode a read and emit complete tokens. An incomplete UTF-8 character,
    /// escape sequence or paste remains buffered for the next read.
    pub(crate) fn push(&mut self, bytes: &[u8]) -> Vec<InputEvent> {
        self.utf8_tail.extend_from_slice(bytes);
        let mut consumed = 0;
        for chunk in self.utf8_tail.utf8_chunks() {
            self.pending.push_str(chunk.valid());
            consumed += chunk.valid().len();
            if !chunk.invalid().is_empty() {
                // A truncated valid prefix must wait; malformed subsequences
                // use the same maximal-subpart replacement as StringDecoder.
                if std::str::from_utf8(&self.utf8_tail[consumed..])
                    .is_err_and(|error| error.error_len().is_none())
                {
                    break;
                }
                self.pending.push('\u{fffd}');
                consumed += chunk.invalid().len();
            }
        }
        self.utf8_tail.drain(..consumed);

        let mut events = Vec::new();
        let mut index = 0;
        while index < self.pending.len() {
            let Some(offset) = self.pending[index..].find('\x1b') else {
                split_control_bytes(&self.pending[index..], &mut events);
                index = self.pending.len();
                break;
            };
            let escape = index + offset;
            split_control_bytes(&self.pending[index..escape], &mut events);
            index = escape;
            let Some(length) = escape_sequence_len(&self.pending[escape..]) else {
                break;
            };
            let end = escape + length;
            if &self.pending[escape..end] == PASTE_START {
                let Some(offset) = self.pending[end..].find(PASTE_END) else {
                    break;
                };
                events.push(InputEvent::Paste(self.pending[end..end + offset].into()));
                index = end + offset + PASTE_END.len();
            } else {
                events.push(InputEvent::Text(self.pending[escape..end].into()));
                index = end;
            }
        }
        self.pending.drain(..index);
        events
    }

    pub(crate) fn has_pending_escape(&self) -> bool {
        // Do not start an escape deadline while assembling a paste marker or
        // waiting for its end (src/input-parser.ts:277-285).
        self.pending.starts_with('\x1b')
            && !self.pending.starts_with(PASTE_START)
            && self.pending != "\x1b[200"
    }

    pub(crate) fn flush_pending_escape(&mut self) -> Option<String> {
        self.pending
            .starts_with('\x1b')
            .then(|| std::mem::take(&mut self.pending))
    }

    pub(crate) fn reset(&mut self) {
        self.pending.clear();
        self.utf8_tail.clear();
    }
}

enum ControlSequence {
    Complete(usize),
    Pending,
    Invalid,
}

fn control_sequence(input: &str, prefix: usize) -> ControlSequence {
    let bytes = input.as_bytes();
    let Some(&kind) = bytes.get(prefix) else {
        return ControlSequence::Pending;
    };
    if kind != b'[' && kind != b'O' {
        return ControlSequence::Invalid;
    }
    let payload = prefix + 1;
    for (offset, &byte) in bytes[payload..].iter().enumerate() {
        // Preserve Linux-console ESC[[ keys and rxvt shifted editing keys
        // whose '$' is a final byte (src/input-parser.ts:41-50).
        if kind == b'[' && offset == 0 && byte == b'[' {
            continue;
        }
        let rxvt_shift =
            kind == b'[' && offset == 1 && byte == b'$' && b"235678".contains(&bytes[payload]);
        if rxvt_shift || (0x40..=0x7e).contains(&byte) {
            return ControlSequence::Complete(payload + offset + 1);
        }
        if if kind == b'[' {
            (0x20..=0x3f).contains(&byte)
        } else {
            byte == b';' || byte.is_ascii_digit()
        } {
            continue;
        }
        return ControlSequence::Invalid;
    }
    ControlSequence::Pending
}

pub(crate) fn is_complete_control_sequence(input: &str) -> bool {
    input.starts_with('\x1b')
        && matches!(
            control_sequence(input, if input.starts_with("\x1b\x1b") { 2 } else { 1 }),
            ControlSequence::Complete(end) if end == input.len()
        )
}

fn escape_sequence_len(input: &str) -> Option<usize> {
    let double = input.starts_with("\x1b\x1b");
    match control_sequence(input, if double { 2 } else { 1 }) {
        ControlSequence::Pending => None,
        ControlSequence::Complete(end) => Some(end),
        ControlSequence::Invalid if double => Some(2),
        // A char replaces Ink's surrogate-pair length calculation
        // (src/input-parser.ts:125-141), without splitting UTF-8.
        ControlSequence::Invalid => input[1..].chars().next().map(|c| 1 + c.len_utf8()),
    }
}

fn split_control_bytes(text: &str, events: &mut Vec<InputEvent>) {
    let mut start = 0;
    for (index, byte) in text.bytes().enumerate() {
        // CR and TAB stay inside text runs; BS, DEL and Ctrl+C must reach
        // key handling as individual events (src/input-parser.ts:185-210).
        if matches!(byte, 0x7f | 0x08 | 0x03) {
            if start < index {
                events.push(InputEvent::Text(text[start..index].into()));
            }
            events.push(InputEvent::Text(text[index..index + 1].into()));
            start = index + 1;
        }
    }
    if start < text.len() {
        events.push(InputEvent::Text(text[start..].into()));
    }
}

#[cfg(test)]
mod tests;
