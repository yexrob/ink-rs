use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use super::animation::{Animation, AnimationOptions, Subscription, Values};
use super::context::{Contexts, Provider};
use super::focus::{FocusManager, FocusOptions, FocusRegistration};
use super::input::{Handler, InputHandler, PasteHandler, Registration};
use super::output::{CursorCleanup, CursorPosition, OutputStream, OutputWriter};
use super::state::{BoxMetrics, BoxReader, NodeRef, State, StateSlot, Store};
use super::{AppHandle, Dirty, WindowSize};
use crate::{Callback, KeyEvent};

type Setup = Box<dyn FnOnce() -> Box<dyn Any>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    WindowSize,
    Graphics,
    NodeRef,
    State(TypeId),
    Provide(TypeId),
    Context(TypeId),
    Memo(TypeId, TypeId),
    Callback(TypeId),
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
    Graphics,
    NodeRef {
        handle: NodeRef,
        slot: StateSlot,
    },
    State {
        handle: Box<dyn Any>,
        slot: StateSlot,
    },
    Memo(Box<dyn Any>),
    Provide(Rc<Provider>),
    Context(Option<Rc<Provider>>),
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
    pub(super) contexts: &'a Contexts,
    pub(super) name: &'static str,
    pub(super) cursor: usize,
}

