//! Port of Ink src/input-parser.ts. UTF-8 carry replaces Node's
//! stdin.setEncoding('utf8') (src/components/App.tsx:397).

use super::{MouseButton, MouseEvent, MouseKind};

const PASTE_START: &str = "\x1b[200~";
const PASTE_END: &str = "\x1b[201~";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InputEvent {
    Text(String),
    Paste(String),
    Mouse(MouseEvent),
    StringSequence(String),
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
            match string_sequence(&self.pending[escape..]) {
                ControlSequence::Complete(length) => {
                    let end = escape + length;
                    events.push(InputEvent::StringSequence(self.pending[escape..end].into()));
                    index = end;
                    continue;
                }
                ControlSequence::Pending => break,
                ControlSequence::Invalid => {}
            }
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
                let sequence = &self.pending[escape..end];
                match mouse_report(sequence) {
                    Some(report) => events.extend(report.map(InputEvent::Mouse)),
                    None => events.push(InputEvent::Text(sequence.into())),
                }
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
        if self.pending.starts_with("\x1b[<") {
            self.pending.clear();
            return None;
        }
        if is_string_start(&self.pending) {
            return Some(self.pending.drain(..2).collect());
        }
        self.pending
            .starts_with('\x1b')
            .then(|| std::mem::take(&mut self.pending))
    }

    pub(crate) fn reset(&mut self) {
        self.pending.clear();
        self.utf8_tail.clear();
    }
}

/// Decodes an SGR mouse report: `None` when `sequence` is not one, `Some(None)`
/// for buttons 8 to 11 (back, forward), which ink-rs drops.
fn mouse_report(sequence: &str) -> Option<Option<MouseEvent>> {
    let payload = sequence.strip_prefix("\x1b[<")?;
    let release = payload.ends_with('m');
    let payload = payload.strip_suffix(if release { 'm' } else { 'M' })?;
    let mut fields = payload.split(';');
    let mut number = || {
        let field = fields.next()?;
        if field.is_empty() || !field.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        field.parse::<u32>().ok()
    };
    let button = number()?;
    let x = i32::try_from(number()?.checked_sub(1)?).ok()?;
    let y = i32::try_from(number()?.checked_sub(1)?).ok()?;
    if fields.next().is_some() {
        return None;
    }
    if button & 128 != 0 {
        return Some(None);
    }
    let identity = match button & 3 {
        0 => Some(MouseButton::Left),
        1 => Some(MouseButton::Middle),
        2 => Some(MouseButton::Right),
        _ => None,
    };
    let kind = if button & 64 != 0 {
        match button & 3 {
            0 => MouseKind::WheelUp,
            1 => MouseKind::WheelDown,
            2 => MouseKind::WheelLeft,
            _ => MouseKind::WheelRight,
        }
    } else if release {
        MouseKind::Release(identity.unwrap_or(MouseButton::Left))
    } else if let Some(identity) = identity {
        if button & 32 != 0 {
            MouseKind::Drag(identity)
        } else {
            MouseKind::Press(identity)
        }
    } else {
        MouseKind::Move
    };
    Some(Some(MouseEvent {
        kind,
        x,
        y,
        ctrl: button & 16 != 0,
        shift: button & 4 != 0,
        meta: button & 8 != 0,
    }))
}

enum ControlSequence {
    Complete(usize),
    Pending,
    Invalid,
}

fn is_string_start(input: &str) -> bool {
    input.starts_with('\x1b')
        && input
            .as_bytes()
            .get(1)
            .is_some_and(|byte| b"P]_X^".contains(byte))
}

fn string_sequence(input: &str) -> ControlSequence {
    if !is_string_start(input) {
        return ControlSequence::Invalid;
    }
    let bytes = input.as_bytes();
    for (index, &byte) in bytes.iter().enumerate().skip(2) {
        if byte == 0x07 {
            return ControlSequence::Complete(index + 1);
        }
        if byte == 0x1b {
            return match bytes.get(index + 1) {
                Some(b'\\') => ControlSequence::Complete(index + 2),
                None => ControlSequence::Pending,
                _ => ControlSequence::Invalid,
            };
        }
        // Reply payloads are printable ASCII. Controls and typed Unicode
        // abandon the attempt so an Alt introducer cannot swallow those keys.
        if !(0x20..=0x7e).contains(&byte) {
            return ControlSequence::Invalid;
        }
    }
    ControlSequence::Pending
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
        // BS, DEL and Ctrl+C reach key handling as individual events
        // (src/input-parser.ts:185-210). Ink keeps CR and TAB inside text runs
        // for pasted text; splitting them keeps Enter and Tab keys read together
        // apart, and bracketed paste keeps pasted text whole.
        if matches!(byte, 0x7f | 0x08 | 0x03 | b'\r' | b'\t') {
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
