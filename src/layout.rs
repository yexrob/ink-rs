use taffy::prelude::TaffyMaxContent;
use taffy::{self as t, AvailableSpace, NodeId, Rect, Size, compute::compute_leaf_layout};
use taffy::{MaybeMath, MaybeResolve, ResolveOrZero};

use crate::host::HostTree;
use crate::style::{Sides, Spacing};
use crate::text::{measure::measure_text, wrap::wrap_text};
use crate::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, Inset,
    JustifyContent, Position, Style, TextWrap,
};

pub(crate) fn compute(tree: &mut HostTree, root: NodeId) {
    // Consumers use get() for Yoga-compatible text rounding.
    t::compute_root_layout(tree, root, Size::MAX_CONTENT);
}

impl t::TraversePartialTree for HostTree {
    type ChildIter<'a> = std::iter::Copied<std::slice::Iter<'a, NodeId>>;

    fn child_ids(&self, node: NodeId) -> Self::ChildIter<'_> {
        self.children(node).iter().copied()
    }

    fn child_count(&self, node: NodeId) -> usize {
        self.children(node).len()
    }

    fn get_child_id(&self, node: NodeId, index: usize) -> NodeId {
        self.children(node)[index]
    }
}

impl t::TraverseTree for HostTree {}

impl t::CacheTree for HostTree {
    fn cache_get(&mut self, node: NodeId, inputs: &t::LayoutInput) -> Option<t::LayoutOutput> {
        self.node_mut(node).cache.get(inputs)
    }

    fn cache_store(&mut self, node: NodeId, inputs: &t::LayoutInput, output: t::LayoutOutput) {
        self.node_mut(node).cache.store(inputs, output);
    }

    fn cache_clear(&mut self, node: NodeId) {
        self.node_mut(node).cache.clear();
    }
}

impl t::LayoutPartialTree for HostTree {
    type CoreContainerStyle<'a> = &'a t::Style;
    type CustomIdent = <t::Style as t::CoreStyle>::CustomIdent;

    fn get_core_container_style(&self, node: NodeId) -> &t::Style {
        &self.node(node).style
    }

    fn set_unrounded_layout(&mut self, node: NodeId, layout: &t::Layout) {
        self.node_mut(node).layout = *layout;
    }