impl Scope<'_> {
    /// Registers a focusable hook and returns whether its ID has focus.
    /// Defaults are active, no autofocus, and a generated ID. Layout effects
    /// register then activate the hook, so autofocus appears in the first frame.
    /// Registration order governs navigation, not keyed tree order; changing ID
    /// or autofocus appends a new registration. Duplicate IDs share focus, but
    /// cleanup and activation affect this hook's entry, not its duplicates.
    /// Active hooks hold raw mode on TTY stdin; non-TTY stdin skips that hold.
    /// Tab/Shift+Tab navigate; unmodified Esc clears focus. Releases are ignored,
    /// repeats count as presses, and Ctrl/Meta/Super/Hyper block both keys.
    /// Shift blocks Esc. Both keys reach input handlers after focus changes.
    pub fn use_focus(&mut self, options: FocusOptions) -> bool {
        let manager = self.use_focus_manager();
        let store = self.store.clone();
        let key = self.use_memo((), move || store.focus.borrow_mut().allocate());
        let generated = self.use_memo((), || format!("ink-rs:focus:{key}"));
        let id = options.id.unwrap_or(generated);
        let store = self.store.clone();
        let registration_id = id.clone();
        self.use_layout_effect((id.clone(), options.auto_focus), move || {
            FocusRegistration::new(store, key, registration_id, options.auto_focus)
        });
        let store = self.store.clone();
        // Ink use-focus.ts reapplies activation after every re-registration.
        self.use_layout_effect(
            (options.active, id.clone(), options.auto_focus),
            move || {
                store.update_focus(|focus| focus.activate(key, options.active));
            },
        );
        let store = self.store.clone();
        self.use_effect(options.active, move || {
            options
                .active
                .then(|| Registration::new(store, Handler::Focus(key)))
        });
        manager.active_id().as_ref() == Some(&id)
    }

    /// Returns the runtime's focus manager and subscribes this component to ID
    /// changes. Focus starts enabled; disable clears focus, enable does not
    /// restore it. The handle reads live state and can be captured by effects
    /// and input handlers without cloning.
    pub fn use_focus_manager(&mut self) -> FocusManager {
        let store = self.store.clone();
        let dirty = self.dirty.clone();
        self.use_memo((), move || store.focus.borrow_mut().add_reader(&dirty));
        FocusManager::new(self.store)
    }

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

    /// Probes for kitty graphics with Unicode placeholders on the first call.
    /// Changes to support or cell pixel size schedule this component. The probe
    /// runs once per app; non-interactive, non-TTY, string and debug renders
    /// return Unsupported without querying the terminal.
    pub fn use_graphics(&mut self) -> super::Graphics {
        let index = self.position(Kind::Graphics);
        if index == self.hooks.len() {
            self.hooks.push(Hook {
                kind: Kind::Graphics,
                value: Value::Graphics,
            });
        }
        let mut graphics = self.store.graphics.borrow_mut();
        if graphics.value.support == super::GraphicsSupport::Probing {
            graphics.requested = true;
        }
        graphics.value
    }

    /// Drives an animation using the runtime's shared timer. Values start at zero.
    /// Pausing freezes values; resuming or changing the interval restarts at zero.
    /// [`Animation::reset`] restarts the live hook, even from an earlier copy.
    /// Live apps limit delivered ticks using [`crate::Options::max_fps`]; skipped
    /// ticks contribute to elapsed time and delta. [`crate::testing::render`]
    /// does not throttle.
    /// An active animation keeps a live app running until it pauses or unmounts.
    pub fn use_animation(&mut self, options: AnimationOptions) -> Animation {
        let interval = options.interval.clamp(
            Duration::from_millis(1),
            Duration::from_millis(2_147_483_647),
        );
        let values = self.use_state(Values::default);
        let reset_key = self.use_state(|| 0_u64);
        let store = self.store.clone();
        self.use_layout_effect((options.active, interval, reset_key.get()), move || {
            options.active.then(|| {
                // Unlike Ink's render-time reset, layout writes settle before paint.
                values.set(Values::default());
                Subscription::new(store, interval, values)
            })
        });
        Animation::new(values, reset_key)
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
    /// [`OutputWriter::write_control`] sends raw sequences without redrawing.
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
            let handle = self.store.allocate(None, self.dirty.clone());
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

    /// Tracks a View's size and position in integer cells. Width and height
    /// include borders and padding; client sizes exclude borders but include
    /// padding. Left and top are relative to the parent's layout box and
    /// exclude scroll offsets.
    ///
    /// Returns zeros and `has_measured: false` before attachment and after
    /// detachment. Metrics reach the frame where the View first lays out:
    /// commit checks re-render readers before paint, unlike Ink's passive
    /// `use-box-metrics.ts` effect and layout-listener microtask. An unchanged
    /// layout schedules nothing.
    pub fn use_box_metrics(&mut self, node_ref: NodeRef) -> BoxMetrics {
        let store = self.store.clone();
        let dirty = self.dirty.clone();
        let reader = self.use_memo((), move || {
            let reader = Rc::new(BoxReader {
                node: Cell::new(node_ref),
                seen: Cell::new(BoxMetrics::default()),
                dirty,
            });
            store.box_readers.borrow_mut().push(Rc::downgrade(&reader));
            reader
        });
        reader.node.set(node_ref);
        let metrics = node_ref.box_metrics();
        reader.seen.set(metrics);
        metrics
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

    /// Provides a value to the returned tree, keyed by its type. Use newtypes
    /// for contexts with the same value type. The nearest ancestor wins;
    /// among this component's calls for a type, the last call wins.
    ///
    /// This component's [`Self::use_context`] reads ancestors, not this value.
    /// A changed value schedules mounted readers, including those beneath
    /// components with equal props, for the same commit. Equal values do not.
    pub fn provide<T: Clone + PartialEq + 'static>(&mut self, value: T) {
        let kind = Kind::Provide(TypeId::of::<T>());
        let position = self.position(kind);
        if position == self.hooks.len() {
            self.hooks.push(Hook {
                kind,
                value: Value::Provide(Provider::new(value)),
            });
        } else {
            let Value::Provide(provider) = &self.hooks[position].value else {
                unreachable!()
            };
            provider.update(value);
        }
    }

    /// Clones the nearest ancestor's value of this type, or returns `None`.
    /// Resolves and subscribes on mount; provider changes schedule this
    /// component even when its props are equal. Calls to [`Self::provide`]
    /// in this component do not affect what this hook reads.
    pub fn use_context<T: Clone + 'static>(&mut self) -> Option<T> {
        let kind = Kind::Context(TypeId::of::<T>());
        let position = self.position(kind);
        if position == self.hooks.len() {
            let provider = self.contexts.find(TypeId::of::<T>());
            if let Some(provider) = &provider {
                provider.add_reader(&self.dirty);
            }
            self.hooks.push(Hook {
                kind,
                value: Value::Context(provider),
            });
        }
        let Value::Context(provider) = &self.hooks[position].value else {
            unreachable!()
        };
        provider.as_ref().map(|provider| provider.read())
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

    /// Returns a pointer-equal callback across renders, calling the closure
    /// from the latest render. Like Dioxus's `use_callback`, this has no
    /// dependencies, unlike React's `useCallback`.
    /// After unmount, the handle calls the last closure; writes to state owned
    /// by the unmounted component are ignored.
    pub fn use_callback<A: 'static, R: 'static>(
        &mut self,
        f: impl Fn(A) -> R + 'static,
    ) -> Callback<A, R> {
        let kind = Kind::Callback(TypeId::of::<fn(A) -> R>());
        let position = self.position(kind);
        let next = Callback::new(f);
        if position == self.hooks.len() {
            let current = Rc::new(RefCell::new(next));
            let callback = {
                let current = current.clone();
                Callback::new(move |arg| {
                    let f = current.borrow().clone();
                    f.call(arg)
                })
            };
            self.hooks.push(Hook {
                kind,
                value: Value::Memo(Box::new((current, callback.clone()))),
            });
            callback
        } else {
            let Value::Memo(value) = &self.hooks[position].value else {
                unreachable!()
            };
            let (current, callback) = value
                .downcast_ref::<(Rc<RefCell<Callback<A, R>>>, Callback<A, R>)>()
                .expect("callback hook type");
            *current.borrow_mut() = next;
            callback.clone()
        }
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

    /// Spawns `f()` with `tokio::task::spawn_local` after the commit, like
    /// [`Self::use_effect`]. Changing dependencies or unmounting aborts the task.
    ///
    /// A running task keeps the app alive: `wait_until_exit` cannot reach its
    /// natural end while it runs. A bare `spawn_local` holds nothing; capture
    /// [`Self::use_app`] in it to obtain the same liveness.
    /// A task panic is a component panic: the app displays the error overview
    /// and `wait_until_exit` returns [`crate::Error::Panic`].
    ///
    /// Panics outside a `tokio::task::LocalSet` or local runtime. Tasks need a
    /// driver that yields to the runtime: `ink_rs::tokio`, or
    /// [`crate::testing::mount`] inside a `LocalSet`.
    #[cfg(feature = "tokio")]
    pub fn use_future<D: PartialEq + 'static, F: std::future::Future<Output = ()> + 'static>(
        &mut self,
        deps: D,
        f: impl FnOnce() -> F + 'static,
    ) {
        let store = Rc::downgrade(self.store);
        let queue = self.store.queue.clone();
        self.use_effect(deps, move || {
            let alive = queue.handle();
            super::future::AbortOnDrop(::tokio::task::spawn_local(super::future::Guarded::new(
                store,
                async move {
                    let _alive = alive;
                    f().await;
                },
            )))
        });
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
