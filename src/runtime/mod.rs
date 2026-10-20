use std::any::{Any, TypeId, type_name};
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

use taffy::NodeId;

use crate::element::Node;
use crate::host::{self, HostNode, HostTree, Inline, LineTransform, TextNode};
use crate::text::styled;
use crate::{
    Color, ColorLevel, Dimension, Display, Element, FlexDirection, RenderOptions, Style, TextWrap,
    layout, paint,
};
use hooks::{Hook, Kind, Value};
use state::{RenderGuard, Store};

mod hooks;
mod state;

pub use hooks::Scope;
pub use state::State;

trait RenderComponent {
    fn render(&self, scope: &mut Scope) -> Element;
    fn same_props(&self, other: &dyn RenderComponent) -> bool;
    fn props(&self) -> &dyn Any;
}

struct TypedComponent<F, P> {
    f: F,
    props: P,
}

impl<F, P> RenderComponent for TypedComponent<F, P>
where
    F: Fn(&mut Scope, &P) -> Element + 'static,
    P: PartialEq + 'static,
{
    fn render(&self, scope: &mut Scope) -> Element {
        (self.f)(scope, &self.props)
    }
    fn same_props(&self, other: &dyn RenderComponent) -> bool {
        other.props().downcast_ref::<P>() == Some(&self.props)
    }
    fn props(&self) -> &dyn Any {
        &self.props
    }
}

#[derive(Clone)]
pub(crate) struct Component {
    kind: TypeId,
    name: &'static str,
    render: Rc<dyn RenderComponent>,
}

impl fmt::Debug for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Component").field(&self.name).finish()
    }
}

/// Creates a component element. Pass a function item or non-capturing closure;
/// function pointers and capturing closures cannot provide component identity.
pub fn component<F, P>(f: F, props: P) -> Element
where
    F: Fn(&mut Scope, &P) -> Element + 'static,
    P: PartialEq + 'static,
{
    const {
        assert!(
            std::mem::size_of::<F>() == 0,
            "component requires a zero-sized function item or non-capturing closure; function pointers and capturing closures are not supported"
        );
    }
    Element(
        Node::Component(Component {
            kind: TypeId::of::<F>(),
            name: type_name::<F>(),
            render: Rc::new(TypedComponent { f, props }),
        }),
        None,
    )
}

struct Instance {
    element: Element,
    children: Vec<Instance>,
    host: Option<NodeId>,
    hooks: Vec<Hook>,
    rendered: bool,
    dirty: Rc<Cell<bool>>,
    background: Option<Color>,
    store: Rc<Store>,
}

impl Instance {
    fn new(element: Element, store: &Rc<Store>) -> Self {
        Self {
            element,
            children: Vec::new(),
            host: None,
            hooks: Vec::new(),
            rendered: false,
            dirty: Rc::new(Cell::new(false)),
            background: None,
            store: store.clone(),
        }
    }

