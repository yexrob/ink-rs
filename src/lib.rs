//! Build terminal user interfaces from components, in the spirit of
//! [Ink](https://github.com/vadimdemedes/ink).
//!
//! The current milestone (M0) renders element trees to strings. Elements are
//! plain data built with [`View`], [`Text`] and friends; [`render_to_string`]
//! lays them out with flexbox and paints them into terminal output.
//!
//! See `docs/design.md` for the architecture and the milestone plan.

// Skeleton stage: nothing reads the element data or calls the text
// functions until the M0 pipeline lands. Remove with the first implementation.
#![allow(dead_code)]

mod border;
mod color;
mod components;
mod element;
mod render;
mod style;
mod text;

pub use border::{BorderStyle, BoxChars};
pub use color::{Color, ColorLevel, ParseColorError};
pub use components::{Newline, Spacer, Static, Text, Transform, View};
pub use element::{Element, TextChild};
pub use render::{RenderOptions, render_to_string};
pub use style::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, Inset,
    JustifyContent, Overflow, Percent, Position, Style, Styled, TextWrap,
};
