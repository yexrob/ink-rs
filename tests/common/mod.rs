//! Helpers for the tests ported from Ink.
//!
//! Styled expectations use a flat markup instead of raw escapes. `{…}` sets
//! the complete style of the text that follows and `{/}` returns to plain:
//!
//! ```text
//! {fg=green}╭──╮{/}
//! {fg=red,bg=#ff8800,bold}x{/} plain {fg=196,underline}y{/}
//! ```
//!
//! Colors are the sixteen names below, a 256-color index, or `#rrggbb`.
//! Attributes are `bold`, `dim`, `italic`, `underline`, `inverse`, `hidden`,
//! `strike` and `overline`. `{{` is a literal brace. Each line starts plain;
//! a tag left open at the end of the last line means the output leaves that
//! style active. Other escape sequences stay as raw escapes in the string.

#![allow(dead_code)]

use std::fmt::Write as _;

use ink_rs::{ColorLevel, Element, RenderOptions, render_to_string};

pub struct Cleanup(pub fn());

impl Drop for Cleanup {
    fn drop(&mut self) {
        (self.0)();
    }
}

/// Renders the way Ink's test helper does: 100 columns, chalk level 1
/// (`FORCE_COLOR=true`, sixteen colors).
pub fn render(element: impl Into<Element>) -> String {
    render_cols(element, 100)
}

pub fn render_cols(element: impl Into<Element>, columns: u16) -> String {
    render_to_string(
        element,
        RenderOptions {
            columns,
            color_level: ColorLevel::Basic,
        },
    )
}

/// Renders with truecolor, for tests that set `chalk.level = 3`.
pub fn render_truecolor(element: impl Into<Element>, columns: u16) -> String {
    render_to_string(
        element,
        RenderOptions {
            columns,
            color_level: ColorLevel::TrueColor,
        },
    )
}

pub fn lines(lines: &[&str]) -> String {
    lines.join("\n")
}

/// Expands markup lines into a string with SGR sequences.
pub fn styled(lines: &[&str]) -> String {
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let mut style = Sgr::default();
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c != '{' {
                out.push(c);
                continue;
            }
            if chars.peek() == Some(&'{') {
                chars.next();
                out.push('{');
                continue;
            }
            let tag: String = chars.by_ref().take_while(|&c| c != '}').collect();
            style = parse_tag(&tag);
            out.push_str("\x1b[0m");
            out.push_str(&style.codes());
        }
        if i + 1 < lines.len() && style != Sgr::default() {
            out.push_str("\x1b[0m");
        }
    }
    out
}

/// Asserts that two strings display the same: the same characters with the
/// same styles, and the same style left active at the end. How the SGR
/// sequences are split, ordered or repeated does not matter.
#[track_caller]
pub fn assert_ansi_eq(actual: &str, expected: &str) {
    let a = parse(actual);
    let e = parse(expected);
    if a != e {
        panic!(
            "styled output differs\n--- expected\n{}\n--- actual\n{}\n",
            show(&e),
            show(&a)
        );
    }
}

