use unicode_segmentation::UnicodeSegmentation;

use crate::Color;

use super::ansi::{ControlStringKind, Token, tokenize};
use super::sanitize::sanitize_ansi;
use super::width::grapheme_width;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Hyperlink {
    open: String,
    terminator: &'static str,
}

impl Hyperlink {
    fn parse(sequence: &str) -> Option<Option<Self>> {
        let payload = sequence
            .strip_prefix("\x1b]")
            .or_else(|| sequence.strip_prefix('\u{9d}'))?;
        let (payload, terminator) = if let Some(payload) = payload.strip_suffix('\x07') {
            (payload, "\x07")
        } else if let Some(payload) = payload
            .strip_suffix("\x1b\\")
            .or_else(|| payload.strip_suffix('\u{9c}'))
        {
            (payload, "\x1b\\")
        } else {
            return None;
        };
        let (_, uri) = payload.strip_prefix("8;")?.split_once(';')?;
        Some(if uri.is_empty() {
            None
        } else {
            Some(Self {
                open: format!("\x1b]{payload}{terminator}"),
                terminator,
            })
        })
    }

    fn close_into(&self, output: &mut String) {
        output.push_str("\x1b]8;;");
        output.push_str(self.terminator);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CharStyle {
    pub(crate) fg: Option<Color>,
    pub(crate) bg: Option<Color>,
    pub(crate) bold: bool,
    pub(crate) dim: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) inverse: bool,
    pub(crate) hidden: bool,
    pub(crate) strikethrough: bool,
    pub(crate) overline: bool,
    pub(crate) link: Option<Hyperlink>,
}

impl CharStyle {
    pub(crate) fn fill_from(&mut self, outer: &Self) {
        self.fg = self.fg.or(outer.fg);
        self.bg = self.bg.or(outer.bg);
        self.bold |= outer.bold;
        self.dim |= outer.dim;
        self.italic |= outer.italic;
        self.underline |= outer.underline;
        self.inverse |= outer.inverse;
        self.hidden |= outer.hidden;
        self.strikethrough |= outer.strikethrough;
        self.overline |= outer.overline;
        if self.link.is_none() {
            self.link.clone_from(&outer.link);
        }
    }

