//! Build terminal user interfaces from components, in the spirit of
//! [Ink](https://github.com/vadimdemedes/ink).
//!
//! Build trees with [`View`], [`Text`] and [`component`]. Components use
//! [`Scope`] hooks for state, memoized values and effects. [`State`] handles
//! schedule their owning component when written on the UI thread.
//! Bind [`Scope::use_node_ref`] handles with [`View::node_ref`] and call
//! [`measure_element`] to read the latest committed layout in integer cells.
//! [`render_to_string`] mounts a tree, settles layout effects, paints one
//! frame, runs passive effects and unmounts with cleanup.
//! [`testing::render`] keeps a tree mounted, records presented frames and
//! exposes rerender, settle, input and clock advance operations without
//! terminal I/O.
//! [`testing::mount`] drives a live app with [`Options`], an in-memory terminal,
//! and a clock advanced by the test. [`testing::TestApp::input`] feeds bytes to
//! active [`Scope::use_input`], [`Scope::use_paste`] and [`Scope::use_focus`] hooks.
//!
//! See `docs/design.md` for the architecture and the milestone plan.

mod app;
mod border;
mod color;
mod components;
#[cfg(unix)]
mod driver;
mod element;
mod error;
mod host;
mod input;
mod layout;
mod output;
mod paint;
mod render;
mod runtime;
mod style;
mod terminal;
mod text;

pub mod testing;

pub use app::Options;
pub use border::{BorderStyle, BoxChars};
pub use color::{Color, ColorLevel, ParseColorError};
pub use components::{Newline, Spacer, Static, Text, Transform, View};
#[cfg(unix)]
pub use driver::{Instance, render};
pub use element::{Element, Key, TextChild};
pub use error::Error;
pub use input::kitty::{KittyFlags, KittyKeyboard};
pub use input::{KeyEvent, KeyEventType};
pub use render::{RenderOptions, render_to_string};
pub use runtime::{
    AppHandle, BoxMetrics, CursorPosition, ElementMetrics, FocusManager, FocusOptions, NodeRef,
    OutputStream, OutputWriter, Scope, State, WindowSize, component, measure_element,
};
pub use style::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, Inset,
    JustifyContent, Overflow, Percent, Position, Style, Styled, TextWrap,
};