    fn matches(&self, next: &Element) -> bool {
        self.element.1 == next.1
            && match (&self.element.0, &next.0) {
                (Node::Component(a), Node::Component(b)) => a.kind == b.kind,
                (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
            }
    }

    fn has_dirty(&self) -> bool {
        self.dirty.get() || self.children.iter().any(Self::has_dirty)
    }

    fn retire(&mut self, cleanups: &mut Vec<Box<dyn Any>>) {
        // React commitDeletionEffects traverses parent before child on unmount.
        let mut states: Vec<Box<dyn Any>> = Vec::new();
        for hook in self.hooks.drain(..) {
            match hook.value {
                Value::State { slot, .. } => states.push(Box::new(slot)),
                Value::Effect(mut effect) => {
                    if let Some(cleanup) = effect.cleanup.take() {
                        cleanups.push(cleanup);
                    }
                }
                Value::Memo(value) => cleanups.push(value),
            }
        }
        for child in &mut self.children {
            child.retire(cleanups);
        }
        // React detaches hook state after unmount effects: cleanup can read its owner.
        cleanups.extend(states);
    }

    fn collect_cleanups(&mut self, cleanups: &mut Vec<Box<dyn Any>>) {
        for hook in &mut self.hooks {
            if let Value::Effect(effect) = &mut hook.value
                && effect.setup.is_some()
                && let Some(cleanup) = effect.cleanup.take()
            {
                cleanups.push(cleanup);
            }
        }
        for child in &mut self.children {
            child.collect_cleanups(cleanups);
        }
    }

    fn effects(&mut self, layout: bool) {
        // React recursivelyTraverseLayoutEffects/PassiveMountEffects visit children first.
        for child in &mut self.children {
            child.effects(layout);
        }
        for hook in &mut self.hooks {
            if matches!(hook.kind, Kind::LayoutEffect(_)) == layout
                && let Value::Effect(effect) = &mut hook.value
                && let Some(setup) = effect.setup.take()
            {
                effect.cleanup = Some(setup());
            }
        }
    }

    fn host_roots(&self, nodes: &mut Vec<NodeId>) {
        if let Some(host) = self.host {
            nodes.push(host);
        } else {
            for child in &self.children {
                child.host_roots(nodes);
            }
        }
    }
}

pub(crate) struct Runtime {
    tree: HostTree,
    root: NodeId,
    instance: Instance,
    cleanups: Vec<Box<dyn Any>>,
    options: RenderOptions,
}

impl Runtime {
    pub(crate) fn new(element: Element, options: RenderOptions) -> Self {
        let mut tree = HostTree::new();
        let root = host::create_view(
            &mut tree,
            Style {
                width: Dimension::Cells(options.columns),
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0,
                ..Style::default()
            },
            false,
        );
        let store = Store::new();
        Self {
            tree,
            root,
            instance: Instance::new(Element::fragment([element]), &store),
            cleanups: Vec::new(),
            options,
        }
    }

