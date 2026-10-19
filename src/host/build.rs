use taffy::NodeId;

use super::{
    HostTree, Inline, LineTransform, TextNode, append_child, create_text, create_view, text_style,
};
use crate::element::Node;
use crate::text::styled;
use crate::{Color, ColorLevel, Dimension, Display, Element, FlexDirection, Style, TextWrap};

pub(crate) fn build(element: Element, columns: u16, level: ColorLevel) -> (HostTree, NodeId) {
    let mut tree = HostTree::new();
    let root = create_view(
        &mut tree,
        Style {
            width: Dimension::Cells(columns),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            ..Style::default()
        },
        false,
    );
    mount(&mut tree, root, element, None, level);
    (tree, root)
}

fn mount(
    tree: &mut HostTree,
    parent: NodeId,
    element: Element,
    background: Option<Color>,
    level: ColorLevel,
) {
    let node = match element.0 {
        Node::Empty => return,
        Node::Fragment(children) => {
            for child in children {
                mount(tree, parent, child, background, level);
            }
            return;
        }
        Node::View(view) => {
            let background = view.style.background_color.or(background);
            let node = create_view(tree, view.style, false);
            for child in view.children {
                mount(tree, node, child, background, level);
            }
            node
        }
        Node::Static(mut value) => {
            let background = value.style.background_color.or(background);
            if value.len == 0 {
                value.style.display = Display::None;
            }
            let node = create_view(tree, value.style, true);
            for index in 0..value.len {
                mount(tree, node, (value.render)(index), background, level);
            }
            node
        }
        Node::Text(mut text) => {
            if text.children.is_empty() {
                return;
            }
            text.style.background_color = text.style.background_color.or(background);
            create_text(
                tree,
                TextNode {
                    children: inline(text.children, text.style.background_color, level),
                    wrap: text.wrap,
                    transform: LineTransform::Style(text_style(text.style, level)),
                },
            )
        }
        Node::Transform(transform) => {
            if transform.children.is_empty() {
                return;
            }
            create_text(
                tree,
                TextNode {
                    children: inline(transform.children, background, level),
                    wrap: TextWrap::Wrap,
                    transform: LineTransform::Function(transform.transform),
                },
            )
        }
        Node::Str(_) => unreachable!("strings must be children of Text or Transform"),
    };
    append_child(tree, parent, node);
}

fn inline(elements: Vec<Element>, background: Option<Color>, level: ColorLevel) -> Vec<Inline> {
    let mut children = Vec::new();
    for element in elements {
        match element.0 {
            Node::Empty => {}
            Node::Fragment(elements) => children.extend(inline(elements, background, level)),
            Node::Str(value) => children.push(Inline::Content(styled::parse(&value))),
            Node::Text(mut text) => {
                if text.children.is_empty() {
                    continue;
                }
                text.style.background_color = text.style.background_color.or(background);
                children.push(Inline::Span {
                    children: inline(text.children, text.style.background_color, level),
                    transform: LineTransform::Style(text_style(text.style, level)),
                });
            }
            Node::Transform(transform) => children.push(Inline::Span {
                children: inline(transform.children, background, level),
                transform: LineTransform::Function(transform.transform),
            }),
            // Keep the reconciler.ts diagnostic, including Ink's component names.
            Node::View(_) | Node::Static(_) => {
                panic!("<Box> can\u{2019}t be nested inside <Text> component")
            }
        }
    }
    children
}
