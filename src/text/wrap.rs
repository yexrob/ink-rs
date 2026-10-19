use crate::TextWrap;

/// Wraps or truncates `text` to `max_width` columns, keeping ANSI styling
/// intact across the line breaks it inserts.
pub(crate) fn wrap_text(text: &str, max_width: usize, wrap: TextWrap) -> String {
    let _ = (text, max_width, wrap);
    todo!()
}

#[cfg(test)]
mod tests;
