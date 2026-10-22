use std::cell::OnceCell;
use std::rc::Rc;

use taffy::{Cache, Layout, NodeId, Size};

use crate::components::TextStyle;
use crate::text::measure::{TextSize, measure_text};
use crate::text::styled::{self, CharStyle, StyledLine, StyledText};
use crate::{ColorLevel, Style, TextWrap};

pub(crate) struct HostTree {
    nodes: Vec<Option<LayoutNode>>,
    free: Vec<usize>,
}

pub(crate) struct LayoutNode {
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub(crate) props: HostNode,
    /// Squashed text and its natural size, kept until the props change.
    squashed: OnceCell<(StyledText, TextSize)>,
    pub(crate) style: taffy::Style,
    pub(crate) cache: Cache,
    pub(crate) layout: Layout,
    pub(crate) hoisted_children: Vec<NodeId>,
    pub(crate) available_inner_size: Size<Option<f32>>,
    pub(crate) definite_inner_size: Size<Option<f32>>,
}

impl LayoutNode {
    /// Squashes a text node on its first measurement or paint. Both run
    /// outside the render guard, so a nested transform's panic stays uncaught
    /// as it does in Ink's output phase.
    pub(crate) fn squashed(&self) -> (&StyledText, TextSize) {
        let (lines, size) = self.squashed.get_or_init(|| {
            let lines = self.props.text.as_ref().expect("text node").squash();
            let size = measure_text(&lines);
            (lines, size)
        });
        (lines, *size)
    }
}

impl HostTree {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free: Vec::new(),
        }
    }

    pub(crate) fn node(&self, node: NodeId) -> &LayoutNode {
        self.nodes[usize::from(node)]
            .as_ref()
            .expect("live host node")
    }

    pub(crate) fn node_mut(&mut self, node: NodeId) -> &mut LayoutNode {
        self.nodes[usize::from(node)]
            .as_mut()
            .expect("live host node")
    }

    pub(crate) fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.node(node).parent
    }

    pub(crate) fn children(&self, node: NodeId) -> &[NodeId] {
        &self.node(node).children
    }

    pub(crate) fn props(&self, node: NodeId) -> &HostNode {
        &self.node(node).props
    }

    pub(crate) fn unrounded_layout(&self, node: NodeId) -> &Layout {
        &self.node(node).layout
    }

    fn mark_dirty(&mut self, node: NodeId) {
        let mut current = Some(node);
        while let Some(id) = current {
            let node = self.node_mut(id);
            // Yoga markDirty propagates measurements affected by a mutation to ancestors.
            node.cache.clear();
            current = node.parent;
        }
    }
}
pub(crate) type TransformFn = Rc<dyn Fn(&str, usize) -> String>;

#[derive(Clone)]
pub(crate) enum LineTransform {
    Style(CharStyle),
    Function(TransformFn),
}

impl PartialEq for LineTransform {
    fn eq(&self, other: &Self) -> bool {
        // Ink dom.ts setTransform invalidates inline transforms on every update.
        matches!((self, other), (Self::Style(a), Self::Style(b)) if a == b)
    }
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

#[derive(Clone, PartialEq)]
pub(crate) enum Inline {
    Content(StyledText),
    Span {
        children: Vec<Inline>,
        transform: LineTransform,
    },
}

#[derive(Clone, PartialEq)]
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
    let slot = tree.free.pop().unwrap_or_else(|| {
        tree.nodes.push(None);
        tree.nodes.len() - 1
    });
    let node = NodeId::from(slot);
    tree.nodes[slot] = Some(LayoutNode {
        parent: None,
        children: Vec::new(),
        style: taffy::Style::default(),
        props: props.clone(),
        squashed: OnceCell::new(),
        cache: Cache::new(),
        layout: Layout::default(),
        hoisted_children: Vec::new(),
        available_inner_size: Size::NONE,
        definite_inner_size: Size::NONE,
    });
    set_props(tree, node, props);
    node
}

/// Replacing inline content or wrapping must invalidate measurements (Ink dom.ts).
pub(crate) fn set_props(tree: &mut HostTree, node: NodeId, props: HostNode) {
    assert!(
        props.text.is_none() || tree.children(node).is_empty(),
        "text is a layout leaf"
    );
    let layout = tree.node_mut(node);
    layout.style = crate::layout::map_style(&props.style);
    layout.props = props;
    layout.squashed = OnceCell::new();
    tree.mark_dirty(node);
}

pub(crate) fn append_child(tree: &mut HostTree, parent: NodeId, child: NodeId) {
    insert_child(tree, parent, child, None);
}

/// React's removeChild frees the removed host subtree; moves only detach it.
pub(crate) fn free_subtree(tree: &mut HostTree, node: NodeId) {
    if let Some(parent) = tree.parent(node) {
        remove_child(tree, parent, node);
    }
    for child in tree.children(node).to_vec() {
        free_subtree(tree, child);
    }
    tree.nodes[usize::from(node)] = None;
    tree.free.push(usize::from(node));
}

/// A detached node stays allocated and may be inserted under another parent.
pub(crate) fn remove_child(tree: &mut HostTree, parent: NodeId, child: NodeId) {
    let index = tree
        .children(parent)
        .iter()
        .position(|&id| id == child)
        .expect("child must belong to parent");
    tree.mark_dirty(child);
    tree.node_mut(parent).children.remove(index);
    tree.node_mut(child).parent = None;
}

pub(crate) fn insert_child(
    tree: &mut HostTree,
    parent: NodeId,
    child: NodeId,
    before: Option<NodeId>,
) {
    assert!(tree.props(parent).text.is_none(), "text is a layout leaf");
    if before == Some(child) && tree.parent(child) == Some(parent) {
        return;
    }
    if let Some(old_parent) = tree.parent(child) {
        remove_child(tree, old_parent, child);
    }
    let children = tree.children(parent);
    let index = before
        .and_then(|before| children.iter().position(|&id| id == before))
        .unwrap_or(children.len());
    tree.node_mut(parent).children.insert(index, child);
    tree.node_mut(child).parent = Some(parent);
    tree.mark_dirty(child);
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
