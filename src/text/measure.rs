#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextSize {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

/// The widest line in terminal columns, and the number of lines. ANSI
/// sequences take no space.
pub(crate) fn measure_text(text: &str) -> TextSize {
    let _ = text;
    todo!()
}

#[cfg(test)]
mod tests;
