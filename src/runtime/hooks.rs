use std::any::{Any, TypeId};
use std::rc::Rc;

use super::output::{CursorCleanup, CursorPosition, OutputStream, OutputWriter};
use super::state::{ElementMetrics, NodeRef, State, StateSlot, Store};
use super::{AppHandle, Dirty, WindowSize};

type Setup = Box<dyn FnOnce() -> Box<dyn Any>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    WindowSize,
    NodeRef,
    State(TypeId),
    Memo(TypeId, TypeId),
    Effect(TypeId),
    LayoutEffect(TypeId),
    InsertionEffect,
}

pub(super) struct Effect {
    deps: Box<dyn Any>,
    pub(super) setup: Option<Setup>,
    pub(super) cleanup: Option<Box<dyn Any>>,
}

pub(super) enum Value {
    WindowSize,
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
    /// Returns a thread-safe handle for submitting updates to this app.
    /// The hook does not retain a handle; the returned handle keeps the loop alive.
    pub fn use_app(&mut self) -> AppHandle {
        self.store.queue.handle()
    }

    /// Returns the current window dimensions in cells. Changes to either
    /// dimension schedule this component. String and debug renders use
    /// [`crate::RenderOptions::columns`] and 24 rows.
    pub fn use_window_size(&mut self) -> WindowSize {
        let index = self.position(Kind::WindowSize);
        if index == self.hooks.len() {
            self.hooks.push(Hook {
                kind: Kind::WindowSize,
                value: Value::WindowSize,
            });
        }
        // Ink use-window-size.ts rechecks at subscription. Reading during
        // render registers the reader before any commit effects can resize.
        self.store.window_size.get()
    }

    /// Declares the terminal cursor in cells relative to the live frame, below
    /// Static output. `None` hides it. Each render commits this declaration;
    /// a component that skips rendering retains its previous declaration.
    /// Unmount clears the shared cursor. Debug and string rendering ignore it.
    pub fn use_cursor(&mut self, position: Option<CursorPosition>) {
        let index = self.position(Kind::InsertionEffect);
        let store = self.store.clone();
        let setup: Setup = Box::new(move || {
            store.cursor.set(position);
            Box::new(CursorCleanup(store))
        });
        // Ink useCursor uses an insertion effect without dependencies.
        if let Some(Hook {
            value: Value::Effect(effect),
            ..
        }) = self.hooks.get_mut(index)
        {
            effect.setup = Some(setup);
        } else {
            self.hooks.push(Hook {
                kind: Kind::InsertionEffect,
                value: Value::Effect(Effect {
                    deps: Box::new(()),
                    setup: Some(setup),
                    cleanup: None,
                }),
            });
        }
    }

    /// Returns a writer for stdout. Writes preserve the last painted frame and
    /// do not wait for the frame throttle. No stream object is exposed.
    pub fn use_stdout(&mut self) -> OutputWriter {
        OutputWriter::new(self.store, OutputStream::Stdout)
    }

    /// Returns a writer for stderr, with the same timing as [`Scope::use_stdout`].
    pub fn use_stderr(&mut self) -> OutputWriter {
        OutputWriter::new(self.store, OutputStream::Stderr)
    }

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

    /// Runs after the commit, and after the frame is written when that frame
    /// is not throttled. The returned value is dropped on
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