    fn apply_sgr(&mut self, parameters: &str) {
        let values: Vec<Option<u16>> = parameters
            .split(';')
            .map(|s| {
                if s.is_empty() {
                    Some(0)
                } else {
                    s.parse().ok()
                }
            })
            .collect();
        let mut index = 0;
        while index < values.len() {
            let code = values[index];
            index += 1;
            match code {
                Some(0) => {
                    *self = Self {
                        link: self.link.take(),
                        ..Self::default()
                    };
                }
                Some(1) => self.bold = true,
                Some(2) => self.dim = true,
                Some(3) => self.italic = true,
                Some(4) => self.underline = true,
                Some(7) => self.inverse = true,
                Some(8) => self.hidden = true,
                Some(9) => self.strikethrough = true,
                Some(22) => {
                    self.bold = false;
                    self.dim = false;
                }
                Some(23) => self.italic = false,
                Some(24) => self.underline = false,
                Some(27) => self.inverse = false,
                Some(28) => self.hidden = false,
                Some(29) => self.strikethrough = false,
                Some(53) => self.overline = true,
                Some(55) => self.overline = false,
                Some(30..=37) => self.fg = Some(Color::from_ansi16((code.unwrap() - 30) as u8)),
                Some(90..=97) => self.fg = Some(Color::from_ansi16((code.unwrap() - 90 + 8) as u8)),
                Some(40..=47) => self.bg = Some(Color::from_ansi16((code.unwrap() - 40) as u8)),
                Some(100..=107) => {
                    self.bg = Some(Color::from_ansi16((code.unwrap() - 100 + 8) as u8))
                }
                Some(39) => self.fg = None,
                Some(49) => self.bg = None,
                Some(38 | 48 | 58) => {
                    // Consume unsupported underline colors too, so their RGB
                    // components cannot turn into unrelated SGR attributes.
                    let mode = values.get(index).copied().flatten();
                    let count = match mode {
                        Some(5) => 1,
                        Some(2) => 3,
                        _ => 0,
                    };
                    let component = |offset| {
                        values
                            .get(index + offset)
                            .copied()
                            .flatten()
                            .and_then(|n| u8::try_from(n).ok())
                    };
                    let color = match mode {
                        Some(5) => component(1).map(Color::Ansi256),
                        Some(2) => match (component(1), component(2), component(3)) {
                            (Some(r), Some(g), Some(b)) => Some(Color::Rgb(r, g, b)),
                            _ => None,
                        },
                        _ => None,
                    };
                    index += 1 + count;
                    if let Some(color) = color {
                        match code {
                            Some(38) => self.fg = Some(color),
                            Some(48) => self.bg = Some(color),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StyledChar {
    pub(crate) grapheme: String,
    // string-width permits widths above two for repeated Hangul jamo and
    // trailing spacing marks; do not split a grapheme to enforce a width cap.
    pub(crate) width: usize,
    pub(crate) style: CharStyle,
}

pub(crate) type StyledLine = Vec<StyledChar>;
pub(crate) type StyledText = Vec<StyledLine>;

/// Concatenation joins the boundary lines; explicit line breaks remain intact.
pub(crate) fn append(target: &mut StyledText, source: StyledText) {
    let mut lines = source.into_iter();
    if let Some(first) = lines.next() {
        if let Some(last) = target.last_mut() {
            for cell in first {
                append_cell(last, cell);
            }
        } else {
            target.push(first);
        }
        target.extend(lines);
    }
}

fn append_cell(line: &mut StyledLine, cell: StyledChar) {
    let Some(previous) = line.last_mut() else {
        line.push(cell);
        return;
    };
    // Inline boundaries are not grapheme boundaries: a mark, ZWJ sequence, or
    // flag can continue the preceding span. Its first scalar owns the style.
    let boundary = previous.grapheme.len();
    let joined = format!("{}{}", previous.grapheme, cell.grapheme);
    let mut graphemes = joined.grapheme_indices(true);
    let (_, first) = graphemes.next().unwrap();
    if first.len() == boundary {
        line.push(cell);
        return;
    }
    previous.grapheme = first.to_owned();
    previous.width = grapheme_width(first);
    line.extend(graphemes.map(|(_, grapheme)| StyledChar {
        grapheme: grapheme.to_owned(),
        width: grapheme_width(grapheme),
        style: cell.style.clone(),
    }));
}

/// External strings enter here, including Transform results. Tabs remain cells
/// until the assembled inline line has its final tab-stop origin.
pub(crate) fn parse(text: &str) -> StyledText {
    let sanitized = sanitize_ansi(text);
    let mut plain = String::new();
    let mut runs = Vec::new();
    let mut style = CharStyle::default();
    for token in tokenize(&sanitized) {
        match token {
            Token::Text(value) if !value.is_empty() => {
                runs.push((plain.len(), style.clone()));
                plain.push_str(value);
            }
            Token::Csi {
                params,
                final_char: 'm',
                ..
            } => style.apply_sgr(params),
            Token::ControlString {
                kind: ControlStringKind::Osc,
                value,
            } => {
                if let Some(link) = Hyperlink::parse(value) {
                    style.link = link;
                }
            }
            _ => {}
        }
    }
    if plain.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![Vec::new()];
    let mut run = 0;
    // Segment after removing sequences: ANSI inside a cluster must not split
    // it, and its first code point owns the style (docs/design.md).
    for (index, grapheme) in plain.grapheme_indices(true) {
        while run + 1 < runs.len() && runs[run + 1].0 <= index {
            run += 1;
        }
        if grapheme == "\n" {
            lines.push(Vec::new());
        } else {
            lines.last_mut().unwrap().push(StyledChar {
                grapheme: grapheme.to_owned(),
                width: grapheme_width(grapheme),
                style: runs[run].1.clone(),
            });
        }
    }
    lines
}

pub(crate) fn expand_tabs(line: &[StyledChar]) -> StyledLine {
    let mut column = 0;
    let mut expanded = Vec::new();
    for cell in line {
        if cell.grapheme == "\t" {
            let spaces = 8 - column % 8;
            expanded.extend((0..spaces).map(|_| StyledChar {
                grapheme: " ".to_owned(),
                width: 1,
                style: cell.style.clone(),
            }));
            column += spaces;
        } else {
            expanded.push(cell.clone());
            column += cell.width;
        }
    }
    expanded
}

fn sgr(output: &mut String, code: &str) {
    output.push_str("\x1b[");
    output.push_str(code);
    output.push('m');
}

fn transition(output: &mut String, from: &CharStyle, to: &CharStyle) {
    // Keep SGR outside OSC 8 even when the link survives an attribute change.
    let style_changed = from != to;
    if style_changed && let Some(link) = &from.link {
        link.close_into(output);
    }
    for (old, new, background) in [(from.fg, to.fg, false), (from.bg, to.bg, true)] {
        if old != new {
            if let Some(color) = new {
                sgr(output, &color.sgr(background));
            } else {
                sgr(output, if background { "49" } else { "39" });
            }
        }
    }
    let reset_intensity = from.bold && !to.bold || from.dim && !to.dim;
    if reset_intensity {
        sgr(output, "22");
    }
    if to.bold && (!from.bold || reset_intensity) {
        sgr(output, "1");
    }
    if to.dim && (!from.dim || reset_intensity) {
        sgr(output, "2");
    }
    for (old, new, open, close) in [
        (from.italic, to.italic, "3", "23"),
        (from.underline, to.underline, "4", "24"),
        (from.inverse, to.inverse, "7", "27"),
        (from.hidden, to.hidden, "8", "28"),
        (from.strikethrough, to.strikethrough, "9", "29"),
        (from.overline, to.overline, "53", "55"),
    ] {
        if old != new {
            sgr(output, if new { open } else { close });
        }
    }
    if style_changed && let Some(link) = &to.link {
        output.push_str(&link.open);
    }
}

/// Canonical, self-contained encoding of one line; no terminal state leaks.
pub(crate) fn serialize(line: &[StyledChar]) -> String {
    let mut output = String::new();
    let default = CharStyle::default();
    let mut previous = &default;
    for cell in line {
        if previous != &cell.style {
            transition(&mut output, previous, &cell.style);
        }
        output.push_str(&cell.grapheme);
        previous = &cell.style;
    }
    transition(&mut output, previous, &default);
    output
}
