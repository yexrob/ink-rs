use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use super::Dirty;

thread_local! {
    static REGISTRY: RefCell<HashMap<u64, Weak<Store>>> = RefCell::new(HashMap::new());
    static RENDER_DEPTH: Cell<usize> = const { Cell::new(0) };
}
static NEXT_RUNTIME: AtomicU64 = AtomicU64::new(1);

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
    pub(super) id: u64,
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
    pub(super) fn new() -> Rc<Self> {
        let store = Rc::new(Self {
            id: NEXT_RUNTIME.fetch_add(1, Ordering::Relaxed),
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
/// A handle is `Copy` even when its value is not.
pub struct State<T> {
    runtime: u64,
    pub(super) slot: usize,
    generation: u64,
    marker: PhantomData<Rc<T>>,
}

impl<T> Copy for State<T> {}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> State<T> {
    fn resolve(self) -> Option<Rc<StateValue>> {
        let store = REGISTRY
            .with(|registry| registry.borrow().get(&self.runtime).and_then(Weak::upgrade))?;
        let slots = store.slots.borrow();
        let slot = slots.get(self.slot)?;
        (slot.generation == self.generation)
            .then(|| slot.value.clone())
            .flatten()
    }

    /// Reads the current value. Panics after the owner unmounts.
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
    pub fn set(self, next: T)
    where
        T: PartialEq,
    {
        Self::assert_not_rendering();
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
        drop(previous);
    }

    /// Mutates the value and schedules its owner, without an equality check.
    pub fn update(self, f: impl FnOnce(&mut T)) {
        Self::assert_not_rendering();
        let Some(value) = self.resolve() else { return };
        if !value.dirty.mounted.get() {
            return;
        }
        value.dirty.mark();
        f(value
            .value
            .borrow_mut()
            .downcast_mut::<T>()
            .expect("state type"));
    }

    fn assert_not_rendering() {
        debug_assert!(
            RENDER_DEPTH.with(|depth| depth.get() == 0),
            "state writes during render are forbidden"
        );
    }
}
