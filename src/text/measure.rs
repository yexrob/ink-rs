use super::styled::StyledLine;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextSize {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

/// The widest line in terminal columns, and the number of lines.
pub(crate) fn measure_text(lines: &[StyledLine]) -> TextSize {
    TextSize {
        width: lines
            .iter()
            .map(|line| line.iter().map(|cell| cell.width).sum())
            .max()
            .unwrap_or(0),
        height: lines.len(),
    }
}

#[cfg(test)]
mod tests;
