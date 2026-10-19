/// Keeps SGR styling and OSC sequences such as hyperlinks, and removes every
/// other control sequence and C0 control except tab and newline: cursor
/// movement or screen clearing would corrupt layout. Colon SGR parameters
/// are rewritten to the semicolon form or dropped.
pub(crate) fn sanitize_ansi(text: &str) -> String {
    let _ = text;
    todo!()
}

#[cfg(test)]
mod tests;
