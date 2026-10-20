use std::ops::BitOr;

/// Opt-in kitty keyboard protocol negotiation, matching Ink's `kittyKeyboard`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KittyKeyboard {
    /// Do not query or enable the protocol (the default).
    #[default]
    Disabled,
    /// Query when interactive and stdin and stdout are TTYs. Enable with these
    /// flags if a response arrives through raw-mode input within 200 ms.
    Auto(KittyFlags),
    /// Enable with these flags when stdin and stdout are TTYs, regardless of
    /// interactive mode or whether an input hook holds raw mode.
    Enabled(KittyFlags),
}

/// Kitty protocol features, combined with `|`. The default disambiguates
/// escape codes, matching Ink's default flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KittyFlags(u8);

impl KittyFlags {
    /// Disambiguate escape codes (bit 1).
    pub const DISAMBIGUATE_ESCAPE_CODES: Self = Self(1);
    /// Report press, repeat and release events (bit 2).
    pub const REPORT_EVENT_TYPES: Self = Self(2);
    /// Report alternate key encodings (bit 4).
    pub const REPORT_ALTERNATE_KEYS: Self = Self(4);
    /// Report all keys as escape codes (bit 8).
    pub const REPORT_ALL_KEYS_AS_ESCAPE_CODES: Self = Self(8);
    /// Report associated text (bits 16 | 8, value 24). The protocol defines
    /// associated text only with all-key reporting, so this includes bit 8,
    /// matching Ink's `resolveFlags` in src/kitty-keyboard.ts.
    pub const REPORT_ASSOCIATED_TEXT: Self = Self(16 | 8);

    pub(crate) fn bits(self) -> u8 {
        self.0
    }
}

impl Default for KittyFlags {
    /// Returns [`Self::DISAMBIGUATE_ESCAPE_CODES`].
    fn default() -> Self {
        Self::DISAMBIGUATE_ESCAPE_CODES
    }
}

impl BitOr for KittyFlags {
    type Output = Self;

    /// Combines the requested protocol features.
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

pub(crate) fn is_query_response(text: &str) -> bool {
    // Ink App.handleReadable: /^\x1b\[\?\d+u$/, on text events, not paste.
    text.strip_prefix("\x1b[?")
        .and_then(|body| body.strip_suffix('u'))
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}
