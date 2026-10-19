use super::ansi::{ControlStringKind, Token, has_ansi_control_characters, tokenize};
use super::width::is_mark;

/// Keeps SGR styling and OSC sequences such as hyperlinks, and removes every
/// other control sequence and C0 control except tab and newline: cursor
/// movement or screen clearing would corrupt layout. Colon SGR parameters
/// are rewritten to the semicolon form or dropped.
pub(crate) fn sanitize_ansi(text: &str) -> String {
    let clean = |text: &str| {
        text.chars()
            .filter(|&c| !c.is_control() || matches!(c, '\t' | '\n'))
            .collect::<String>()
    };
    if !has_ansi_control_characters(text) {
        return clean(text);
    }
    let mut output = String::new();
    let mut pending = String::new();
    for token in tokenize(text) {
        match token {
            Token::Text(value) => {
                let value = clean(value);
                let marks = value
                    .char_indices()
                    .find(|&(_, c)| !is_mark(c))
                    .map_or(value.len(), |(i, _)| i);
                output.push_str(&value[..marks]);
                if marks < value.len() {
                    output.push_str(&pending);
                    pending.clear();
                    output.push_str(&value[marks..]);
                }
            }
            Token::ControlString {
                kind: ControlStringKind::Osc,
                value,
            } => {
                output.push_str(&pending);
                pending.clear();
                output.push_str(value);
            }
            Token::Csi {
                params,
                intermediates: "",
                final_char: 'm',
                ..
            } if params
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b';' | b':')) =>
            {
                let params: Vec<_> = params.split(';').filter_map(normalize_parameter).collect();
                if !params.is_empty() {
                    pending.push_str("\x1b[");
                    pending.push_str(&params.join(";"));
                    pending.push('m');
                }
            }
            _ => {}
        }
    }
    output.push_str(&pending);
    output
}

// Ink src/sanitize-ansi.ts drops unsupported subparameters before layout.
fn normalize_parameter(parameter: &str) -> Option<String> {
    let mut parts: Vec<_> = parameter.split(':').collect();
    if parts.len() == 1 {
        return Some(parameter.to_owned());
    }
    if parts[0] == "4" {
        return Some(
            if matches!(parts[1], "0" | "") {
                "24"
            } else {
                "4"
            }
            .to_owned(),
        );
    }
    if !matches!(parts[0], "38" | "48") {
        return None;
    }
    if parts[1] == "5" && parts.len() == 3 {
        return Some(parts.join(";"));
    }
    if parts[1] == "2" {
        if parts.len() == 6 && matches!(parts[2], "" | "0") {
            parts.remove(2);
        }
        if parts.len() == 5 {
            return Some(parts.join(";"));
        }
    }
    None
}

#[cfg(test)]
mod tests;
