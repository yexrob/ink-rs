use super::Output;
use crate::paint::{Cell, Frame};
use crate::text::styled;

mod log_update;
mod log_update_blank_growth;
mod log_update_columns;
mod log_update_sparse;

fn frame(lines: &[&str]) -> Frame {
    Frame {
        live: lines
            .iter()
            .map(|line| {
                styled::parse(line)
                    .into_iter()
                    .flatten()
                    .flat_map(|glyph| {
                        let continuation = Cell::Continuation(glyph.style.clone());
                        let width = glyph.width;
                        std::iter::once(Cell::Glyph(glyph))
                            .chain(std::iter::repeat_n(continuation, width.saturating_sub(1)))
                    })
                    .collect()
            })
            .collect(),
        static_lines: Vec::new(),
    }
}

struct Terminal {
    output: Output,
    parser: vt100::Parser,
    columns: usize,
    height: usize,
}

impl Terminal {
    fn new(columns: usize, rows: usize) -> Self {
        Self {
            // Run the same ports with synchronized output enabled: its private
            // mode pair must not change screen semantics or no-write behavior.
            output: Output::new(true),
            parser: vt100::Parser::new(rows as u16, columns as u16, 100),
            columns,
            height: rows,
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    fn render(&mut self, lines: &[&str], cursor: Option<(usize, usize)>) -> String {
        let mut bytes = Vec::new();
        self.output
            .render(&frame(lines), cursor, self.columns, self.height, &mut bytes);
        self.feed(&bytes);
        String::from_utf8(bytes).unwrap()
    }

    fn clear(&mut self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.output.clear(&mut bytes);
        self.feed(&bytes);
        bytes
    }

    fn done(&mut self) {
        let mut bytes = Vec::new();
        self.output.done(&mut bytes);
        self.feed(&bytes);
    }

    fn rows(&self) -> Vec<String> {
        self.parser.screen().rows(0, self.columns as u16).collect()
    }

    fn scrollback(&mut self) -> Vec<String> {
        self.parser.screen_mut().set_scrollback(usize::MAX);
        let count = self.parser.screen().scrollback();
        let mut rows = Vec::new();
        for offset in (1..=count).rev() {
            self.parser.screen_mut().set_scrollback(offset);
            rows.push(self.rows()[0].clone());
        }
        self.parser.screen_mut().set_scrollback(0);
        rows
    }

    fn assert_rows(&self, expected: &[&str]) {
        let rows = self.rows();
        assert_eq!(&rows[..expected.len()], expected);
        assert!(
            rows[expected.len()..].iter().all(String::is_empty),
            "{rows:?}"
        );
    }

    fn assert_cursor(&self, x: u16, y: u16, visible: bool) {
        assert_eq!(self.parser.screen().cursor_position(), (y, x));
        assert_eq!(!self.parser.screen().hide_cursor(), visible);
    }
}
