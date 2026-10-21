use std::fmt;
use std::rc::Rc;

use crate::{Color, Element, Key, NodeRef, Position, Style, Styled, TextChild, TextWrap};

/// A flexbox container, Ink's `<Box>`. Style it with the [`Styled`] methods.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct View {
    pub(crate) node_ref: Option<NodeRef>,
    pub(crate) style: Style,
    pub(crate) children: Vec<Element>,
}

impl View {
    /// Binds a ref to this View's host node at commit, for [`crate::measure_element`].
    pub fn node_ref(mut self, node_ref: NodeRef) -> Self {
        self.node_ref = Some(node_ref);
        self
    }

    /// Identifies this element among its siblings.
    pub fn key(self, key: impl Into<Key>) -> Element {
        Element::from(self).key(key)
    }

    pub fn new() -> View {
        View::default()
    }

    pub fn child(mut self, child: impl Into<Element>) -> View {
        self.children.push(child.into());
        self
    }

    pub fn children<I>(mut self, children: I) -> View
    where
        I: IntoIterator,
        I::Item: Into<Element>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }
}

impl Styled for View {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

/// Styled text, Ink's `<Text>`. Children are strings, nested `Text`, or
/// elements that render text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Text {
    pub(crate) style: TextStyle,
    pub(crate) wrap: TextWrap,
    pub(crate) children: Vec<Element>,
}

/// Unset fields inherit from the enclosing `Text`; an unset background also
/// inherits from the enclosing [`View`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TextStyle {
    pub(crate) color: Option<Color>,
    pub(crate) background_color: Option<Color>,
    pub(crate) dim_color: bool,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) strikethrough: bool,
    pub(crate) inverse: bool,
}

impl Text {
    /// Identifies this element among its siblings.
    pub fn key(self, key: impl Into<Key>) -> Element {
        Element::from(self).key(key)
    }

    pub fn new(content: impl Into<TextChild>) -> Text {
        Text::default().child(content)
    }

    pub fn child(mut self, child: impl Into<TextChild>) -> Text {
        self.children.push(child.into().0);
        self
    }

    pub fn children<I>(mut self, children: I) -> Text
    where
        I: IntoIterator,
        I::Item: Into<TextChild>,
    {
        self.children
            .extend(children.into_iter().map(|c| c.into().0));
        self
    }

    pub fn color(mut self, color: impl Into<Option<Color>>) -> Text {
        self.style.color = color.into();
        self
    }

    pub fn background_color(mut self, color: impl Into<Option<Color>>) -> Text {
        self.style.background_color = color.into();
        self
    }

    pub fn dim_color(mut self, on: bool) -> Text {
        self.style.dim_color = on;
        self
    }

    pub fn bold(mut self, on: bool) -> Text {
        self.style.bold = on;
        self
    }

    pub fn italic(mut self, on: bool) -> Text {
        self.style.italic = on;
        self
    }

    pub fn underline(mut self, on: bool) -> Text {
        self.style.underline = on;
        self
    }

    pub fn strikethrough(mut self, on: bool) -> Text {
        self.style.strikethrough = on;
        self
    }

    pub fn inverse(mut self, on: bool) -> Text {
        self.style.inverse = on;
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Text {
        self.wrap = wrap;
        self
    }
}

type TransformFn = dyn Fn(&str, usize) -> String;

/// Rewrites the rendered output of its children line by line, Ink's
/// `<Transform>`. The function receives each line, ANSI styling included,
/// and its index.
#[derive(Clone)]
pub struct Transform {
    pub(crate) transform: Rc<TransformFn>,
    pub(crate) children: Vec<Element>,
}

impl Transform {
    /// Identifies this element among its siblings.
    pub fn key(self, key: impl Into<Key>) -> Element {
        Element::from(self).key(key)
    }

    pub fn new(transform: impl Fn(&str, usize) -> String + 'static) -> Transform {
        Transform {
            transform: Rc::new(transform),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl Into<TextChild>) -> Transform {
        self.children.push(child.into().0);
        self
    }

    pub fn children<I>(mut self, children: I) -> Transform
    where
        I: IntoIterator,
        I::Item: Into<TextChild>,
    {
        self.children
            .extend(children.into_iter().map(|c| c.into().0));
        self
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.transform, &other.transform) && self.children == other.children
    }
}

impl fmt::Debug for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transform")
            .field("children", &self.children)
            .finish_non_exhaustive()
    }
}

/// Output that is written once above the live region and never redrawn,
/// Ink's `<Static>`: logs, finished tasks. Each item renders through the
/// closure, which receives the item and its index.
#[derive(Clone)]
pub struct Static {
    pub(crate) style: Style,
    pub(crate) len: usize,
    pub(crate) render: Rc<dyn Fn(usize) -> Element>,
}

impl Static {
    /// Identifies this element among its siblings.
    pub fn key(self, key: impl Into<Key>) -> Element {
        Element::from(self).key(key)
    }

    pub fn new<T: 'static>(
        items: impl Into<Rc<[T]>>,
        render: impl Fn(&T, usize) -> Element + 'static,
    ) -> Static {
        let items = items.into();
        Static {
            style: Style {
                position: Position::Absolute,
                flex_direction: crate::FlexDirection::Column,
                ..Style::default()
            },
            len: items.len(),
            render: Rc::new(move |i| render(&items[i], i)),
        }
    }
}

impl Styled for Static {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl PartialEq for Static {
    fn eq(&self, other: &Self) -> bool {
        self.style == other.style
            && self.len == other.len
            && Rc::ptr_eq(&self.render, &other.render)
    }
}

impl fmt::Debug for Static {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Static")
            .field("style", &self.style)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

/// One or more line breaks inside [`Text`], Ink's `<Newline>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Newline {
    pub(crate) count: usize,
}

impl Newline {
    pub fn new() -> Newline {
        Newline { count: 1 }
    }

    pub fn count(count: usize) -> Newline {
        Newline { count }
    }
}

impl Default for Newline {
    fn default() -> Self {
        Newline::new()
    }
}

/// Fills the free space along its parent's main axis, Ink's `<Spacer>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Spacer;

impl From<Spacer> for View {
    fn from(_: Spacer) -> View {
        View::new().flex_grow(1.0)
    }
}
