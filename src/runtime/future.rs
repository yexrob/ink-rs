use std::future::Future;
use std::pin::Pin;
use std::rc::Weak;
use std::task::{Context, Poll};

use super::state::Store;

pub(super) struct AbortOnDrop(pub(super) ::tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(super) struct Guarded<F> {
    store: Weak<Store>,
    future: Pin<Box<F>>,
}

impl<F> Guarded<F> {
    pub(super) fn new(store: Weak<Store>, future: F) -> Self {
        Self {
            store,
            future: Box::pin(future),
        }
    }
}

impl<F: Future<Output = ()>> Future for Guarded<F> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut result = Poll::Pending;
        // Capture before Tokio can swallow the panic or the process hook can
        // restore a terminal whose app is still running.
        if let Err(error) = super::panic::guard(true, || {
            result = self.future.as_mut().poll(cx);
        }) {
            if let Some(store) = self.store.upgrade() {
                let mut panic = store.panic.borrow_mut();
                if panic.is_none() {
                    *panic = Some(error);
                }
                drop(panic);
                store.queue.notify();
            }
            return Poll::Ready(());
        }
        result
    }
}
