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
    let _ = text;
    todo!()
}

/// Whether `text` contains ESC or a C1 control, so tokenizing can be skipped
/// for plain strings.
pub(crate) fn has_ansi_control_characters(text: &str) -> bool {
    let _ = text;
    todo!()
}

#[cfg(test)]
mod tests;
