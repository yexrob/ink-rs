use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, ThreadId};

use super::app_handle::UpdateQueue;
use super::output::{CursorDeclaration, WriteDestination};
use super::{Dirty, WindowSize};
use crate::ColorLevel;

/// Committed layout in integer cells, relative to the live tree's root.
/// Coordinates exclude scrolling offsets and terminal viewport placement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ElementMetrics {
    /// Zero-based column, including ancestor layout offsets.
    pub x: i32,
    /// Zero-based row, including ancestor layout offsets.
    pub y: i32,
    /// Width including borders and padding.
    pub width: u32,
    /// Height including borders and padding.
    pub height: u32,
    /// Width excluding borders, but including padding.
    pub client_width: u32,
    /// Height excluding borders, but including padding.
    pub client_height: u32,
}

/// A View's committed layout in integer cells, relative to its parent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoxMetrics {
    /// Width including borders and padding.
    pub width: u32,
    /// Height including borders and padding.
    pub height: u32,
    /// Column relative to the parent's layout box, excluding scroll offsets.
    pub left: i32,
    /// Row relative to the parent's layout box, excluding scroll offsets.
    pub top: i32,
    /// Width excluding borders, but including padding.
    pub client_width: u32,
    /// Height excluding borders, but including padding.
    pub client_height: u32,
    /// Whether the tracked View is attached and has committed layout.
    pub has_measured: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct NodeLayout {
    pub(crate) metrics: ElementMetrics,
    pub(crate) paint_origin: taffy::Point<i32>,
    pub(crate) left: i32,
    pub(crate) top: i32,
}

/// A UI-thread handle to a View's committed layout, created by
/// [`super::Scope::use_node_ref`]. Equality identifies the owning hook slot.
#[derive(Clone, Copy)]
pub struct NodeRef(pub(super) State<Option<NodeLayout>>);

impl fmt::Debug for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeRef")
            .field("runtime", &self.0.runtime)
            .field("slot", &self.0.slot)
            .field("generation", &self.0.generation)
            .finish()
    }
}

impl PartialEq for NodeRef {
    fn eq(&self, other: &Self) -> bool {
        (self.0.runtime, self.0.slot, self.0.generation)
            == (other.0.runtime, other.0.slot, other.0.generation)
    }
}

impl Eq for NodeRef {}

impl NodeRef {
    pub(crate) fn refresh(self, layout: Option<NodeLayout>) -> bool {
        let Some(value) = self.0.resolve().filter(|value| value.dirty.mounted.get()) else {
            return false;
        };
        // React ref attachment does not schedule an update, unlike state writes.
        *value
            .value
            .borrow_mut()
            .downcast_mut::<Option<NodeLayout>>()
            .expect("node ref type") = layout;
        true
    }

    pub(super) fn layout(self) -> Option<NodeLayout> {
        let value = self.0.resolve().filter(|value| value.dirty.mounted.get())?;
        *value
            .value
            .borrow()
            .downcast_ref::<Option<NodeLayout>>()
            .expect("node ref type")
    }

    pub(super) fn box_metrics(self) -> BoxMetrics {
        let Some(layout) = self.layout() else {
            return BoxMetrics::default();
        };
        BoxMetrics {
            width: layout.metrics.width,
            height: layout.metrics.height,
            left: layout.left,
            top: layout.top,
            client_width: layout.metrics.client_width,
            client_height: layout.metrics.client_height,
            has_measured: true,
        }
    }
}

/// Reads the latest committed layout, including during render. Before attachment,
/// after detachment, or after the owning component unmounts, returns zeros.
/// Layout effects see the layout of their commit; reads do not schedule work.
pub fn measure_element(node_ref: NodeRef) -> ElementMetrics {
    node_ref
        .layout()
        .map_or_else(ElementMetrics::default, |layout| layout.metrics)
}

pub(super) struct BoxReader {
    pub(super) node: Cell<NodeRef>,
    pub(super) seen: Cell<BoxMetrics>,
    pub(super) dirty: Rc<Dirty>,
}

thread_local! {
    static REGISTRY: RefCell<HashMap<u64, Weak<Store>>> = RefCell::new(HashMap::new());
    static RENDER_DEPTH: Cell<usize> = const { Cell::new(0) };
}
static NEXT_RUNTIME: AtomicU64 = AtomicU64::new(1);

pub(super) fn resolve_store(id: u64) -> Option<Rc<Store>> {
    REGISTRY.with(|registry| registry.borrow().get(&id).and_then(Weak::upgrade))
}

#[track_caller]
pub(super) fn assert_not_rendering() {
    debug_assert!(
        RENDER_DEPTH.with(|depth| depth.get() == 0),
        "state and output writes during render are forbidden"
    );
}

pub(super) struct RenderGuard;

impl RenderGuard {
    pub(super) fn enter() -> Self {
        RENDER_DEPTH.with(|depth| depth.set(depth.get() + 1));
        Self
    }
}