    /// The caller consumes the painted frame before passive effects run.
    pub(crate) fn frame(&mut self, write: impl FnOnce(paint::Frame)) {
        for round in 0..50 {
            {
                let _guard = RenderGuard::enter();
                reconcile(
                    &mut self.instance,
                    None,
                    false,
                    None,
                    &mut self.tree,
                    self.options.color_level,
                    &mut self.cleanups,
                );
                order_children(&mut self.tree, self.root, &self.instance.children);
            }
            // React's mutation unmount phase precedes layout/passive setup.
            self.instance.collect_cleanups(&mut self.cleanups);
            for cleanup in self.cleanups.drain(..) {
                drop(cleanup);
            }
            {
                let _guard = RenderGuard::enter();
                layout::compute(&mut self.tree, self.root);
            }
            self.instance.effects(true);
            if !self.instance.has_dirty() {
                break;
            }
            assert!(
                round < 49,
                "maximum update depth exceeded (50 layout-effect rounds)"
            );
        }
        let frame = {
            let _guard = RenderGuard::enter();
            paint::paint(&self.tree, self.root, self.options.color_level)
        };
        write(frame);
        self.instance.effects(false);
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.instance.retire(&mut self.cleanups);
        for cleanup in self.cleanups.drain(..) {
            drop(cleanup);
        }
    }
}

fn remove(instance: &mut Instance, tree: &mut HostTree, cleanups: &mut Vec<Box<dyn Any>>) {
    instance.retire(cleanups);
    let mut roots = Vec::new();
    instance.host_roots(&mut roots);
    for root in roots {
        host::free_subtree(tree, root);
    }
}

fn reconcile_children(
    instance: &mut Instance,
    next: Option<Vec<Element>>,
    inside_text: bool,
    background: Option<Color>,
    tree: &mut HostTree,
    level: ColorLevel,
    cleanups: &mut Vec<Box<dyn Any>>,
) -> bool {
    let mut changed = false;
    if let Some(next) = next {
        // ReactChildFiber updateFromMap uses keys or original sibling indices.
        let mut positions: Vec<_> = (0..instance.children.len()).map(Some).collect();
        let length = next.len();
        for (index, element) in next.into_iter().enumerate() {
            let found = if element.1.is_some() {
                instance.children[index..]
                    .iter()
                    .position(|child| child.element.1 == element.1)
                    .map(|offset| index + offset)
            } else {
                positions
                    .iter()
                    .position(|position| *position == Some(index))
                    .filter(|&position| instance.children[position].element.1.is_none())
            };
            let mut child = found.map(|position| {
                positions.remove(position);
                instance.children.remove(position)
            });
            if child.as_ref().is_some_and(|child| !child.matches(&element)) {
                remove(child.as_mut().unwrap(), tree, cleanups);
                child = None;
            }
            let child = child.unwrap_or_else(|| Instance::new(element.clone(), &instance.store));
            // Retain ownership in the tree before user render code can unwind.
            instance.children.insert(index, child);
            positions.insert(index, None);
            changed |= reconcile(
                &mut instance.children[index],
                Some(element),
                inside_text,
                background,
                tree,
                level,
                cleanups,
            );
        }
        for child in &mut instance.children[length..] {
            remove(child, tree, cleanups);
            changed = true;
        }
        instance.children.truncate(length);
    } else {
        for child in &mut instance.children {
            changed |= reconcile(child, None, inside_text, background, tree, level, cleanups);
        }
    }
    changed
}

fn reconcile(
    instance: &mut Instance,
    next: Option<Element>,
    inside_text: bool,
    background: Option<Color>,
    tree: &mut HostTree,
    level: ColorLevel,
    cleanups: &mut Vec<Box<dyn Any>>,
) -> bool {
    let supplied = next.is_some();
    let context_changed = instance.background != background;
    if instance.rendered && !supplied && !context_changed && !instance.has_dirty() {
        return false;
    }
    let mut render = !instance.rendered || supplied;
    if let (Node::Component(previous), Some(Element(Node::Component(next), _))) =
        (&instance.element.0, &next)
    {
        render = !instance.rendered || !previous.render.same_props(next.render.as_ref());
    }
    if let Some(next) = next {
        instance.element = next;
    }
    if !render && !context_changed && !instance.has_dirty() {
        return false;
    }
    let mut changed = !instance.rendered || supplied || context_changed;
    instance.background = background;
    let node = instance.element.0.clone();
    let child_background = match &node {
        Node::View(view) => view.style.background_color.or(background),
        Node::Static(value) => value.style.background_color.or(background),
        Node::Text(text) => text.style.background_color.or(background),
        _ => background,
    };
    let text = matches!(node, Node::Text(_) | Node::Transform(_));
    let next_children = match &node {
        Node::Component(component) => {
            let dirty = instance.dirty.replace(false);
            if render || dirty {
                let previous_count = instance.hooks.len();
                let mut scope = Scope {
                    hooks: &mut instance.hooks,
                    store: &instance.store,
                    dirty: instance.dirty.clone(),
                    name: component.name,
                    cursor: 0,
                };
                let element = component.render.render(&mut scope);
                debug_assert!(
                    !instance.rendered || scope.cursor == previous_count,
                    "hook order changed in {}",
                    component.name
                );
                changed = true;
                Some(vec![element])
            } else {
                changed = context_changed;
                None
            }
        }
        Node::View(_) | Node::Static(_) if inside_text => {
            panic!("<Box> can\u{2019}t be nested inside <Text> component")
        }
        Node::View(view) if render => Some(view.children.clone()),
        Node::Static(value) if render => {
            Some((0..value.len).map(|index| (value.render)(index)).collect())
        }
        Node::Text(text) if render => Some(text.children.clone()),
        Node::Transform(transform) if render => Some(transform.children.clone()),
        Node::Fragment(children) if render => Some(children.clone()),
        Node::Str(_) if !inside_text => {
            unreachable!("strings must be children of Text or Transform")
        }
        _ => None,
    };
    changed |= reconcile_children(
        instance,
        next_children,
        inside_text || text,
        child_background,
        tree,
        level,
        cleanups,
    );
    match &node {
        Node::View(view) => {
            update_view(instance, view.style.clone(), false, tree);
            order_children(tree, instance.host.unwrap(), &instance.children);
        }
        Node::Static(value) => {
            let mut style = value.style.clone();
            if value.len == 0 {
                style.display = Display::None;
            }
            update_view(instance, style, true, tree);
            order_children(tree, instance.host.unwrap(), &instance.children);
        }
        Node::Text(_) | Node::Transform(_) if !inside_text && changed => {
            if instance.children.is_empty() {
                if let Some(host) = instance.host.take() {
                    host::free_subtree(tree, host);
                }
            } else {
                let text = TextNode {
                    children: inline_children(instance, child_background, level),
                    wrap: match &node {
                        Node::Text(text) => text.wrap,
                        _ => TextWrap::Wrap,
                    },
                    transform: transform(&node, background, level),
                };
                if let Some(host) = instance.host {
                    if tree.props(host).text.as_ref() != Some(&text) {
                        host::set_props(
                            tree,
                            host,
                            HostNode {
                                style: Style::default(),
                                text: Some(text),
                                is_static: false,
                            },
                        );
                    }
                } else {
                    instance.host = Some(host::create_text(tree, text));
                }
            }
        }
        _ => {}
    }
    instance.rendered = true;
    changed
}

fn update_view(instance: &mut Instance, style: Style, is_static: bool, tree: &mut HostTree) {
    if let Some(node) = instance.host {
        if tree.props(node).style != style {
            host::set_props(
                tree,
                node,
                HostNode {
                    style,
                    text: None,
                    is_static,
                },
            );
        }
    } else {
        instance.host = Some(host::create_view(tree, style, is_static));
    }
}

fn order_children(tree: &mut HostTree, parent: NodeId, children: &[Instance]) {
    let mut nodes = Vec::new();
    for child in children {
        child.host_roots(&mut nodes);
    }
    let mut before = None;
    for node in nodes.into_iter().rev() {
        let current = tree.children(parent);
        let index = current.iter().position(|&child| child == node);
        if index.is_none_or(|index| current.get(index + 1).copied() != before) {
            if before.is_none() {
                host::append_child(tree, parent, node);
            } else {
                host::insert_child(tree, parent, node, before);
            }
        }
        before = Some(node);
    }
}

fn transform(node: &Node, background: Option<Color>, level: ColorLevel) -> LineTransform {
    match node {
        Node::Text(text) => {
            let mut style = text.style;
            style.background_color = style.background_color.or(background);
            LineTransform::Style(host::text_style(style, level))
        }
        Node::Transform(transform) => LineTransform::Function(transform.transform.clone()),
        _ => unreachable!(),
    }
}

fn inline_children(
    instance: &Instance,
    background: Option<Color>,
    level: ColorLevel,
) -> Vec<Inline> {
    let mut result = Vec::new();
    for child in &instance.children {
        match &child.element.0 {
            Node::Str(value) => result.push(Inline::Content(styled::parse(value))),
            Node::Text(text) => {
                if !child.children.is_empty() {
                    result.push(Inline::Span {
                        children: inline_children(
                            child,
                            text.style.background_color.or(background),
                            level,
                        ),
                        transform: transform(&child.element.0, background, level),
                    });
                }
            }
            Node::Transform(_) => result.push(Inline::Span {
                children: inline_children(child, background, level),
                transform: transform(&child.element.0, background, level),
            }),
            _ => result.extend(inline_children(child, background, level)),
        }
    }
    result
}
