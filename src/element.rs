use crate::runtime::Component;
use crate::{Newline, Spacer, Static, Text, Transform, View};

/// A node of the UI tree. Build one from [`View`], [`Text`], [`Static`],
/// [`Transform`], [`Newline`] or [`Spacer`], or combine several with
/// [`Element::fragment`].
///
/// Strings are not elements: as in Ink, text has to live inside a [`Text`].
#[derive(Clone, Debug, Default)]
pub struct Element(pub(crate) Node, pub(crate) Option<Key>);

#[derive(Clone, Debug, Default)]
pub(crate) enum Node {
    #[default]
    Empty,
    Fragment(Vec<Element>),
    View(View),
    Text(Text),
    Transform(Transform),
    Static(Static),
    Component(Component),
    /// A string child of [`Text`] or [`Transform`].
    Str(String),
}

/// A sibling identity. Integer keys and their string forms denote the same key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(String);

impl From<String> for Key {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for Key {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<&String> for Key {
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}
macro_rules! integer_keys {
    ($($ty:ty),*) => {$(impl From<$ty> for Key {
        fn from(value: $ty) -> Self { Self(value.to_string()) }
    })*};
}
integer_keys!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl Element {
    /// Identifies this element among its siblings, like React's `key`.
    pub fn key(mut self, key: impl Into<Key>) -> Element {
        self.1 = Some(key.into());
        self
    }

    /// An element that renders nothing.
    pub fn empty() -> Element {
        Element(Node::Empty, None)
    }

    /// Siblings without a wrapper, like React's `<>…</>`.
    pub fn fragment<I>(children: I) -> Element
    where
        I: IntoIterator,
        I::Item: Into<Element>,
    {
        Element(
            Node::Fragment(children.into_iter().map(Into::into).collect()),
            None,
        )
    }
}

macro_rules! into_element {
    ($($ty:ident),*) => {
        $(impl From<$ty> for Element {
            fn from(v: $ty) -> Self {
                Element(Node::$ty(v), None)
            }
        })*
    };
}

into_element!(View, Text, Transform, Static);

impl From<Newline> for Element {
    fn from(v: Newline) -> Self {
        Text::new(v).into()
    }
}

impl From<Spacer> for Element {
    fn from(_: Spacer) -> Self {
        View::from(Spacer).into()
    }
}

impl<T: Into<Element>> From<Option<T>> for Element {
    fn from(v: Option<T>) -> Self {
        v.map_or_else(Element::empty, Into::into)
    }
}

impl<T: Into<Element>> From<Vec<T>> for Element {
    fn from(v: Vec<T>) -> Self {
        Element::fragment(v)
    }
}

/// A child of [`Text`] or [`Transform`]: a string, nested text, or an
/// element that renders text. An element that renders a [`View`] inside text
/// is an error at render time, as in Ink.
#[derive(Clone, Debug)]
pub struct TextChild(pub(crate) Element);

impl From<&str> for TextChild {
    fn from(s: &str) -> Self {
        TextChild(Element(Node::Str(s.to_owned()), None))
    }
}

impl From<String> for TextChild {
    fn from(s: String) -> Self {
        TextChild(Element(Node::Str(s), None))
    }
}

impl From<&String> for TextChild {
    fn from(s: &String) -> Self {
        s.as_str().into()
    }
}

impl From<Newline> for TextChild {
    fn from(n: Newline) -> Self {
        TextChild(Element(Node::Str("\n".repeat(n.count)), None))
    }
}

impl From<Text> for TextChild {
    fn from(t: Text) -> Self {
        TextChild(t.into())
    }
}

impl From<Transform> for TextChild {
    fn from(t: Transform) -> Self {
        TextChild(t.into())
    }
}

impl From<Element> for TextChild {
    fn from(e: Element) -> Self {
        TextChild(e)
    }
}
