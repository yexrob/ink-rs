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
    /// The terminal's default foreground or background, without adding SGR.
    /// On a Text's `background_color`, stops the enclosing View's background,
    /// like Ink's `backgroundColor=""`; an outer Text's styling applies.
    Reset,
}

impl Color {
    /// Parses `#rgb` or `#rrggbb`. Panics on malformed input, so it can build
    /// constants; use [`str::parse`] for untrusted strings.
    pub const fn hex(hex: &str) -> Color {
        match parse_hex(hex) {
            Some(color) => color,
            None => panic!("invalid hex color"),
        }
    }

    /// Prop colors follow chalk; embedded SGR bypasses this conversion.
    pub(crate) fn at_level(self, level: ColorLevel) -> Option<Self> {
        if self == Self::Reset || level == ColorLevel::None {
            return None;
        }
        let color = match (self, level) {
            (Self::Rgb(r, g, b), ColorLevel::Basic | ColorLevel::Ansi256) => {
                Self::Ansi256(rgb_to_ansi256(r, g, b))
            }
            _ => self,
        };
        Some(match (color, level) {
            (Self::Ansi256(n), ColorLevel::Basic) => ansi256_to_basic(n),
            _ => color,
        })
    }

    pub(crate) fn sgr(self, background: bool) -> String {
        let offset = if background { 10 } else { 0 };
        match self {
            Self::Reset => (39 + offset).to_string(),
            Self::Ansi256(n) => format!("{};5;{n}", 38 + offset),
            Self::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", 38 + offset),
            _ => {
                let index = NAMED.iter().position(|&color| color == self).unwrap();
                let code = if index < 8 { 30 + index } else { 82 + index };
                (code + offset).to_string()
            }
        }
    }

    pub(crate) fn from_ansi16(index: u8) -> Self {
        NAMED[usize::from(index)]
    }
}

const NAMED: [Color; 16] = [
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::White,
    Color::Gray,
    Color::BrightRed,
    Color::BrightGreen,
    Color::BrightYellow,
    Color::BrightBlue,
    Color::BrightMagenta,
    Color::BrightCyan,
    Color::BrightWhite,
];

const fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

const fn parse_hex(hex: &str) -> Option<Color> {
    let bytes = hex.as_bytes();
    if (bytes.len() != 4 && bytes.len() != 7) || bytes[0] != b'#' {
        return None;
    }
    let mut rgb = [0; 3];
    let mut i = 0;
    while i < 3 {
        let index = if bytes.len() == 4 { i + 1 } else { i * 2 + 1 };
        let Some(high) = hex_digit(bytes[index]) else {
            return None;
        };
        let low = if bytes.len() == 4 {
            high
        } else {
            let Some(low) = hex_digit(bytes[index + 1]) else {
                return None;
            };
            low
        };
        rgb[i] = high * 16 + low;
        i += 1;
    }
    Some(Color::Rgb(rgb[0], rgb[1], rgb[2]))
}

// ansi-styles/index.js uses a grayscale ramp before the 6x6x6 cube.
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        return match r {
            0..=7 => 16,
            249..=255 => 231,
            _ => (((f64::from(r) - 8.0) / 247.0 * 24.0).round() as u8) + 232,
        };
    }
    let channel = |n| (f64::from(n) / 255.0 * 5.0).round() as u8;
    16 + 36 * channel(r) + 6 * channel(g) + channel(b)
}

fn ansi256_to_basic(n: u8) -> Color {
    if n < 16 {
        return Color::from_ansi16(n);
    }
    let (r, g, b) = if n >= 232 {
        let gray = (f64::from(n - 232) * 10.0 + 8.0) / 255.0;
        (gray, gray, gray)
    } else {
        let n = n - 16;
        (
            f64::from(n / 36) / 5.0,
            f64::from(n % 36 / 6) / 5.0,
            f64::from(n % 6) / 5.0,
        )
    };
    let index = (r.round() as u8) | ((g.round() as u8) << 1) | ((b.round() as u8) << 2);
    Color::from_ansi16(index + if r.max(g).max(b) == 1.0 { 8 } else { 0 })
}

// Ink src/colorize.ts uses ECMAScript \s (BOM included, NEL excluded).
fn color_whitespace(c: char) -> bool {
    matches!(c, '\t'..='\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}'
        | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

/// Parses Ink's color strings: chalk names (`"red"`, `"redBright"`,
/// `"gray"`), `"#ff8800"`, `"rgb(255, 136, 0)"` and `"ansi256(208)"`.
impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let names = [
            "black",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "white",
            "blackBright",
            "redBright",
            "greenBright",
            "yellowBright",
            "blueBright",
            "magentaBright",
            "cyanBright",
            "whiteBright",
        ];
        if let Some(index) = names.iter().position(|&name| name == s) {
            return Ok(NAMED[index]);
        }
        if matches!(s, "gray" | "grey") {
            return Ok(Self::Gray);
        }
        let error = || ParseColorError(s.to_owned());
        if s.starts_with('#') {
            return parse_hex(s).ok_or_else(error);
        }
        let component = |value: &str| {
            let value = value.trim_matches(color_whitespace);
            if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err(error());
            }
            value.parse::<u8>().map_err(|_| error())
        };
        if let Some(body) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<_> = body.split(',').collect();
            if let [r, g, b] = parts.as_slice() {
                return Ok(Self::Rgb(component(r)?, component(g)?, component(b)?));
            }
        }
        if let Some(body) = s.strip_prefix("ansi256(").and_then(|s| s.strip_suffix(')')) {
            let trimmed = body.trim_matches(color_whitespace);
            let leading = body.chars().take_while(|&c| color_whitespace(c)).count();
            let trailing = body
                .chars()
                .rev()
                .take_while(|&c| color_whitespace(c))
                .count();
            if leading <= 1 && trailing <= 1 {
                return Ok(Self::Ansi256(component(trimmed)?));
            }
        }
        Err(error())
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
