use taffy::prelude::TaffyMaxContent;
use taffy::{self as t, AvailableSpace, NodeId, Rect, Size, compute::compute_leaf_layout};

use crate::host::HostTree;
use crate::style::{Sides, Spacing};
use crate::text::{measure::measure_text, wrap::wrap_text};
use crate::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, Inset,
    JustifyContent, Position, Style, TextWrap,
};

pub(crate) fn compute(tree: &mut HostTree, root: NodeId) {
    // Consumers use get() for Yoga-compatible text rounding.
    tree.disable_rounding();
    tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |inputs, _, context, style| {
        compute_leaf_layout(
            inputs,
            style,
            |_, _| 0.0,
            |known, available| {
                let Some(text) = context.and_then(|node| node.text.as_ref()) else {
                    return Size::ZERO;
                };
                let lines = text.squash();
                let natural = measure_text(&lines);
                // Ink dom.ts treats both intrinsic requests as Yoga's undefined mode.
                let width = known.width.or(match available.width {
                    AvailableSpace::Definite(width) => Some(width),
                    _ => None,
                });
                let measured =
                    if let Some(width) = width.filter(|&width| (natural.width as f32) > width) {
                        let measured =
                            measure_text(&wrap_text(&lines, width.max(0.0) as usize, text.wrap));
                        Size {
                            width: if matches!(text.wrap, TextWrap::Wrap | TextWrap::Hard) {
                                measured.width as f32
                            } else {
                                width
                            },
                            height: measured.height as f32,
                        }
                    } else {
                        Size {
                            width: natural.width as f32,
                            height: natural.height as f32,
                        }
                    };
                Size {
                    width: known.width.unwrap_or(measured.width.ceil()),
                    height: known.height.unwrap_or(measured.height),
                }
            },
        )
    })
    .unwrap();
}

/// Yoga's text nodes floor their origin and round their extent outwards when
/// fractional. Taffy's uniform rounding can move centered text by one cell.
pub(crate) fn get(
    tree: &HostTree,
    node: NodeId,
    parent_absolute: t::Point<f32>,
) -> (t::Layout, t::Point<f32>) {
    let mut layout = *tree.unrounded_layout(node);
    let absolute = t::Point {
        x: parent_absolute.x + layout.location.x,
        y: parent_absolute.y + layout.location.y,
    };
    let text = tree.get_node_context(node).unwrap().text.is_some();
    fn round(value: f32, floor: bool, ceil: bool) -> f32 {
        let fraction = value - value.floor();
        if fraction < 0.0001 {
            value.floor()
        } else if fraction > 0.9999 || ceil {
            value.ceil()
        } else if floor {
            value.floor()
        } else {
            (value + 0.5).floor()
        }
    }
    fn extent(start: f32, length: f32, text: bool) -> f32 {
        let fractional = (length - length.round()).abs() > 0.0001;
        round(start + length, text && !fractional, text && fractional) - round(start, text, false)
    }
    layout.location.x = round(layout.location.x, text, false);
    layout.location.y = round(layout.location.y, text, false);
    layout.size.width = extent(absolute.x, layout.size.width, text);
    layout.size.height = extent(absolute.y, layout.size.height, text);
    (layout, absolute)
}

fn dimension(value: Dimension) -> t::Dimension {
    match value {
        Dimension::Auto => t::Dimension::auto(),
        Dimension::Cells(value) => t::Dimension::length(value.into()),
        Dimension::Percent(value) => t::Dimension::percent(value / 100.0),
    }
}

fn limit(value: Dimension, minimum: bool) -> t::LengthPercentageAuto {
    match value {
        Dimension::Auto if minimum => t::LengthPercentageAuto::length(0.0),
        Dimension::Auto => t::LengthPercentageAuto::auto(),
        Dimension::Cells(value) => t::LengthPercentageAuto::length(value.into()),
        Dimension::Percent(value) => t::LengthPercentageAuto::percent(value / 100.0),
    }
}

fn inset(value: Option<Inset>) -> t::LengthPercentageAuto {
    match value {
        None => t::LengthPercentageAuto::auto(),
        Some(Inset::Cells(value)) => t::LengthPercentageAuto::length(value.into()),
        Some(Inset::Percent(value)) => t::LengthPercentageAuto::percent(value / 100.0),
    }
}

fn rect<T: Copy, U>(sides: Sides<T>, f: impl Fn(T) -> U) -> Rect<U> {
    Rect {
        top: f(sides.top),
        right: f(sides.right),
        bottom: f(sides.bottom),
        left: f(sides.left),
    }
}

