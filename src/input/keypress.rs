//! String overload of Ink src/parse-keypress.ts @ 26d2c3f.

mod names;

use super::KeyEventType;
use names::{codepoint_name, legacy_name, special_letter_name, special_number_name};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ParsedKey {
    pub(crate) name: String,
    pub(crate) ctrl: bool,
    pub(crate) meta: bool,
    pub(crate) shift: bool,
    pub(crate) sequence: String,
    pub(crate) super_key: bool,
    pub(crate) hyper: bool,
    pub(crate) caps_lock: bool,
    pub(crate) num_lock: bool,
    pub(crate) event_type: Option<KeyEventType>,
    pub(crate) is_kitty_protocol: bool,
    pub(crate) text: Option<String>,
    pub(crate) is_printable: bool,
}

pub(crate) fn parse_keypress(input: &str) -> ParsedKey {
    if let Some(key) = kitty_key(input) {
        return key;
    }
    if let Some(key) = kitty_special_key(input) {
        return key;
    }
    let mut key = ParsedKey {
        sequence: input.into(),
        ..ParsedKey::default()
    };
    let bare = input.strip_prefix('\x1b').unwrap_or(input);
    let single = single_char(bare);
    match input {
        "\r" | "\x1b\r" => {
            key.name = "return".into();
            key.meta = input.len() == 2;
        }
        "\x1bOM" | "\x1b\x1bOM" => {
            // Keypad Enter must produce the same input as Return
            // (src/parse-keypress.ts:490-495).
            key.name = "return".into();
            key.sequence = "\r".into();
            key.meta = input.len() == 4;
        }
        "\n" => key.name = "enter".into(),
        "\t" | "\x1b\t" => {
            key.name = "tab".into();
            key.meta = input.len() == 2;
        }
        "\x08" | "\x1b\x08" | "\x7f" | "\x1b\x7f" => {
            key.name = "backspace".into();
            key.meta = input.starts_with('\x1b');
        }
        "\x1b" | "\x1b\x1b" => {
            key.name = "escape".into();
            key.meta = input.len() == 2;
        }
        // Replaces /^\x1b?[ \x00]$/ (src/parse-keypress.ts:515).
        _ if matches!(single, Some(' ' | '\0')) => {
            key.name = "space".into();
            key.ctrl = bare == "\0";
            key.meta = input.starts_with('\x1b');
        }
        // Replaces /^\x1b?[\x00-\x1f]$/ (src/parse-keypress.ts:519).
        _ if single.is_some_and(|c| c <= '\x1f') => {
            let cp = u32::from(single.unwrap());
            key.name = char::from_u32(cp + if cp <= 26 { 96 } else { 64 })
                .unwrap()
                .into();
            key.ctrl = true;
            key.meta = input.starts_with('\x1b');
        }
        _ if input.len() == 1 && input.as_bytes()[0].is_ascii_digit() => {
            key.name = "number".into();
        }
        _ if input.len() == 1 && input.as_bytes()[0].is_ascii_alphabetic() => {
            key.name = input.to_ascii_lowercase();
            key.shift = input.as_bytes()[0].is_ascii_uppercase();
        }
        // Replaces /^(?:\x1b)([^\p{C}\[])$/u (line 6), excluding only Cc: a
        // keyboard does not send format, private-use or unassigned characters
        // after ESC, and Cn follows the JavaScript engine's Unicode version.
        _ if input.starts_with('\x1b') && single.is_some_and(|c| c != '[' && !c.is_control()) => {
            key.name = bare.to_lowercase();
            key.meta = true;
            // Replaces /^[A-Z]$/ (src/parse-keypress.ts:539).
            key.shift = single.unwrap().is_ascii_uppercase();
        }
        _ => {
            if let Some((code, modifier)) = legacy_function_key(input) {
                key.ctrl = modifier & 4 != 0
                    || matches!(
                        code.as_str(),
                        "Oa" | "Ob"
                            | "Oc"
                            | "Od"
                            | "Oe"
                            | "[2^"
                            | "[3^"
                            | "[5^"
                            | "[6^"
                            | "[7^"
                            | "[8^"
                    );
                key.meta = input.starts_with("\x1b\x1b") || modifier & 10 != 0;
                key.shift = modifier & 1 != 0
                    || matches!(
                        code.as_str(),
                        "[a" | "[b"
                            | "[c"
                            | "[d"
                            | "[e"
                            | "[2$"
                            | "[3$"
                            | "[5$"
                            | "[6$"
                            | "[7$"
                            | "[8$"
                            | "[Z"
                    );
                key.name = legacy_name(&code).unwrap_or("").into();
            }
        }
    }
    key
}

