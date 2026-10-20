//! Text handling below the layout layer: ANSI parsing, sanitizing,
//! measuring and wrapping.

pub(crate) mod ansi;
pub(crate) mod measure;
pub(crate) mod sanitize;
pub(crate) mod styled;
pub(crate) mod width;
mod width_data;
pub(crate) mod wrap;

/// Returns the renderer's column width of the widest line in `text`.
/// ANSI escape sequences occupy no columns; tabs use eight-column stops.
/// Graphemes follow string-width 8.3 rules, as they do in paint.
///
/// Use this when calculating columns for [`crate::Scope::use_cursor`]:
/// `unicode-width` disagrees with the renderer on ZWJ emoji and Hangul jamo.
pub fn text_width(text: &str) -> usize {
    let lines = styled::parse(text)
        .iter()
        .map(|line| styled::expand_tabs(line))
        .collect::<Vec<_>>();
    measure::measure_text(&lines).width
}
