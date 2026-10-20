//! Build terminal user interfaces from components, in the spirit of
//! [Ink](https://github.com/vadimdemedes/ink).
//!
//! Build trees with [`View`], [`Text`] and [`component`]. Components use
//! [`Scope`] hooks for state, memoized values and effects. [`State`] handles
//! schedule their owning component when written on the UI thread.
//! [`render_to_string`] mounts a tree, settles layout effects, paints one
//! frame, runs passive effects and unmounts with cleanup.
//! [`testing::render`] keeps a tree mounted, records presented frames and
//! exposes rerender and settle operations without terminal I/O.
//!
//! See `docs/design.md` for the architecture and the milestone plan.

mod border;
mod color;
mod components;
mod element;
mod host;
mod layout;
mod paint;
mod render;
mod runtime;
mod style;
mod text;

pub mod testing;

pub use border::{BorderStyle, BoxChars};
pub use color::{Color, ColorLevel, ParseColorError};
pub use components::{Newline, Spacer, Static, Text, Transform, View};
pub use element::{Element, Key, TextChild};
pub use render::{RenderOptions, render_to_string};
pub use runtime::{Scope, State, component};
pub use style::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, Inset,
    JustifyContent, Overflow, Percent, Position, Style, Styled, TextWrap,
};