fn single_char(input: &str) -> Option<char> {
    let mut chars = input.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(first)
}

fn digits(input: &str) -> bool {
    !input.is_empty() && input.bytes().all(|b| b.is_ascii_digit())
}

fn take_digits(input: &str) -> (&str, &str) {
    let count = input.bytes().take_while(u8::is_ascii_digit).count();
    input.split_at(count)
}

fn number(input: &str) -> f64 {
    input.parse().unwrap_or(f64::NAN)
}

fn modifier_bits(input: &str, clamp: bool) -> u32 {
    // JS subtracts in Number space, then bitwise operators use ToInt32.
    // Keep overflow and zero modifiers from panicking on untrusted input
    // (src/parse-keypress.ts:334, 554).
    let value = number(input) - 1.0;
    let value = if clamp { value.max(0.0) } else { value };
    if value.is_finite() {
        value.rem_euclid(4294967296.0) as u32
    } else {
        0
    }
}

fn kitty_modifiers(key: &mut ParsedKey, modifiers: &str, event: &str) {
    let bits = if modifiers.is_empty() {
        0
    } else {
        modifier_bits(modifiers, true)
    };
    key.ctrl = bits & 4 != 0;
    key.shift = bits & 1 != 0;
    key.meta = bits & (2 | 32) != 0;
    key.super_key = bits & 8 != 0;
    key.hyper = bits & 16 != 0;
    key.caps_lock = bits & 64 != 0;
    key.num_lock = bits & 128 != 0;
    key.event_type = Some(match number(event) {
        3.0 => KeyEventType::Release,
        2.0 => KeyEventType::Repeat,
        _ => KeyEventType::Press,
    });
}

fn modifier_fields(input: &str, empty_allowed: bool) -> Option<(&str, &str)> {
    let (modifiers, event) = input.split_once(':').unwrap_or((input, ""));
    if !(digits(modifiers) || empty_allowed && modifiers.is_empty())
        || (input.contains(':') && !digits(event))
    {
        return None;
    }
    Some((modifiers, event))
}

fn kitty_key(input: &str) -> Option<ParsedKey> {
    // Replaces /^\x1b\[(\d+)(?::\d*(?::\d+)?)?(?:;(\d*)(?::(\d+))?(?:;([\d:]+))?)?u$/
    // (src/parse-keypress.ts:169). Validate the grammar before decoding so a
    // rejected primary code point keeps Ink's empty-kitty-key rule (:459-474).
    let body = input.strip_prefix("\x1b[")?.strip_suffix('u')?;
    let mut fields = body.split(';');
    let primary = fields.next()?;
    let mut codes = primary.split(':');
    let code = codes.next()?;
    if !digits(code) {
        return None;
    }
    if let Some(shifted) = codes.next() {
        if !shifted.is_empty() && !digits(shifted) {
            return None;
        }
        if let Some(base) = codes.next()
            && !digits(base)
        {
            return None;
        }
    }
    if codes.next().is_some() {
        return None;
    }
    let (modifiers, event) = match fields.next() {
        Some(value) => modifier_fields(value, true)?,
        None => ("", ""),
    };
    let text_field = fields.next();
    if fields.next().is_some()
        || text_field
            .is_some_and(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit() || b == b':'))
    {
        return None;
    }
    let mut key = ParsedKey {
        sequence: input.into(),
        is_kitty_protocol: true,
        ..ParsedKey::default()
    };
    let cp = number(code);
    let cp = if cp == 57414.0 { 13.0 } else { cp };
    if !(0.0..=0x10ffff as f64).contains(&cp) {
        return Some(key);
    }
    let Some(character) = char::from_u32(cp as u32) else {
        return Some(key);
    };
    let cp = u32::from(character);
    kitty_modifiers(&mut key, modifiers, event);
    key.text = text_field.map(|text| {
        text.split(':')
            .map(|part| {
                let cp = number(part);
                if (0.0..=0x10ffff as f64).contains(&cp) {
                    char::from_u32(cp as u32).unwrap_or('?')
                } else {
                    '?'
                }
            })
            .collect()
    });
    let (name, printable) = if cp == 32 {
        ("space".into(), true)
    } else if cp == 13 {
        ("return".into(), true)
    } else if let Some(name) = codepoint_name(cp) {
        (name.into(), false)
    } else if cp < 32 || (127..=159).contains(&cp) || (57344..=63743).contains(&cp) {
        (String::new(), false)
    } else {
        (character.to_lowercase().collect(), true)
    };
    key.name = name;
    if printable && key.text.as_ref().is_none_or(String::is_empty) {
        key.text = Some(character.into());
    }
    key.is_printable = printable || key.text.is_some();
    Some(key)
}