fn spacing<T: Copy + Default>(value: Spacing<T>) -> Sides<T> {
    Sides {
        top: value
            .sides
            .top
            .or(value.y)
            .or(value.all)
            .unwrap_or_default(),
        bottom: value
            .sides
            .bottom
            .or(value.y)
            .or(value.all)
            .unwrap_or_default(),
        left: value
            .sides
            .left
            .or(value.x)
            .or(value.all)
            .unwrap_or_default(),
        right: value
            .sides
            .right
            .or(value.x)
            .or(value.all)
            .unwrap_or_default(),
    }
}

pub(crate) fn map_style(style: &Style) -> t::Style {
    t::Style {
        display: match style.display {
            Display::Flex => t::Display::Flex,
            Display::None => t::Display::None,
        },
        position: match style.position {
            Position::Absolute => t::Position::Absolute,
            _ => t::Position::Relative,
        },
        inset: rect(
            if style.position == Position::Static {
                Sides::default()
            } else {
                style.inset
            },
            inset,
        ),
        size: Size {
            width: dimension(style.width),
            height: dimension(style.height),
        },
        min_size: Size {
            width: limit(style.min_width, true),
            height: limit(style.min_height, true),
        },
        max_size: Size {
            width: limit(style.max_width, false),
            height: limit(style.max_height, false),
        },
        aspect_ratio: style.aspect_ratio,
        margin: rect(spacing(style.margin), |v| {
            t::LengthPercentageAuto::length(v.into())
        }),
        padding: rect(spacing(style.padding), |v| {
            t::LengthPercentage::length(v.into())
        }),
        border: rect(style.border, |side| {
            t::LengthPercentage::length(if style.border_style.is_some() && side.visible {
                1.0
            } else {
                0.0
            })
        }),
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,
        flex_basis: dimension(style.flex_basis),
        flex_direction: match style.flex_direction {
            FlexDirection::Row => t::FlexDirection::Row,
            FlexDirection::Column => t::FlexDirection::Column,
            FlexDirection::RowReverse => t::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => t::FlexDirection::ColumnReverse,
        },
        flex_wrap: match style.flex_wrap {
            FlexWrap::NoWrap => t::FlexWrap::NoWrap,
            FlexWrap::Wrap => t::FlexWrap::Wrap,
            FlexWrap::WrapReverse => t::FlexWrap::WrapReverse,
        },
        align_items: match style.align_items.unwrap_or(AlignItems::Stretch) {
            AlignItems::FlexStart => t::AlignItems::FLEX_START,
            AlignItems::FlexEnd => t::AlignItems::FLEX_END,
            AlignItems::Center => t::AlignItems::CENTER,
            AlignItems::Stretch => t::AlignItems::STRETCH,
            AlignItems::Baseline => t::AlignItems::BASELINE,
        },
        align_self: match style.align_self.unwrap_or(AlignSelf::Auto) {
            AlignSelf::Auto => None,
            AlignSelf::FlexStart => Some(t::AlignSelf::FLEX_START),
            AlignSelf::FlexEnd => Some(t::AlignSelf::FLEX_END),
            AlignSelf::Center => Some(t::AlignSelf::CENTER),
            AlignSelf::Stretch => Some(t::AlignSelf::STRETCH),
            AlignSelf::Baseline => Some(t::AlignSelf::BASELINE),
        },
        align_content: match style.align_content.unwrap_or(AlignContent::FlexStart) {
            AlignContent::FlexStart => t::AlignContent::FLEX_START,
            AlignContent::FlexEnd => t::AlignContent::FLEX_END,
            AlignContent::Center => t::AlignContent::CENTER,
            AlignContent::Stretch => t::AlignContent::STRETCH,
            AlignContent::SpaceBetween => t::AlignContent::SPACE_BETWEEN,
            AlignContent::SpaceAround => t::AlignContent::SPACE_AROUND,
            AlignContent::SpaceEvenly => t::AlignContent::SPACE_EVENLY,
        },
        justify_content: match style.justify_content.unwrap_or(JustifyContent::FlexStart) {
            JustifyContent::FlexStart => t::JustifyContent::FLEX_START,
            JustifyContent::FlexEnd => t::JustifyContent::FLEX_END,
            JustifyContent::Center => t::JustifyContent::CENTER,
            JustifyContent::SpaceBetween => t::JustifyContent::SPACE_BETWEEN,
            JustifyContent::SpaceAround => t::JustifyContent::SPACE_AROUND,
            JustifyContent::SpaceEvenly => t::JustifyContent::SPACE_EVENLY,
        },
        gap: Size {
            width: t::LengthPercentage::length(style.column_gap.or(style.gap).unwrap_or(0).into()),
            height: t::LengthPercentage::length(style.row_gap.or(style.gap).unwrap_or(0).into()),
        },
        // Ink styles.ts does not send overflow to Yoga; it belongs to paint.
        ..t::Style::default()
    }
}
