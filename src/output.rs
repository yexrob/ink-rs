use crate::paint::{Cell, Frame};
use crate::text::styled::{self, CharStyle};

/// Inline primary-screen output. The caller supplies new Static rows, not history.
/// Dimensions must be positive; terminal resize/reflow belongs to the driver.
pub(crate) struct Output {
    live: Vec<Vec<Cell>>,
    // Coordinates are relative to the live region, even when CRLF scrolls it.
    // x == columns represents the terminal's pending-wrap position.
    cursor: (usize, usize),
    // Last row reached, including a short frame's blank rest row. Moving
    // beyond it requires CRLF rather than cursor motion (bottom-margin scroll).
    reached: usize,
    rest: (usize, usize),
    declared: Option<(usize, usize)>,
    shown: bool,
    full: bool,
    synchronized: bool,
}

impl Output {
    pub(crate) fn new(synchronized: bool) -> Self {
        Self {
            live: Vec::new(),
            cursor: (0, 0),
            reached: 0,
            rest: (0, 0),
            declared: None,
            shown: true,
            full: false,
            synchronized,
        }
    }

    pub(crate) fn render(
        &mut self,
        frame: &Frame,
        cursor: Option<(usize, usize)>,
        columns: usize,
        rows: usize,
        output: &mut Vec<u8>,
    ) {
        assert!(columns > 0 && rows > 0, "viewport must be positive");
        // Unlike Ink's overflow redraw (src/ink.tsx:764-786), keep only rows
        // reachable by relative moves. Cut rows must never enter scrollback.
        let cut = frame.live.len().saturating_sub(rows);
        let mut live: Vec<_> = frame.live[cut..]
            .iter()
            .map(|row| content(row, columns))
            .collect();
        if live.is_empty() {
            // Ink src/output.ts:399 and src/ink.tsx:727: empty output + '\n'.
            live.push(Vec::new());
        }
        let full = live.len() == rows;
        // Ink src/cursor-helpers.ts:132 clamps a position past the output;
        // the height cap also maps a cursor in cut rows to the first live row.
        let declared = cursor.map(|(x, y)| {
            (
                x.min(columns - 1),
                y.saturating_sub(cut).min(live.len() - 1),
            )
        });
        if frame.static_lines.is_empty()
            && self.live == live
            && self.declared == declared
            && self.full == full
        {
            return;
        }

        let mut bytes = Vec::new();
        // Ink src/ink.tsx:451-453 creates its log with showCursor: false;
        // src/cursor-helpers.ts:107-118 hides before returning to the bottom.
        self.hide(&mut bytes);
        if !frame.static_lines.is_empty() {
            // Ink src/ink.tsx:802-815 clears live output before new Static rows.
            self.move_to((0, 0), &mut bytes);
            bytes.extend_from_slice(b"\x1b[J");
            for row in &frame.static_lines {
                write_cells(&content(row, columns), &mut bytes);
                bytes.extend_from_slice(b"\r\n");
            }
            self.forget();
        }

        for (y, row) in live.iter().enumerate() {
            let old = self.live.get(y).map_or(&[][..], Vec::as_slice);
            let Some(mut start) = (0..old.len().max(row.len())).find(|&x| old.get(x) != row.get(x))
            else {
                continue;
            };
            let mut end = (start..old.len().max(row.len()))
                .rfind(|&x| old.get(x) != row.get(x))
                .unwrap()
                + 1;
            // Both grids own glyph boundaries: replacing half an old wide glyph
            // must also repaint the cells of the new glyph covering that half.
            while is_continuation(old, start) || is_continuation(row, start) {
                start -= 1;
            }
            while is_continuation(old, end) || is_continuation(row, end) {
                end += 1;
            }
            let shortened = row.len() < old.len();
            self.move_to((start, y), &mut bytes);
            let written_end = end.min(row.len());
            if start < written_end {
                write_cells(&row[start..written_end], &mut bytes);
                self.cursor.0 = written_end;
            }
            if shortened {
                // Ink test/log-update-sparse.ts:62 (LS5): EL in pending wrap
                // erases xterm's last cell. A shortened row ends before it.
                self.move_to((row.len(), y), &mut bytes);
                bytes.extend_from_slice(b"\x1b[K");
            }
        }
        // Ink src/log-update.ts:278-287 erases dropped rows without relocating
        // the frame. Erase before parking on the new rest row.
        for y in live.len()..self.live.len() {
            self.move_to((0, y), &mut bytes);
            bytes.extend_from_slice(b"\x1b[K");
        }

        // Ink src/ink.tsx:723-727: no trailing newline at viewport height.
        // Normalize the column so returning from a declared cursor never needs
        // to reconstruct pending wrap with an out-of-range CHA.
        let rest_y = if full { live.len() - 1 } else { live.len() };
        self.move_to((0, rest_y), &mut bytes);
        self.rest = self.cursor;
        if let Some(position) = declared {
            self.move_to(position, &mut bytes);
            bytes.extend_from_slice(b"\x1b[?25h");
            self.shown = true;
        }
        self.live = live;
        self.declared = declared;
        self.full = full;
        self.reached = rest_y;
        self.append(bytes, output);
    }

