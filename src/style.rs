use crate::{BorderStyle, Color};

/// Layout and box styles shared by [`View`](crate::View) and
/// [`Static`](crate::Static). Set them through the [`Styled`] builder methods.
///
/// Shorthands keep their own slot so that the most specific one wins when
/// resolved: `margin_top` over `margin_y` over `margin`, `border_top_color`
/// over `border_color`, `overflow_x` over `overflow`.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub(crate) position: Position,
    pub(crate) inset: Sides<Option<Inset>>,
    pub(crate) gap: Option<u16>,
    pub(crate) column_gap: Option<u16>,
    pub(crate) row_gap: Option<u16>,
    pub(crate) margin: Spacing<i16>,
    pub(crate) padding: Spacing<u16>,
    pub(crate) flex_grow: f32,
    pub(crate) flex_shrink: f32,
    pub(crate) flex_direction: FlexDirection,
    pub(crate) flex_basis: Dimension,
    pub(crate) flex_wrap: FlexWrap,
    pub(crate) align_items: Option<AlignItems>,
    pub(crate) align_self: Option<AlignSelf>,
    pub(crate) align_content: Option<AlignContent>,
    pub(crate) justify_content: Option<JustifyContent>,
    pub(crate) width: Dimension,
    pub(crate) height: Dimension,
    pub(crate) min_width: Dimension,
    pub(crate) min_height: Dimension,
    pub(crate) max_width: Dimension,
    pub(crate) max_height: Dimension,
    pub(crate) aspect_ratio: Option<f32>,
    pub(crate) display: Display,
    pub(crate) border_style: Option<BorderStyle>,
    pub(crate) border: Sides<BorderSide>,
    pub(crate) border_color: Option<Color>,
    pub(crate) border_dim_color: bool,
    pub(crate) border_background_color: Option<Color>,
    pub(crate) overflow: Option<Overflow>,
    pub(crate) overflow_x: Option<Overflow>,
    pub(crate) overflow_y: Option<Overflow>,
    pub(crate) content_offset_x: i16,
    pub(crate) content_offset_y: i16,
    pub(crate) background_color: Option<Color>,
}

/// Ink's `<Box>` defaults: a row that does not grow, shrinks, and does not
/// wrap.
impl Default for Style {
    fn default() -> Self {
        Style {
            position: Position::Relative,
            inset: Sides::default(),
            gap: None,
            column_gap: None,
            row_gap: None,
            margin: Spacing::default(),
            padding: Spacing::default(),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Row,
            flex_basis: Dimension::Auto,
            flex_wrap: FlexWrap::NoWrap,
            align_items: None,
            align_self: None,
            align_content: None,
            justify_content: None,
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_width: Dimension::Auto,
            max_height: Dimension::Auto,
            aspect_ratio: None,
            display: Display::Flex,
            border_style: None,
            border: Sides::default(),
            border_color: None,
            border_dim_color: false,
            border_background_color: None,
            overflow: None,
            overflow_x: None,
            overflow_y: None,
            content_offset_x: 0,
            content_offset_y: 0,
            background_color: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Sides<T> {
    pub(crate) top: T,
    pub(crate) right: T,
    pub(crate) bottom: T,
    pub(crate) left: T,
}

/// Margin or padding as set, before shorthands are resolved.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Spacing<T> {
    pub(crate) all: Option<T>,
    pub(crate) x: Option<T>,
    pub(crate) y: Option<T>,
    pub(crate) sides: Sides<Option<T>>,
}

/// Per-side border settings; `None` falls back to the shorthand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BorderSide {
    pub(crate) visible: bool,
    pub(crate) color: Option<Color>,
    pub(crate) dim_color: Option<bool>,
    pub(crate) background_color: Option<Color>,
}

impl Default for BorderSide {
    fn default() -> Self {
        BorderSide {
            visible: true,
            color: None,
            dim_color: None,
            background_color: None,
        }
    }
}

/// A width, height, or flex basis.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dimension {
    #[default]
    Auto,
    Cells(u16),
    Percent(f32),
}

impl From<u16> for Dimension {
    fn from(cells: u16) -> Self {
        Dimension::Cells(cells)
    }
}

impl From<Percent> for Dimension {
    fn from(Percent(p): Percent) -> Self {
        Dimension::Percent(p)
    }
}

/// A percentage of the parent, as in Ink's `width="50%"`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Percent(pub f32);

/// An offset for `top`, `right`, `bottom` and `left`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Inset {
    Cells(i16),
    Percent(f32),
}

impl From<i16> for Inset {
    fn from(cells: i16) -> Self {
        Inset::Cells(cells)
    }
}