/// The canonical markup of `s`, one string per line, for failure messages
/// and for writing expectations.
pub fn to_markup(s: &str) -> Vec<String> {
    markup_lines(&parse(s))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Col {
    #[default]
    Default,
    Named(u8),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Sgr {
    fg: Col,
    bg: Col,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    inverse: bool,
    hidden: bool,
    strike: bool,
    overline: bool,
}

const NAMES: [&str; 16] = [
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "gray",
    "brightRed",
    "brightGreen",
    "brightYellow",
    "brightBlue",
    "brightMagenta",
    "brightCyan",
    "brightWhite",
];

impl Sgr {
    fn flags(&mut self) -> [(&'static str, &mut bool); 8] {
        [
            ("bold", &mut self.bold),
            ("dim", &mut self.dim),
            ("italic", &mut self.italic),
            ("underline", &mut self.underline),
            ("inverse", &mut self.inverse),
            ("hidden", &mut self.hidden),
            ("strike", &mut self.strike),
            ("overline", &mut self.overline),
        ]
    }

    fn codes(mut self) -> String {
        let mut codes = Vec::new();
        for (col, base) in [(self.fg, 30), (self.bg, 40)] {
            match col {
                Col::Default => {}
                Col::Named(n) if n < 8 => codes.push(format!("{}", base + n)),
                Col::Named(n) => codes.push(format!("{}", base + 60 + n - 8)),
                Col::Indexed(n) => codes.push(format!("{};5;{n}", base + 8)),
                Col::Rgb(r, g, b) => codes.push(format!("{};2;{r};{g};{b}", base + 8)),
            }
        }
        for (code, (_, on)) in [1, 2, 3, 4, 7, 8, 9, 53].into_iter().zip(self.flags()) {
            if *on {
                codes.push(code.to_string());
            }
        }
        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    fn tag(mut self) -> String {
        if self == Sgr::default() {
            return "{/}".into();
        }
        let mut parts = Vec::new();
        for (key, col) in [("fg", self.fg), ("bg", self.bg)] {
            match col {
                Col::Default => {}
                Col::Named(n) => parts.push(format!("{key}={}", NAMES[n as usize])),
                Col::Indexed(n) => parts.push(format!("{key}={n}")),
                Col::Rgb(r, g, b) => parts.push(format!("{key}=#{r:02x}{g:02x}{b:02x}")),
            }
        }
        for (name, on) in self.flags() {
            if *on {
                parts.push(name.into());
            }
        }
        format!("{{{}}}", parts.join(","))
    }

    fn apply(&mut self, params: &str) {
        let nums: Vec<u32> = if params.is_empty() {
            vec![0]
        } else {
            params
                .split(';')
                .map(|p| if p.is_empty() { 0 } else { p.parse().unwrap() })
                .collect()
        };
        let mut i = 0;
        while i < nums.len() {
            let n = nums[i];
            match n {
                0 => *self = Sgr::default(),
                1 => self.bold = true,
                2 => self.dim = true,
                3 => self.italic = true,
                4 => self.underline = true,
                7 => self.inverse = true,
                8 => self.hidden = true,
                9 => self.strike = true,
                22 => (self.bold, self.dim) = (false, false),
                23 => self.italic = false,
                24 => self.underline = false,
                27 => self.inverse = false,
                28 => self.hidden = false,
                29 => self.strike = false,
                53 => self.overline = true,
                55 => self.overline = false,
                30..=37 => self.fg = Col::Named((n - 30) as u8),
                90..=97 => self.fg = Col::Named((n - 90 + 8) as u8),
                40..=47 => self.bg = Col::Named((n - 40) as u8),
                100..=107 => self.bg = Col::Named((n - 100 + 8) as u8),
                39 => self.fg = Col::Default,
                49 => self.bg = Col::Default,
                38 | 48 => {
                    let col = match nums.get(i + 1) {
                        Some(5) => {
                            i += 2;
                            Col::Indexed(nums[i] as u8)
                        }
                        Some(2) => {
                            i += 4;
                            Col::Rgb(nums[i - 2] as u8, nums[i - 1] as u8, nums[i] as u8)
                        }
                        _ => panic!("malformed extended color in SGR {params:?}"),
                    };
                    if n == 38 {
                        self.fg = col;
                    } else {
                        self.bg = col;
                    }
                }
                _ => panic!("unsupported SGR parameter {n} in {params:?}"),
            }
            i += 1;
        }
    }
}

fn parse_tag(tag: &str) -> Sgr {
    let mut style = Sgr::default();
    if tag == "/" {
        return style;
    }
    for part in tag.split(',') {
        if let Some((key, value)) = part.split_once('=') {
            let col = if let Some(hex) = value.strip_prefix('#') {
                let v = u32::from_str_radix(hex, 16).unwrap();
                Col::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
            } else if let Ok(n) = value.parse::<u8>() {
                Col::Indexed(n)
            } else {
                let n = NAMES.iter().position(|&name| name == value);
                Col::Named(n.unwrap_or_else(|| panic!("unknown color {value:?}")) as u8)
            };
            match key {
                "fg" => style.fg = col,
                "bg" => style.bg = col,
                _ => panic!("unknown markup key {key:?}"),
            }
        } else {
            let flag = style.flags().into_iter().find(|(name, _)| *name == part);
            *flag
                .unwrap_or_else(|| panic!("unknown markup flag {part:?}"))
                .1 = true;
        }
    }
    style
}

#[derive(Debug, PartialEq, Eq)]
enum Item {
    Char(char, Sgr),
    Newline,
    /// A non-SGR escape sequence, compared verbatim.
    Raw(String),
    /// The style left active at the end.
    End(Sgr),
}

fn parse(s: &str) -> Vec<Item> {
    let mut items = Vec::new();
    let mut style = Sgr::default();
    let mut rest = s;
    while let Some(c) = rest.chars().next() {
        if c == '\x1b' {
            let len = escape_len(rest);
            let seq = &rest[..len];
            match seq.strip_prefix("\x1b[").and_then(|p| p.strip_suffix('m')) {
                Some(params) if params.bytes().all(|b| b.is_ascii_digit() || b == b';') => {
                    style.apply(params)
                }
                _ => items.push(Item::Raw(seq.to_owned())),
            }
            rest = &rest[len..];
            continue;
        }
        items.push(if c == '\n' {
            Item::Newline
        } else {
            Item::Char(c, style)
        });
        rest = &rest[c.len_utf8()..];
    }
    items.push(Item::End(style));
    items
}

/// Length of the escape sequence at the start of `s`: CSI through its final
/// byte, OSC/DCS/APC/PM/SOS through BEL or ST, otherwise ESC plus one char.
fn escape_len(s: &str) -> usize {
    let b = s.as_bytes();
    match b.get(1) {
        Some(b'[') => {
            let end = b[2..].iter().position(|c| (0x40..=0x7e).contains(c));
            end.map_or(b.len(), |e| e + 3)
        }
        Some(b']' | b'P' | b'_' | b'^' | b'X') => {
            let mut i = 2;
            while i < b.len() {
                if b[i] == 0x07 {
                    return i + 1;
                }
                if b[i] == 0x1b && b.get(i + 1) == Some(&b'\\') {
                    return i + 2;
                }
                i += 1;
            }
            b.len()
        }
        Some(_) => 1 + s[1..].chars().next().unwrap().len_utf8(),
        None => 1,
    }
}

fn markup_lines(items: &[Item]) -> Vec<String> {
    let mut lines = vec![String::new()];
    let mut current = Sgr::default();
    for item in items {
        let line = lines.last_mut().unwrap();
        match item {
            Item::Char(c, style) => {
                if *style != current {
                    line.push_str(&style.tag());
                    current = *style;
                }
                if *c == '{' {
                    line.push('{');
                }
                line.push(*c);
            }
            Item::Newline => {
                if current != Sgr::default() {
                    line.push_str("{/}");
                    current = Sgr::default();
                }
                lines.push(String::new());
            }
            Item::Raw(seq) => line.push_str(seq),
            Item::End(style) => {
                if *style != current {
                    line.push_str(&style.tag());
                }
            }
        }
    }
    lines
}

fn show(items: &[Item]) -> String {
    let mut out = String::new();
    for (i, line) in markup_lines(items).iter().enumerate() {
        let _ = writeln!(out, "{i:>3} |{}|", line.escape_debug());
    }
    out
}

/// A vt100 screen and scrollback fed from the in-memory app terminal.
pub struct Screen {
    parser: vt100::Parser,
    columns: u16,
    writes: usize,
}

impl Screen {
    pub fn new(terminal: &ink_rs::testing::TestTerminal, columns: u16, rows: u16) -> Self {
        let mut parser = vt100::Parser::new(rows, columns, 1000);
        for (_, write) in terminal.output() {
            parser.process(write.as_bytes());
        }
        Self {
            parser,
            columns,
            writes: terminal.output().len(),
        }
    }

    /// Feeds writes made since the last observation.
    pub fn update(&mut self, terminal: &ink_rs::testing::TestTerminal) {
        let output = terminal.output();
        for (_, write) in &output[self.writes..] {
            self.parser.process(write.as_bytes());
        }
        self.writes = output.len();
    }

    /// Models reconstruct-terminal.ts's xterm shrink: drop rows below the
    /// cursor first, then scroll top rows into history. vt100 set_size alone
    /// truncates the bottom. Growth does not pull rows from scrollback.
    pub fn resize(&mut self, columns: u16, rows: u16) {
        assert!(columns > 0 && rows > 0);
        let old_rows = self.parser.screen().size().0;
        if rows < old_rows {
            let scroll = (self.cursor().0 + 1).saturating_sub(rows);
            if scroll > 0 {
                self.parser.process(format!("\x1b[{scroll}S").as_bytes());
            }
        }
        self.parser.screen_mut().set_size(rows, columns);
        self.columns = columns;
    }

    pub fn rows(&self) -> Vec<String> {
        self.parser.screen().rows(0, self.columns).collect()
    }

    pub fn cursor(&self) -> (u16, u16) {
        self.parser.screen().cursor_position()
    }

    pub fn cursor_visible(&self) -> bool {
        !self.parser.screen().hide_cursor()
    }

    pub fn alternate_screen(&self) -> bool {
        self.parser.screen().alternate_screen()
    }

    pub fn scrollback(&mut self) -> Vec<String> {
        self.parser.screen_mut().set_scrollback(usize::MAX);
        let count = self.parser.screen().scrollback();
        let mut lines = Vec::new();
        for offset in (1..=count).rev() {
            self.parser.screen_mut().set_scrollback(offset);
            lines.push(self.rows()[0].clone());
        }
        self.parser.screen_mut().set_scrollback(0);
        lines
    }

    pub fn lines(&mut self) -> Vec<String> {
        let mut lines = self.scrollback();
        lines.extend(self.rows());
        lines
    }
}

pub fn assert_no_full_clear(terminal: &ink_rs::testing::TestTerminal) {
    let output = terminal.writes().join("");
    for sequence in ["\x1b[2J", "\x1b[3J", "\x1b[1;1H\x1b[J"] {
        assert!(!output.contains(sequence), "{output:?}");
    }
}
