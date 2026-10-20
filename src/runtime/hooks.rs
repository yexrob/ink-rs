use std::any::{Any, TypeId};
use std::rc::Rc;

use super::Dirty;
use super::state::{ElementMetrics, NodeRef, State, StateSlot, Store};

type Setup = Box<dyn FnOnce() -> Box<dyn Any>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    NodeRef,
    State(TypeId),
    Memo(TypeId, TypeId),
    Effect(TypeId),
    LayoutEffect(TypeId),
}

pub(super) struct Effect {
    deps: Box<dyn Any>,
    pub(super) setup: Option<Setup>,
    pub(super) cleanup: Option<Box<dyn Any>>,
}

pub(super) enum Value {
    NodeRef {
        handle: NodeRef,
        slot: StateSlot,
    },
    State {
        handle: Box<dyn Any>,
        slot: StateSlot,
    },
    Memo(Box<dyn Any>),
    Effect(Effect),
}

pub(super) struct Hook {
    pub(super) kind: Kind,
    pub(super) value: Value,
}

/// Hook access for one component render. Hooks must retain their order.
pub struct Scope<'a> {
    pub(super) hooks: &'a mut Vec<Hook>,
    pub(super) store: &'a Rc<Store>,
    pub(super) dirty: Rc<Dirty>,
    pub(super) name: &'static str,
    pub(super) cursor: usize,
}

impl Scope<'_> {
    /// Creates a stable ref to bind with [`crate::View::node_ref`].
    /// Its metrics remain zero until the bound View commits layout.
    pub fn use_node_ref(&mut self) -> NodeRef {
        let position = self.position(Kind::NodeRef);
        if position == self.hooks.len() {
            let handle = self
                .store
                .allocate(ElementMetrics::default(), self.dirty.clone());
            self.hooks.push(Hook {
                kind: Kind::NodeRef,
                value: Value::NodeRef {
                    handle: NodeRef(handle),
                    slot: StateSlot::new(self.store.clone(), handle.slot),
                },
            });
        }
        let Value::NodeRef { handle, .. } = &self.hooks[position].value else {
            unreachable!()
        };
        *handle
    }

    fn position(&mut self, kind: Kind) -> usize {
        let position = self.cursor;
        self.cursor += 1;
        if let Some(hook) = self.hooks.get(position) {
            assert!(
                hook.kind == kind,
                "hook order changed in {} at position {position}",
                self.name
            );
        }
        position
    }

    /// Initializes a state slot on mount, then returns the same handle.
    pub fn use_state<T: 'static>(&mut self, init: impl FnOnce() -> T) -> State<T> {
        let kind = Kind::State(TypeId::of::<T>());
        let position = self.position(kind);
        if position == self.hooks.len() {
            let handle = self.store.allocate(init(), self.dirty.clone());
            self.hooks.push(Hook {
                kind,
                value: Value::State {
                    slot: StateSlot::new(self.store.clone(), handle.slot),
                    handle: Box::new(handle),
                },
            });
        }
        let Value::State { handle, .. } = &self.hooks[position].value else {
            unreachable!()
        };
        *handle.downcast_ref::<State<T>>().expect("state hook type")
    }

    /// Recomputes a value when dependencies change.
    pub fn use_memo<D: PartialEq + 'static, T: Clone + 'static>(
        &mut self,
        deps: D,
        f: impl FnOnce() -> T,
    ) -> T {
        let kind = Kind::Memo(TypeId::of::<D>(), TypeId::of::<T>());
        let position = self.position(kind);
        if let Some(Hook {
            value: Value::Memo(value),
            ..
        }) = self.hooks.get(position)
        {
            let (previous, value) = value.downcast_ref::<(D, T)>().expect("memo hook type");
            if *previous == deps {
                return value.clone();
            }
        }
        let value = f();
        let hook = Hook {
            kind,
            value: Value::Memo(Box::new((deps, value.clone()))),
        };
        if position == self.hooks.len() {
            self.hooks.push(hook);
        } else {
            self.hooks[position] = hook;
        }
        value
    }

    /// Runs after the frame is written. The returned value is dropped on
    /// dependency changes and unmount. Use `()` dependencies to run on mount.
    pub fn use_effect<D: PartialEq + 'static, C: 'static>(
        &mut self,
        deps: D,
        f: impl FnOnce() -> C + 'static,
    ) {
        self.effect(Kind::Effect(TypeId::of::<D>()), deps, f);
    }

    /// Runs after layout and before paint. State updates repeat layout.
    /// The returned value is the cleanup, as with [`Scope::use_effect`].
    pub fn use_layout_effect<D: PartialEq + 'static, C: 'static>(
        &mut self,
        deps: D,
        f: impl FnOnce() -> C + 'static,
    ) {
        self.effect(Kind::LayoutEffect(TypeId::of::<D>()), deps, f);
    }

    fn effect<D: PartialEq + 'static, C: 'static>(
        &mut self,
        kind: Kind,
        deps: D,
        f: impl FnOnce() -> C + 'static,
    ) {
        let position = self.position(kind);
        if let Some(Hook {
            value: Value::Effect(effect),
            ..
        }) = self.hooks.get_mut(position)
        {
            if *effect
                .deps
                .downcast_ref::<D>()
                .expect("effect dependencies")
                != deps
            {
                effect.deps = Box::new(deps);
                effect.setup = Some(Box::new(|| Box::new(f())));
            }
        } else {
            self.hooks.push(Hook {
                kind,
                value: Value::Effect(Effect {
                    deps: Box::new(deps),
                    setup: Some(Box::new(|| Box::new(f()))),
                    cleanup: None,
                }),
            });
        }
    }
}
