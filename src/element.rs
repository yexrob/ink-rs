use crate::{Newline, Spacer, Static, Text, Transform, View};

/// A node of the UI tree. Build one from [`View`], [`Text`], [`Static`],
/// [`Transform`], [`Newline`] or [`Spacer`], or combine several with
/// [`Element::fragment`].
///
/// Strings are not elements: as in Ink, text has to live inside a [`Text`].
#[derive(Clone, Debug, Default)]
pub struct Element(pub(crate) Node);

#[derive(Clone, Debug, Default)]
pub(crate) enum Node {
    #[default]
    Empty,
    Fragment(Vec<Element>),
    View(View),
    Text(Text),
    Transform(Transform),
    Static(Static),
    /// A string child of [`Text`] or [`Transform`].
    Str(String),
}

impl Element {
    /// An element that renders nothing.
    pub fn empty() -> Element {
        Element(Node::Empty)
    }

    /// Siblings without a wrapper, like React's `<>…</>`.
    pub fn fragment<I>(children: I) -> Element
    where
        I: IntoIterator,
        I::Item: Into<Element>,
    {
        Element(Node::Fragment(
            children.into_iter().map(Into::into).collect(),
        ))
    }
}

macro_rules! into_element {
    ($($ty:ident),*) => {
        $(impl From<$ty> for Element {
            fn from(v: $ty) -> Self {
                Element(Node::$ty(v))
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
        TextChild(Element(Node::Str(s.to_owned())))
    }
}

impl From<String> for TextChild {
    fn from(s: String) -> Self {
        TextChild(Element(Node::Str(s)))
    }
}

impl From<&String> for TextChild {
    fn from(s: &String) -> Self {
        s.as_str().into()
    }
}

impl From<Newline> for TextChild {
    fn from(n: Newline) -> Self {
        TextChild(Element(Node::Str("\n".repeat(n.count))))
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
