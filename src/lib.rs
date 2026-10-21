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
//! The default `macros` feature provides `rsx!`, syntax sugar for builder trees.
//!
//! ```
//! # #[cfg(feature = "macros")]
//! # {
//! use ink_rs::{Text, View, render_to_string, rsx};
//!
//! let tree = rsx! { View { Text { "Hello, Ink!" } } };
//! assert_eq!(render_to_string(tree, Default::default()), "Hello, Ink!");
//! # }
//! ```
//!
//! On Linux, macOS and Windows, `render` mounts on process stdout and its
//! `wait_until_exit` blocks the calling thread. The default `tokio` feature adds `ink_rs::tokio` and
//! `Scope::use_future` for tasks on a local runtime or `tokio::task::LocalSet`.
//! Tasks can write `State` on the UI thread; `use_future` aborts them on
//! dependency changes and unmount, and keeps the app alive until they finish.
//! A sync `ink_rs::render(..).wait_until_exit()` blocks every task on its thread.
//!
//! See [docs/design.md](https://github.com/yexrob/ink-rs/blob/main/docs/design.md)
//! for the architecture.

#![cfg_attr(not(windows), forbid(unsafe_code))]
#![cfg_attr(windows, deny(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod app;
mod border;
mod callback;
mod color;
mod components;
mod driver;
mod element;
mod error;
mod error_overview;
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
pub use callback::Callback;
pub use color::{Color, ColorLevel, ParseColorError};
pub use components::{Newline, Spacer, Static, Text, Transform, View};
#[cfg(feature = "tokio")]
pub use driver::tokio;
pub use driver::{Instance, render};
pub use element::{Element, Key, TextChild};
pub use error::{Error, PanicError, PanicLocation};
#[cfg(feature = "macros")]
pub use ink_rs_macros::{Props, rsx};
pub use input::kitty::{KittyFlags, KittyKeyboard};
pub use input::{KeyEvent, KeyEventType};
pub use render::{RenderOptions, render_to_string};
pub use runtime::{
    AlreadySuspended, Animation, AnimationOptions, AppHandle, BoxMetrics, CursorPosition,
    ElementMetrics, FocusManager, FocusOptions, NodeRef, OutputStream, OutputWriter, Scope, State,
    WindowSize, component, measure_element,
};
pub use style::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, Inset,
    JustifyContent, Overflow, Percent, Position, Style, Styled, TextWrap,
};
pub use text::text_width;
