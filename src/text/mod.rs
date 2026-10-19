//! Text handling below the layout layer: ANSI parsing, sanitizing,
//! measuring and wrapping.

pub(crate) mod ansi;
pub(crate) mod measure;
pub(crate) mod sanitize;
pub(crate) mod width;
mod width_data;
pub(crate) mod wrap;