impl From<Percent> for Inset {
    fn from(Percent(p): Percent) -> Self {
        Inset::Percent(p)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Position {
    #[default]
    Relative,
    Absolute,
    Static,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlignItems {
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
    Baseline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlignSelf {
    Auto,
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
    Baseline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Center,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Display {
    #[default]
    Flex,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
}

/// How [`Text`](crate::Text) fits a container narrower than its content.
/// Ink's `wrap="truncate"` is [`TextWrap::TruncateEnd`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextWrap {
    /// Break at word boundaries.
    #[default]
    Wrap,
    /// Fill every line to the full width, breaking inside words.
    Hard,
    TruncateStart,
    TruncateMiddle,
    TruncateEnd,
}

macro_rules! setters {
    ($($(#[$doc:meta])* $name:ident($ty:ty) => |$s:ident, $v:ident| $body:expr;)*) => {
        $(
            $(#[$doc])*
            fn $name(mut self, $v: $ty) -> Self {
                let $s = self.style_mut();
                $body;
                self
            }
        )*
    };
}

/// Builder methods for [`Style`], named after Ink's `<Box>` props.
pub trait Styled: Sized {
    #[doc(hidden)]
    fn style_mut(&mut self) -> &mut Style;

    setters! {
        position(Position) => |s, v| s.position = v;
        top(impl Into<Inset>) => |s, v| s.inset.top = Some(v.into());
        right(impl Into<Inset>) => |s, v| s.inset.right = Some(v.into());
        bottom(impl Into<Inset>) => |s, v| s.inset.bottom = Some(v.into());
        left(impl Into<Inset>) => |s, v| s.inset.left = Some(v.into());

        /// Sets both `column_gap` and `row_gap`.
        gap(u16) => |s, v| s.gap = Some(v);
        column_gap(u16) => |s, v| s.column_gap = Some(v);
        row_gap(u16) => |s, v| s.row_gap = Some(v);

        margin(i16) => |s, v| s.margin.all = Some(v);
        margin_x(i16) => |s, v| s.margin.x = Some(v);
        margin_y(i16) => |s, v| s.margin.y = Some(v);
        margin_top(i16) => |s, v| s.margin.sides.top = Some(v);
        margin_right(i16) => |s, v| s.margin.sides.right = Some(v);
        margin_bottom(i16) => |s, v| s.margin.sides.bottom = Some(v);
        margin_left(i16) => |s, v| s.margin.sides.left = Some(v);

        padding(u16) => |s, v| s.padding.all = Some(v);
        padding_x(u16) => |s, v| s.padding.x = Some(v);
        padding_y(u16) => |s, v| s.padding.y = Some(v);
        padding_top(u16) => |s, v| s.padding.sides.top = Some(v);
        padding_right(u16) => |s, v| s.padding.sides.right = Some(v);
        padding_bottom(u16) => |s, v| s.padding.sides.bottom = Some(v);
        padding_left(u16) => |s, v| s.padding.sides.left = Some(v);

        flex_grow(f32) => |s, v| s.flex_grow = v;
        flex_shrink(f32) => |s, v| s.flex_shrink = v;
        flex_direction(FlexDirection) => |s, v| s.flex_direction = v;
        flex_basis(impl Into<Dimension>) => |s, v| s.flex_basis = v.into();
        flex_wrap(FlexWrap) => |s, v| s.flex_wrap = v;
        align_items(AlignItems) => |s, v| s.align_items = Some(v);
        align_self(AlignSelf) => |s, v| s.align_self = Some(v);
        align_content(AlignContent) => |s, v| s.align_content = Some(v);
        justify_content(JustifyContent) => |s, v| s.justify_content = Some(v);

        width(impl Into<Dimension>) => |s, v| s.width = v.into();
        height(impl Into<Dimension>) => |s, v| s.height = v.into();
        min_width(impl Into<Dimension>) => |s, v| s.min_width = v.into();
        min_height(impl Into<Dimension>) => |s, v| s.min_height = v.into();
        max_width(impl Into<Dimension>) => |s, v| s.max_width = v.into();
        max_height(impl Into<Dimension>) => |s, v| s.max_height = v.into();
        aspect_ratio(f32) => |s, v| s.aspect_ratio = Some(v);
        display(Display) => |s, v| s.display = v;

        border_style(BorderStyle) => |s, v| s.border_style = Some(v);
        border_top(bool) => |s, v| s.border.top.visible = v;
        border_right(bool) => |s, v| s.border.right.visible = v;
        border_bottom(bool) => |s, v| s.border.bottom.visible = v;
        border_left(bool) => |s, v| s.border.left.visible = v;
        border_color(impl Into<Option<Color>>) => |s, v| s.border_color = v.into();
        border_top_color(impl Into<Option<Color>>) => |s, v| s.border.top.color = v.into();
        border_right_color(impl Into<Option<Color>>) => |s, v| s.border.right.color = v.into();
        border_bottom_color(impl Into<Option<Color>>) => |s, v| s.border.bottom.color = v.into();
        border_left_color(impl Into<Option<Color>>) => |s, v| s.border.left.color = v.into();
        border_dim_color(bool) => |s, v| s.border_dim_color = v;
        border_top_dim_color(bool) => |s, v| s.border.top.dim_color = Some(v);
        border_right_dim_color(bool) => |s, v| s.border.right.dim_color = Some(v);
        border_bottom_dim_color(bool) => |s, v| s.border.bottom.dim_color = Some(v);
        border_left_dim_color(bool) => |s, v| s.border.left.dim_color = Some(v);
        border_background_color(impl Into<Option<Color>>) => |s, v| s.border_background_color = v.into();
        border_top_background_color(impl Into<Option<Color>>) => |s, v| s.border.top.background_color = v.into();
        border_right_background_color(impl Into<Option<Color>>) => |s, v| s.border.right.background_color = v.into();
        border_bottom_background_color(impl Into<Option<Color>>) => |s, v| s.border.bottom.background_color = v.into();
        border_left_background_color(impl Into<Option<Color>>) => |s, v| s.border.left.background_color = v.into();

        overflow(Overflow) => |s, v| s.overflow = Some(v);
        overflow_x(Overflow) => |s, v| s.overflow_x = Some(v);
        overflow_y(Overflow) => |s, v| s.overflow_y = Some(v);
        /// Shifts children left by this many columns; combine with
        /// [`Overflow::Hidden`] to scroll.
        content_offset_x(i16) => |s, v| s.content_offset_x = v;
        /// Shifts children up by this many rows.
        content_offset_y(i16) => |s, v| s.content_offset_y = v;
        /// Fills the box, and is inherited by descendant text that sets no
        /// background of its own.
        background_color(impl Into<Option<Color>>) => |s, v| s.background_color = v.into();
    }
}