    pub(crate) fn clear(&mut self, output: &mut Vec<u8>) {
        if self.live.is_empty() {
            return;
        }
        let mut bytes = Vec::new();
        self.hide(&mut bytes);
        // Ink src/cursor-helpers.ts:129-143 finds the top from the real cursor.
        for y in 0..self.live.len() {
            self.move_to((0, y), &mut bytes);
            bytes.extend_from_slice(b"\x1b[K");
        }
        self.move_to((0, 0), &mut bytes);
        self.forget();
        self.append(bytes, output);
    }

    pub(crate) fn done(&mut self, output: &mut Vec<u8>) {
        let mut bytes = Vec::new();
        if !self.live.is_empty() {
            // Ink src/log-update.ts:342-362 returns to the bottom before reset.
            self.move_to(self.rest, &mut bytes);
            if self.full {
                // design.md: unlike Ink, make room for the shell after a full frame.
                bytes.extend_from_slice(b"\r\n");
            }
        }
        if !self.shown {
            bytes.extend_from_slice(b"\x1b[?25h");
            self.shown = true;
        }
        self.forget();
        self.append(bytes, output);
    }

    fn hide(&mut self, bytes: &mut Vec<u8>) {
        if self.shown {
            bytes.extend_from_slice(b"\x1b[?25l");
            self.shown = false;
        }
    }

    fn forget(&mut self) {
        self.live.clear();
        self.cursor = (0, 0);
        self.rest = (0, 0);
        self.reached = 0;
        self.declared = None;
        self.full = false;
    }

    fn move_to(&mut self, (x, y): (usize, usize), bytes: &mut Vec<u8>) {
        let reachable = y.min(self.reached);
        if self.cursor.1 > reachable {
            bytes.extend_from_slice(format!("\x1b[{}A", self.cursor.1 - reachable).as_bytes());
        } else if self.cursor.1 < reachable {
            bytes.extend_from_slice(format!("\x1b[{}B", reachable - self.cursor.1).as_bytes());
        }
        if self.cursor.1 != reachable {
            // Vertical cursor motion cancels pending wrap; normalize the column.
            bytes.push(b'\r');
            self.cursor.0 = 0;
        }
        self.cursor.1 = reachable;
        // Ink src/log-update.ts:291-292: new blank rows need LF, not CNL/CUD,
        // to scroll at the bottom margin. CR makes this independent of ONLCR.
        while self.cursor.1 < y {
            bytes.extend_from_slice(b"\r\n");
            self.cursor = (0, self.cursor.1 + 1);
            self.reached = self.cursor.1;
        }
        if self.cursor.0 != x {
            if x == 0 {
                bytes.push(b'\r');
            } else {
                bytes.extend_from_slice(format!("\x1b[{}G", x + 1).as_bytes());
            }
            self.cursor.0 = x;
        }
    }

    fn append(&self, bytes: Vec<u8>, output: &mut Vec<u8>) {
        if bytes.is_empty() {
            return;
        }
        // Ink src/write-synchronized.ts and src/ink.tsx:802-815; keep the pair
        // and payload together instead of Ink's three stream writes.
        if self.synchronized {
            output.extend_from_slice(b"\x1b[?2026h");
        }
        output.extend(bytes);
        if self.synchronized {
            output.extend_from_slice(b"\x1b[?2026l");
        }
    }
}

fn is_continuation(row: &[Cell], x: usize) -> bool {
    matches!(row.get(x), Some(Cell::Continuation(_)))
}

fn content(row: &[Cell], columns: usize) -> Vec<Cell> {
    let mut end = row.len().min(columns);
    // Paint normally clips to columns. Do not emit half a glyph if a caller
    // hands us a wider grid.
    while is_continuation(row, end) {
        end -= 1;
    }
    // Same content boundary as paint::row_to_line (Ink src/output.ts:388-392).
    while end > 0
        && matches!(row.get(end - 1), Some(Cell::Glyph(c))
        if c.grapheme == " " && c.style == CharStyle::default())
    {
        end -= 1;
    }
    row[..end].to_vec()
}

fn write_cells(row: &[Cell], bytes: &mut Vec<u8>) {
    let line: Vec<_> = row
        .iter()
        .filter_map(|cell| match cell {
            Cell::Glyph(glyph) => Some(glyph.clone()),
            Cell::Continuation(_) => None,
        })
        .collect();
    bytes.extend_from_slice(styled::serialize(&line).as_bytes());
}

#[cfg(test)]
mod tests;