    fn compute_child_layout(
        &mut self,
        node: NodeId,
        mut inputs: t::LayoutInput,
    ) -> t::LayoutOutput {
        if inputs.run_mode == t::RunMode::PerformHiddenLayout {
            return t::compute_hidden_layout(self, node);
        }
        constrain_measurement(self, node, &mut inputs);
        t::compute_cached_layout(self, node, inputs, |tree, node, inputs| {
            record_inner_size(tree, node, inputs);
            let mut output = if tree.node(node).style.display == t::Display::None {
                t::compute_hidden_layout(tree, node)
            } else if !tree.children(node).is_empty() {
                t::compute_flexbox_layout(tree, node, inputs)
            } else {
                compute_leaf_layout(
                    inputs,
                    &tree.node(node).style,
                    |_, _| 0.0,
                    |known, available| {
                        let Some(text) = tree.node(node).props.text.as_ref() else {
                            return Size::ZERO;
                        };
                        let lines = text.squash();
                        let natural = measure_text(&lines);
                        // Ink dom.ts treats both intrinsic requests as Yoga's undefined mode.
                        let width = known.width.or(match available.width {
                            AvailableSpace::Definite(width) => Some(width),
                            _ => None,
                        });
                        let measured = if let Some(width) =
                            width.filter(|&width| (natural.width as f32) > width)
                        {
                            let measured = measure_text(&wrap_text(
                                &lines,
                                width.max(0.0) as usize,
                                text.wrap,
                            ));
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
            };
            if inputs.run_mode == t::RunMode::PerformLayout {
                t::compute_oof_layout(tree, node, &mut output);
            }
            output
        })
    }
}

impl t::LayoutContainingBlock for HostTree {
    type OofItemStyle<'a> = &'a t::Style;

    fn get_oof_item_style(&self, node: NodeId) -> &t::Style {
        &self.node(node).style
    }

    fn clear_hoisted_children(&mut self, node: NodeId) {
        self.node_mut(node).hoisted_children.clear();
    }

    fn add_hoisted_children(&mut self, node: NodeId, hoisted: &[NodeId]) {
        self.node_mut(node)
            .hoisted_children
            .extend_from_slice(hoisted);
    }
}

impl t::LayoutFlexboxContainer for HostTree {
    type FlexboxContainerStyle<'a> = &'a t::Style;
    type FlexboxItemStyle<'a> = t::Style;

    fn get_flexbox_container_style(&self, node: NodeId) -> &t::Style {
        &self.node(node).style
    }

    fn get_flexbox_child_style(&self, node: NodeId) -> t::Style {
        let mut style = self.node(node).style.clone();
        let Some(ratio) = style.aspect_ratio else {
            return style;
        };
        if style.position.is_out_of_flow() {
            return style;
        }
        let Some(parent) = self.parent(node) else {
            return style;
        };
        let parent = self.node(parent);
        let row = matches!(
            parent.style.flex_direction,
            t::FlexDirection::Row | t::FlexDirection::RowReverse
        );
        let main_axis = if row {
            t::AbsoluteAxis::Horizontal
        } else {
            t::AbsoluteAxis::Vertical
        };
        let cross_axis = main_axis.other_axis();
        let size = style
            .size
            .maybe_resolve(parent.definite_inner_size, |_, _| 0.0);
        let alignment = style.align_self.unwrap_or(parent.style.align_items);
        // Yoga computeFlexBasisForChild transfers a definite cross size (including
        // exact stretch) to an automatic main size. At-most space is not exact stretch.
        if size.get_abs(main_axis).is_none() {
            let cross = size.get_abs(cross_axis).or_else(|| {
                (alignment == t::AlignSelf::STRETCH)
                    .then(|| {
                        let margin = style
                            .margin
                            .resolve_or_zero(parent.definite_inner_size.width, |_, _| 0.0);
                        parent
                            .definite_inner_size
                            .get_abs(cross_axis)
                            .map(|cross| (cross - margin.sum_axes().get_abs(cross_axis)).max(0.0))
                    })
                    .flatten()
            });
            if let Some(cross) = cross {
                set_axis(
                    &mut style.size,
                    main_axis,
                    t::Dimension::length(if row { cross * ratio } else { cross / ratio }),
                );
            }
        }
        // Yoga distributeFreeSpaceSecondPass gives the ratio precedence over the
        // cross style and stretch. Keep the node's own container style intact.
        // Taffy resolves this cross size before flexing; the style view cannot
        // transfer a changed main size after grow/shrink (see docs/design.md).
        set_axis(&mut style.size, cross_axis, t::Dimension::auto());
        if alignment == t::AlignSelf::STRETCH {
            style.align_self = Some(t::AlignSelf::FLEX_START);
        }
        style
    }
}

fn set_axis<T>(size: &mut Size<T>, axis: t::AbsoluteAxis, value: T) {
    match axis {
        t::AbsoluteAxis::Horizontal => size.width = value,
        t::AbsoluteAxis::Vertical => size.height = value,
    }
}

fn constrain_measurement(tree: &HostTree, node: NodeId, inputs: &mut t::LayoutInput) {
    let child = tree.node(node);
    if inputs.sizing_mode != t::SizingMode::ContentSize || child.style.position.is_out_of_flow() {
        return;
    }
    let Some(parent) = child.parent.map(|parent| tree.node(parent)) else {
        return;
    };
    let size = child
        .style
        .size
        .maybe_resolve(parent.definite_inner_size, |_, _| 0.0);
    let margin = child
        .style
        .margin
        .resolve_or_zero(parent.available_inner_size.width, |_, _| 0.0)
        .sum_axes();
    // Yoga computeFlexBasisForChild and the post-flex cross measurement use AtMost
    // instead of taffy's max-content request. Rewrite before the cache lookup.
    for axis in [t::AbsoluteAxis::Horizontal, t::AbsoluteAxis::Vertical] {
        if inputs.known_dimensions.get_abs(axis).is_none()
            && size.get_abs(axis).is_none()
            && inputs.available_space.get_abs(axis) == AvailableSpace::MaxContent
            && let Some(available) = parent.available_inner_size.get_abs(axis)
        {
            set_axis(
                &mut inputs.available_space,
                axis,
                AvailableSpace::Definite((available - margin.get_abs(axis)).max(0.0)),
            );
        }
    }
}

fn record_inner_size(tree: &mut HostTree, node: NodeId, inputs: t::LayoutInput) {
    let style = &tree.node(node).style;
    let padding_border = (style
        .padding
        .resolve_or_zero(inputs.parent_size.width, |_, _| 0.0)
        + style
            .border
            .resolve_or_zero(inputs.parent_size.width, |_, _| 0.0))
    .sum_axes();
    let min = style.min_size.maybe_resolve(inputs.parent_size, |_, _| 0.0);
    let max = style.max_size.maybe_resolve(inputs.parent_size, |_, _| 0.0);
    let styled = if inputs.sizing_mode == t::SizingMode::InherentSize {
        style
            .size
            .maybe_resolve(inputs.parent_size, |_, _| 0.0)
            .maybe_apply_aspect_ratio(style.aspect_ratio)
    } else {
        Size::NONE
    };
    let exact = inputs.known_dimensions.or(styled);
    // Yoga calculateAvailableInnerDimension: preserve undefined space, subtract
    // padding/border and clamp to the inner min/max. Each active container owns
    // this context so descendant calls cannot overwrite their parent's constraints.
    let available = exact
        .or(inputs.available_space.into_options())
        .maybe_clamp(min, max)
        .maybe_sub(padding_border)
        .maybe_max(Size::ZERO);
    let definite = inputs
        .known_dimensions
        .zip_map(inputs.known_dimensions_are_definite, |size, definite| {
            size.filter(|_| definite)
        })
        .or(styled)
        .maybe_clamp(min, max)
        .maybe_sub(padding_border)
        .maybe_max(Size::ZERO);
    tree.node_mut(node).available_inner_size = available;
    tree.node_mut(node).definite_inner_size = definite;
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
    let text = tree.props(node).text.is_some();
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
