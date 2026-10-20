use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::Rc;

use super::input::{Handler, InputHandler, PasteHandler, Registration};
use super::output::{CursorCleanup, CursorPosition, OutputStream, OutputWriter};
use super::state::{ElementMetrics, NodeRef, State, StateSlot, Store};
use super::{AppHandle, Dirty, WindowSize};
use crate::KeyEvent;

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
    /// Handles input while `active`, enabling raw mode through a passive effect.
    /// The handler receives text and key identity/modifiers; text runs and paste
    /// fallback stay one call, while functional keys may carry empty text.
    /// Calls use the handler from the latest render, in registration order.
    /// With [`crate::Options::exit_on_ctrl_c`] enabled, Ctrl+C presses and repeats
    /// exit without reaching the handler; releases reach it.
    pub fn use_input(&mut self, active: bool, handler: impl FnMut(&str, KeyEvent) + 'static) {
        let mut handler = Some(Box::new(handler) as Box<dyn FnMut(&str, KeyEvent)>);
        let current: InputHandler =
            self.use_memo((), || Rc::new(RefCell::new(handler.take().unwrap())));
        // useEffectEvent in Ink keeps the subscription stable across renders.
        if let Some(handler) = handler {
            *current.borrow_mut() = handler;
        }
        let store = self.store.clone();
        self.use_effect(active, move || {
            active.then(|| Registration::new(store, Handler::Input(current)))
        });
    }

    /// Handles pasted text while `active`, enabling raw mode and bracketed paste
    /// mode through a passive effect. Each paste arrives in one call, verbatim.
    /// While any paste hook is active, pastes do not reach [`Self::use_input`];
    /// without one, `use_input` receives the paste as one input.
    /// Calls use the handler from the latest render, in registration order.
    pub fn use_paste(&mut self, active: bool, handler: impl FnMut(&str) + 'static) {
        let mut handler = Some(Box::new(handler) as Box<dyn FnMut(&str)>);
        let current: PasteHandler =
            self.use_memo((), || Rc::new(RefCell::new(handler.take().unwrap())));
        // Ink use-paste.ts uses useEffectEvent to retain the latest handler.
        if let Some(handler) = handler {
            *current.borrow_mut() = handler;
        }
        let store = self.store.clone();
        self.use_effect(active, move || {
            active.then(|| Registration::new(store, Handler::Paste(current)))
        });
    }

    /// Returns a thread-safe handle for submitting updates and requesting exit.
    /// The hook does not retain a handle; the returned handle keeps the loop alive.
    /// Exit requests take effect in the next live app step. They have no effect
    /// in [`crate::testing::render`], which has no live app lifecycle.
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
