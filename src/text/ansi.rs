//! Splits strings into text and ECMA-48 control sequences, in 7-bit (ESC)
//! and 8-bit (C1) forms.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Token<'a> {
    Text(&'a str),
    /// `ESC [` or C1 `CSI`, then parameters, intermediates and a final byte.
    Csi {
        value: &'a str,
        params: &'a str,
        intermediates: &'a str,
        final_char: char,
    },
    /// `ESC`, intermediates and a final byte.
    Esc {
        value: &'a str,
        intermediates: &'a str,
        final_char: char,
    },
    /// OSC, DCS, PM, APC or SOS through its terminator.
    ControlString {
        kind: ControlStringKind,
        value: &'a str,
    },
    /// A string terminator without an opening control string.
    St(&'a str),
    /// A standalone C1 control character.
    C1(&'a str),
    /// An incomplete or malformed sequence. Tokenizing stops here; the value
    /// runs to the end of the input.
    Invalid(&'a str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlStringKind {
    Osc,
    Dcs,
    Pm,
    Apc,
    Sos,
}

pub(crate) fn tokenize(text: &str) -> Vec<Token<'_>> {
    if !has_ansi_control_characters(text) {
        return vec![Token::Text(text)];
    }
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < text.len() {
        let c = text[index..].chars().next().unwrap();
        if c != '\x1b' && !('\u{80}'..='\u{9f}').contains(&c) {
            index += c.len_utf8();
            continue;
        }
        if start < index {
            tokens.push(Token::Text(&text[start..index]));
        }
        let from = index + c.len_utf8();
        let introducer = if c == '\x1b' {
            text[from..].chars().next()
        } else {
            Some(c)
        };
        let body = if c == '\x1b' {
            from + usize::from(introducer.is_some())
        } else {
            from
        };
        let control = introducer.filter(|c| {
            if text.as_bytes()[index] == 0x1b {
                c.is_ascii()
            } else {
                true
            }
        });
        let kind = match control {
            Some(']' | '\u{9d}') => Some(ControlStringKind::Osc),
            Some('P' | '\u{90}') => Some(ControlStringKind::Dcs),
            Some('^' | '\u{9e}') => Some(ControlStringKind::Pm),
            Some('_' | '\u{9f}') => Some(ControlStringKind::Apc),
            Some('X' | '\u{98}') => Some(ControlStringKind::Sos),
            _ => None,
        };
        let parsed = if matches!(control, Some('[' | '\u{9b}')) {
            read_sequence(text, body, true).map(|(end, params, intermediates, final_char)| {
                (
                    end,
                    Token::Csi {
                        value: &text[index..end],
                        params,
                        intermediates,
                        final_char,
                    },
                )
            })
        } else if let Some(kind) = kind {
            control_string_end(text, body, kind == ControlStringKind::Osc).map(|end| {
                (
                    end,
                    Token::ControlString {
                        kind,
                        value: &text[index..end],
                    },
                )
            })
        } else if c == '\x1b' {
            match read_sequence(text, from, false) {
                Some((end, _, intermediates, final_char)) => Some((
                    end,
                    Token::Esc {
                        value: &text[index..end],
                        intermediates,
                        final_char,
                    },
                )),
                None if introducer.is_some_and(|c| !(' '..='/').contains(&c)) => {
                    index = from;
                    start = index;
                    continue;
                }
                None => None,
            }
        } else {
            Some((
                from,
                if c == '\u{9c}' {
                    Token::St(&text[index..from])
                } else {
                    Token::C1(&text[index..from])
                },
            ))
        };
        let Some((end, token)) = parsed else {
            tokens.push(Token::Invalid(&text[index..]));
            return tokens;
        };
        tokens.push(token);
        index = end;
        start = end;
    }
    if start < text.len() {
        tokens.push(Token::Text(&text[start..]));
    }
    tokens
}

fn read_sequence(text: &str, from: usize, csi: bool) -> Option<(usize, &str, &str, char)> {
    let bytes = text.as_bytes();
    let mut index = from;
    if csi {
        while bytes.get(index).is_some_and(|b| (0x30..=0x3f).contains(b)) {
            index += 1;
        }
    }
    let middle = index;
    while bytes.get(index).is_some_and(|b| (0x20..=0x2f).contains(b)) {
        index += 1;
    }
    let final_byte = *bytes.get(index)?;
    if !(if csi { 0x40 } else { 0x30 }..=0x7e).contains(&final_byte) {
        return None;
    }
    Some((
        index + 1,
        &text[from..middle],
        &text[middle..index],
        char::from(final_byte),
    ))
}

fn control_string_end(text: &str, from: usize, bell: bool) -> Option<usize> {
    let mut chars = text[from..].char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        if c == '\u{9c}' || (bell && c == '\x07') {
            return Some(from + index + c.len_utf8());
        }
        if c == '\x1b' {
            match chars.peek() {
                Some((_, '\x1b')) => {
                    chars.next();
                }
                Some((_, '\\')) => return Some(from + index + 2),
                _ => {}
            }
        }
    }
    None
}

/// Whether `text` contains ESC or a C1 control, so tokenizing can be skipped
/// for plain strings.
pub(crate) fn has_ansi_control_characters(text: &str) -> bool {
    text.chars()
        .any(|c| c == '\x1b' || ('\u{80}'..='\u{9f}').contains(&c))
}

#[cfg(test)]
mod tests;
