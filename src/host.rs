use std::rc::Rc;

use taffy::{NodeId, TaffyTree};

use crate::components::TextStyle;
use crate::text::styled::{self, CharStyle, StyledLine, StyledText};
use crate::{ColorLevel, Style, TextWrap};

mod build;
pub(crate) use build::build;

pub(crate) type HostTree = TaffyTree<HostNode>;
pub(crate) type TransformFn = Rc<dyn Fn(&str, usize) -> String>;

#[derive(Clone)]
pub(crate) enum LineTransform {
    Style(CharStyle),
    Function(TransformFn),
}

impl LineTransform {
    pub(crate) fn apply(&self, line: &StyledLine, index: usize) -> StyledText {
        match self {
            Self::Style(style) => {
                let mut line = line.clone();
                for cell in &mut line {
                    cell.style.fill_from(style);
                }
                vec![line]
            }
            Self::Function(f) => {
                let mut lines = styled::parse(&f(&styled::serialize(line), index));
                // An empty result still occupies its source line in squash-text-nodes.ts.
                if lines.is_empty() {
                    lines.push(Vec::new());
                }
                lines
            }
        }
    }
}

#[derive(Clone)]
pub(crate) enum Inline {
    Content(StyledText),
    Span {
        children: Vec<Inline>,
        transform: LineTransform,
    },
}

#[derive(Clone)]
pub(crate) struct TextNode {
    pub(crate) children: Vec<Inline>,
    pub(crate) wrap: TextWrap,
    pub(crate) transform: LineTransform,
}

impl TextNode {
    pub(crate) fn squash(&self) -> StyledText {
        squash(&self.children)
            .iter()
            .map(|line| styled::expand_tabs(line))
            .collect()
    }
}

fn squash(children: &[Inline]) -> StyledText {
    let mut result = Vec::new();
    for child in children {
        let lines = match child {
            Inline::Content(lines) => lines.clone(),
            Inline::Span {
                children,
                transform,
            } => {
                let mut lines = squash(children);
                match transform {
                    LineTransform::Style(style) => {
                        styled::style_span(&mut lines, style);
                        lines
                    }
                    LineTransform::Function(_) => lines
                        .iter()
                        .enumerate()
                        .flat_map(|(index, line)| transform.apply(line, index))
                        .collect(),
                }
            }
        };
        styled::append(&mut result, lines);
    }
    result
}

#[derive(Clone)]
pub(crate) struct HostNode {
    pub(crate) style: Style,
    pub(crate) text: Option<TextNode>,
    pub(crate) is_static: bool,
}

pub(crate) fn create_view(tree: &mut HostTree, style: Style, is_static: bool) -> NodeId {
    create(
        tree,
        HostNode {
            style,
            text: None,
            is_static,
        },
    )
}

pub(crate) fn create_text(tree: &mut HostTree, text: TextNode) -> NodeId {
    create(
        tree,
        HostNode {
            style: Style::default(),
            text: Some(text),
            is_static: false,
        },
    )
}

fn create(tree: &mut HostTree, props: HostNode) -> NodeId {
    let node = tree.new_leaf(taffy::Style::default()).unwrap();
    set_props(tree, node, props);
    node
}

/// Replacing inline content or wrapping must invalidate measurements (Ink dom.ts).
pub(crate) fn set_props(tree: &mut HostTree, node: NodeId, props: HostNode) {
    assert!(
        props.text.is_none() || tree.children(node).unwrap().is_empty(),
        "text is a layout leaf"
    );
    tree.set_style(node, crate::layout::map_style(&props.style))
        .unwrap();
    tree.set_node_context(node, Some(props)).unwrap();
}

pub(crate) fn append_child(tree: &mut HostTree, parent: NodeId, child: NodeId) {
    insert_child(tree, parent, child, None);
}

/// A detached node stays allocated and may be inserted under another parent.
pub(crate) fn remove_child(tree: &mut HostTree, parent: NodeId, child: NodeId) {
    tree.remove_child(parent, child).unwrap();
}

pub(crate) fn insert_child(
    tree: &mut HostTree,
    parent: NodeId,
    child: NodeId,
    before: Option<NodeId>,
) {
    assert!(
        tree.get_node_context(parent).unwrap().text.is_none(),
        "text is a layout leaf"
    );
    if before == Some(child) && tree.parent(child) == Some(parent) {
        return;
    }
    if let Some(old_parent) = tree.parent(child) {
        remove_child(tree, old_parent, child);
    }
    let children = tree.children(parent).unwrap();
    let index = before
        .and_then(|before| children.iter().position(|&id| id == before))
        .unwrap_or(children.len());
    tree.insert_child_at_index(parent, index, child).unwrap();
}

pub(crate) fn text_style(style: TextStyle, level: ColorLevel) -> CharStyle {
    if level == ColorLevel::None {
        return CharStyle::default();
    }
    CharStyle {
        fg: style.color.and_then(|color| color.at_level(level)),
        bg: style
            .background_color
            .and_then(|color| color.at_level(level)),
        bold: style.bold,
        dim: style.dim_color,
        italic: style.italic,
        underline: style.underline,
        inverse: style.inverse,
        strikethrough: style.strikethrough,
        ..CharStyle::default()
    }
}
