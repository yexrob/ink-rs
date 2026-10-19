use std::fmt;
use std::str::FromStr;

/// A terminal color.
///
/// The sixteen named colors follow chalk's names: `Gray` is chalk's
/// `blackBright`, and the `Bright*` variants are chalk's `*Bright`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    /// Parses `#rgb` or `#rrggbb`. Panics on malformed input, so it can build
    /// constants; use [`str::parse`] for untrusted strings.
    pub const fn hex(hex: &str) -> Color {
        let _ = hex;
        todo!()
    }
}

/// Parses Ink's color strings: chalk names (`"red"`, `"redBright"`,
/// `"gray"`), `"#ff8800"`, `"rgb(255, 136, 0)"` and `"ansi256(208)"`.
impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = s;
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseColorError(pub(crate) String);

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid color: {:?}", self.0)
    }
}

impl std::error::Error for ParseColorError {}

/// How many colors the output may use. Colors above the level are
/// downgraded the way chalk does it: truecolor to 256, 256 to 16.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorLevel {
    /// No styling at all.
    None,
    /// The sixteen named colors.
    Basic,
    Ansi256,
    #[default]
    TrueColor,
}