impl Drop for RenderGuard {
    fn drop(&mut self) {
        RENDER_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

struct Slot {
    generation: u64,
    value: Option<Rc<StateValue>>,
}

struct StateValue {
    value: RefCell<Box<dyn Any>>,
    dirty: Rc<Dirty>,
}

pub(super) struct Store {
    // Tasks poll outside Runtime's guards; share its first-panic slot without
    // retaining the runtime through a task's lifetime.
    pub(super) panic: RefCell<Option<crate::PanicError>>,
    pub(super) focus: RefCell<super::focus::FocusState>,
    pub(super) box_readers: RefCell<Vec<Weak<BoxReader>>>,
    pub(super) input: RefCell<super::input::InputState>,
    pub(crate) queue: Arc<UpdateQueue>,
    pub(super) window_size: Cell<WindowSize>,
    pub(super) color_level: ColorLevel,
    pub(super) graphics: RefCell<super::graphics::GraphicsState>,
    pub(super) id: u64,
    pub(super) cursor: Cell<Option<CursorDeclaration>>,
    pub(super) animation: RefCell<super::animation::AnimationState>,
    pub(super) destination: RefCell<Option<Rc<dyn WriteDestination>>>,
    pub(super) unmounting: Cell<bool>,
    slots: RefCell<Vec<Slot>>,
    free: RefCell<Vec<usize>>,
}

pub(super) struct StateSlot {
    store: Rc<Store>,
    slot: usize,
}

impl StateSlot {
    pub(super) fn new(store: Rc<Store>, slot: usize) -> Self {
        Self { store, slot }
    }
}

impl Drop for StateSlot {
    fn drop(&mut self) {
        drop(self.store.release(self.slot));
    }
}

impl Store {
    pub(super) fn new(color_level: ColorLevel) -> Rc<Self> {
        let store = Rc::new(Self {
            panic: RefCell::new(None),
            focus: RefCell::default(),
            box_readers: RefCell::default(),
            input: RefCell::default(),
            graphics: RefCell::default(),
            queue: Arc::default(),
            window_size: Cell::new(WindowSize {
                columns: 80,
                rows: 24,
            }),
            color_level,
            id: NEXT_RUNTIME.fetch_add(1, Ordering::Relaxed),
            cursor: Cell::new(None),
            animation: RefCell::default(),
            destination: RefCell::new(None),
            unmounting: Cell::new(false),
            slots: RefCell::new(Vec::new()),
            free: RefCell::new(Vec::new()),
        });
        REGISTRY.with(|registry| {
            registry
                .borrow_mut()
                .insert(store.id, Rc::downgrade(&store))
        });
        store
    }

    pub(super) fn check_box_readers(&self) {
        self.box_readers.borrow_mut().retain(|reader| {
            let Some(reader) = reader.upgrade().filter(|reader| reader.dirty.mounted.get()) else {
                return false;
            };
            if reader.node.get().box_metrics() != reader.seen.get() {
                reader.dirty.mark();
            }
            true
        });
    }

    pub(super) fn allocate<T: 'static>(&self, value: T, dirty: Rc<Dirty>) -> State<T> {
        let value = Some(Rc::new(StateValue {
            value: RefCell::new(Box::new(value)),
            dirty,
        }));
        let mut slots = self.slots.borrow_mut();
        let slot = if let Some(slot) = self.free.borrow_mut().pop() {
            slots[slot].value = value;
            slot
        } else {
            slots.push(Slot {
                generation: 0,
                value,
            });
            slots.len() - 1
        };
        State {
            thread: thread::current().id(),
            runtime: self.id,
            slot,
            generation: slots[slot].generation,
            marker: PhantomData,
        }
    }

    fn release(&self, slot: usize) -> Rc<StateValue> {
        let value = {
            let mut slots = self.slots.borrow_mut();
            let entry = &mut slots[slot];
            entry.generation += 1;
            entry.value.take().expect("mounted state")
        };
        self.free.borrow_mut().push(slot);
        // Drop user values after releasing the arena borrow, like effect cleanups.
        value
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        REGISTRY.with(|registry| registry.borrow_mut().remove(&self.id));
    }
}

/// A UI-thread state handle. Writes after unmount are ignored; reads panic.
/// A handle is `Copy + Send + Sync` even when its value is not. Access from
/// any thread other than the owning UI thread panics.
pub struct State<T> {
    thread: ThreadId,
    runtime: u64,
    pub(super) slot: usize,
    generation: u64,
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for State<T> {}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> State<T> {
    #[track_caller]
    fn resolve(self) -> Option<Rc<StateValue>> {
        assert_eq!(
            self.thread,
            thread::current().id(),
            "state access off the UI thread"
        );
        let store = resolve_store(self.runtime)?;
        let slots = store.slots.borrow();
        let slot = slots.get(self.slot)?;
        (slot.generation == self.generation)
            .then(|| slot.value.clone())
            .flatten()
    }

    /// Reads the current value. Panics after the owner unmounts.
    #[track_caller]
    pub fn get(self) -> T
    where
        T: Clone,
    {
        let value = self
            .resolve()
            .expect("reading state after its component unmounted");
        let value = value.value.borrow();
        value.downcast_ref::<T>().expect("state type").clone()
    }

    /// Replaces the value, scheduling its owner when the value differs.
    #[track_caller]
    pub fn set(self, next: T)
    where
        T: PartialEq,
    {
        assert_not_rendering();
        let Some(value) = self.resolve() else { return };
        // The slot outlives unmount until passive cleanups have run; React
        // ignores updates to unmounted components.
        if !value.dirty.mounted.get() {
            return;
        }
        let previous = {
            let mut current = value.value.borrow_mut();
            if *current.downcast_ref::<T>().expect("state type") == next {
                return;
            }
            std::mem::replace(&mut *current, Box::new(next))
        };
        value.dirty.mark();
        if let Some(store) = resolve_store(self.runtime) {
            store.queue.notify();
        }
        drop(previous);
    }

    /// Mutates the value and schedules its owner, without an equality check.
    #[track_caller]
    pub fn update(self, f: impl FnOnce(&mut T)) {
        assert_not_rendering();
        let Some(value) = self.resolve() else { return };
        if !value.dirty.mounted.get() {
            return;
        }
        value.dirty.mark();
        if let Some(store) = resolve_store(self.runtime) {
            store.queue.notify();
        }
        f(value
            .value
            .borrow_mut()
            .downcast_mut::<T>()
            .expect("state type"));
    }
}