fn kitty_special_key(input: &str) -> Option<ParsedKey> {
    // Replaces /^\x1b\[(\d+)(?:;(\d+)(?::(\d+))?)?([A-Za-z~])$/
    // (src/parse-keypress.ts:174). CSI R stays on the legacy F3 path because
    // Ink's kitty letter table omits it.
    let body = input.strip_prefix("\x1b[")?;
    let terminator = body.chars().next_back()?;
    if !terminator.is_ascii_alphabetic() && terminator != '~' {
        return None;
    }
    let body = &body[..body.len() - 1];
    let (code, fields) = body.split_once(';').unwrap_or((body, ""));
    if !digits(code) {
        return None;
    }
    let (modifiers, event) = if body.contains(';') {
        modifier_fields(fields, false)?
    } else {
        ("", "")
    };
    let cp = number(code);
    let name = if terminator == '~' {
        if cp > u32::MAX as f64 {
            return None;
        }
        special_number_name(cp as u32)?
    } else {
        if cp != 1.0 {
            return None;
        }
        special_letter_name(&terminator.to_string())?
    };
    let mut key = ParsedKey {
        sequence: input.into(),
        name: name.into(),
        is_kitty_protocol: true,
        ..ParsedKey::default()
    };
    kitty_modifiers(&mut key, modifiers, event);
    Some(key)
}

fn legacy_function_key(input: &str) -> Option<(String, u32)> {
    // Replaces /^(?:\x1b+)(O|N|\[|\[\[)(?:(\d+)(?:;(\d+))?([~^$])|(?:1;)?(\d+)?([a-zA-Z]))/
    // (src/parse-keypress.ts:8-9). There is no end anchor: pasted strings
    // can match a key prefix. The '[' alternative backtracks to '[['.
    let body = input.strip_prefix('\x1b')?.trim_start_matches('\x1b');
    let (prefix, body) = if let Some(body) = body.strip_prefix("[[") {
        ("[[", body)
    } else if let Some(body) = body.strip_prefix('[') {
        ("[", body)
    } else if let Some(body) = body.strip_prefix('O') {
        ("O", body)
    } else if let Some(body) = body.strip_prefix('N') {
        ("N", body)
    } else {
        return None;
    };
    let (code, rest) = take_digits(body);
    if !code.is_empty() {
        let (modifier, final_part) = if let Some(after) = rest.strip_prefix(';') {
            take_digits(after)
        } else {
            ("1", rest)
        };
        if !modifier.is_empty() && final_part.starts_with(['~', '^', '$']) {
            return Some((
                format!("{prefix}{code}{}", &final_part[..1]),
                modifier_bits(modifier, false),
            ));
        }
    }
    let body = body.strip_prefix("1;").unwrap_or(body);
    let (modifier, rest) = take_digits(body);
    if rest.as_bytes().first().is_some_and(u8::is_ascii_alphabetic) {
        Some((
            format!("{prefix}{}", &rest[..1]),
            modifier_bits(if modifier.is_empty() { "1" } else { modifier }, false),
        ))
    } else {
        None
    }
}

pub(crate) fn is_non_alphanumeric_key(name: &str) -> bool {
    matches!(
        name,
        "up" | "down"
            | "left"
            | "right"
            | "clear"
            | "end"
            | "home"
            | "insert"
            | "delete"
            | "pageup"
            | "pagedown"
            | "tab"
            | "backspace"
            | "f1"
            | "f2"
            | "f3"
            | "f4"
            | "f5"
            | "f6"
            | "f7"
            | "f8"
            | "f9"
            | "f10"
            | "f11"
            | "f12"
    )
}

#[cfg(test)]
mod tests;
