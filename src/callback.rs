use std::fmt;
use std::rc::Rc;

/// A cloneable function handle whose equality compares pointer identity.
/// Each [`Callback::new`] creates a distinct identity, even for identical code;
/// passing a new callback as a prop triggers a component render.
/// Use [`crate::Scope::use_callback`] to retain identity across renders.
pub struct Callback<A = (), R = ()>(Rc<dyn Fn(A) -> R>);

impl<A, R> Callback<A, R> {
    /// Wraps a function with a new identity.
    pub fn new(f: impl Fn(A) -> R + 'static) -> Self {
        Self(Rc::new(f))
    }

    /// Calls the function with the given argument.
    pub fn call(&self, arg: A) -> R {
        (self.0)(arg)
    }
}

impl<A, R> Clone for Callback<A, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<A, R> PartialEq for Callback<A, R> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<A, R> fmt::Debug for Callback<A, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callback").finish_non_exhaustive()
    }
}
