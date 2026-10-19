use crate::TextWrap;

use super::styled::{StyledChar, StyledLine, StyledText, expand_tabs};

/// Wraps or truncates each line to `max_width` columns; cells keep their
/// style across the breaks this inserts.
pub(crate) fn wrap_text(lines: &[StyledLine], max_width: usize, wrap: TextWrap) -> StyledText {
    if matches!(wrap, TextWrap::Wrap | TextWrap::Hard) {
        if max_width == 0 {
            return lines.to_vec();
        }
        lines
            .iter()
            .flat_map(|line| wrap_line(&expand_tabs(line), max_width, wrap == TextWrap::Hard))
            .collect()
    } else {
        lines
            .iter()
            .map(|line| truncate(line, max_width, wrap))
            .collect()
    }
}

fn line_width(line: &[StyledChar]) -> usize {
    line.iter().map(|cell| cell.width).sum()
}

fn wrap_word(rows: &mut StyledText, word: &[StyledChar], columns: usize, row_width: &mut usize) {
    for (index, cell) in word.iter().enumerate() {
        if cell.width > 0 && *row_width > 0 && cell.width > columns.saturating_sub(*row_width) {
            rows.push(Vec::new());
            *row_width = 0;
        }
        rows.last_mut().unwrap().push(cell.clone());
        *row_width += cell.width;
        if *row_width == columns && index + 1 < word.len() {
            rows.push(Vec::new());
            *row_width = 0;
        }
    }
    // wrap-ansi keeps a trailing zero-width fragment on the preceding row.
    if *row_width == 0 && rows.len() > 1 && !rows.last().unwrap().is_empty() {
        let last = rows.pop().unwrap();
        let previous = rows.last_mut().unwrap();
        previous.extend(last);
        *row_width = line_width(previous);
    }
}

fn wrap_line(line: &[StyledChar], columns: usize, hard: bool) -> StyledText {
    let mut rows = vec![Vec::new()];
    let mut row_width = 0;
    let mut start = 0;
    // wrap-ansi splits on ASCII spaces, including empty words, with trim:false.
    // Keep the separator cell so its style and hyperlink survive the split.
    loop {
        let end = line[start..]
            .iter()
            .position(|cell| cell.grapheme.starts_with(' '))
            .map_or(line.len(), |i| start + i);
        let word = &line[start..end];
        let width = line_width(word);
        if !hard && width > columns {
            let remaining = columns.saturating_sub(row_width);
            let this_breaks = 1 + (width - remaining - 1) / columns;
            let next_breaks = (width - 1) / columns;
            if next_breaks < this_breaks {
                rows.push(Vec::new());
                row_width = 0;
            }
            wrap_word(&mut rows, word, columns, &mut row_width);
        } else if hard {
            wrap_word(&mut rows, word, columns, &mut row_width);
        } else {
            if width > 0 && row_width > 0 && width > columns.saturating_sub(row_width) {
                rows.push(Vec::new());
                row_width = 0;
            }
            rows.last_mut().unwrap().extend_from_slice(word);
            row_width += width;
        }
        if end == line.len() {
            break;
        }
        if row_width >= columns {
            rows.push(Vec::new());
            row_width = 0;
        }
        rows.last_mut().unwrap().push(line[end].clone());
        row_width += line[end].width;
        start = end + 1;
    }
    rows
}

fn prefix(line: &[StyledChar], budget: usize) -> &[StyledChar] {
    let mut width = 0;
    let end = line
        .iter()
        .position(|cell| {
            width += cell.width;
            width > budget
        })
        .unwrap_or(line.len());
    &line[..end]
}

fn suffix(line: &[StyledChar], budget: usize) -> &[StyledChar] {
    let mut width = 0;
    let mut start = line.len();
    for cell in line.iter().rev() {
        if cell.width > budget.saturating_sub(width) {
            break;
        }
        start -= 1;
        width += cell.width;
    }
    &line[start..]
}

fn truncate(line: &[StyledChar], columns: usize, mode: TextWrap) -> StyledLine {
    if columns == 0 {
        return Vec::new();
    }
    if line_width(line) <= columns {
        return line.to_vec();
    }
    let (left, right) = match mode {
        TextWrap::TruncateStart => (&[][..], suffix(line, columns - 1)),
        TextWrap::TruncateMiddle => {
            let half = columns / 2;
            (prefix(line, half), suffix(line, columns - half - 1))
        }
        _ => (prefix(line, columns - 1), &[][..]),
    };
    // cli-truncate puts the ellipsis inside the SGR span of the kept side at
    // the start and end, outside any hyperlink; a middle cut concatenates, and
    // one column returns the bare ellipsis.
    let adjacent = match mode {
        _ if columns == 1 => None,
        TextWrap::TruncateStart => right.first(),
        TextWrap::TruncateMiddle => None,
        _ => left.last(),
    };
    let mut style = adjacent.map(|cell| cell.style.clone()).unwrap_or_default();
    style.link = None;
    let mut result = left.to_vec();
    result.push(StyledChar {
        grapheme: "\u{2026}".to_owned(),
        width: 1,
        style,
    });
    result.extend_from_slice(right);
    result
}

#[cfg(test)]
mod tests;
