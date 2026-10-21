use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use super::Dirty;

#[derive(Clone, Default)]
pub(super) struct Contexts(Option<Rc<Link>>);

struct Link {
    kind: TypeId,
    provider: Rc<Provider>,
    parent: Contexts,
}

impl Contexts {
    pub(super) fn push(&mut self, kind: TypeId, provider: Rc<Provider>) {
        self.0 = Some(Rc::new(Link {
            kind,
            provider,
            parent: self.clone(),
        }));
    }

    pub(super) fn find(&self, kind: TypeId) -> Option<Rc<Provider>> {
        let mut current = self.0.as_ref();
        while let Some(link) = current {
            if link.kind == kind {
                return Some(link.provider.clone());
            }
            current = link.parent.0.as_ref();
        }
        None
    }
}

pub(super) struct Provider {
    value: RefCell<Box<dyn Any>>,
    readers: RefCell<Vec<Weak<Dirty>>>,
}

impl Provider {
    pub(super) fn new<T: 'static>(value: T) -> Rc<Self> {
        Rc::new(Self {
            value: RefCell::new(Box::new(value)),
            readers: RefCell::new(Vec::new()),
        })
    }

    pub(super) fn read<T: Clone + 'static>(&self) -> T {
        self.value
            .borrow()
            .downcast_ref::<T>()
            .expect("context hook type")
            .clone()
    }

    pub(super) fn add_reader(&self, dirty: &Rc<Dirty>) {
        let mut readers = self.readers.borrow_mut();
        // Prune before the list grows, so mounts under an unchanging provider
        // keep it bounded by the mounted readers.
        if readers.len() == readers.capacity() {
            readers.retain(|reader| reader.upgrade().is_some_and(|dirty| dirty.mounted.get()));
        }
        readers.push(Rc::downgrade(dirty));
    }

    pub(super) fn update<T: PartialEq + 'static>(&self, value: T) {
        let mut previous = self.value.borrow_mut();
        if previous.downcast_ref::<T>().expect("context hook type") == &value {
            return;
        }
        *previous = Box::new(value);
        self.readers.borrow_mut().retain(|reader| {
            let Some(dirty) = reader.upgrade().filter(|dirty| dirty.mounted.get()) else {
                return false;
            };
            dirty.mark();
            true
        });
    }
}
