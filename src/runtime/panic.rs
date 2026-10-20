use std::backtrace::Backtrace;
use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Mutex, Once};

use crate::error::{PanicError, PanicLocation};

// Keep take_hook/set_hook pairs from losing each other's hook when live and
// testing apps first mount on different threads.
pub(crate) static HOOK_INSTALL: Mutex<()> = Mutex::new(());

thread_local! {
    static DEPTH: Cell<usize> = const { Cell::new(0) };
    static UNWINDING: Cell<bool> = const { Cell::new(false) };
    static CAPTURED: RefCell<Option<(Option<PanicLocation>, Backtrace)>> = const { RefCell::new(None) };
}

pub(crate) fn capturing() -> bool {
    !cfg!(panic = "abort")
        && DEPTH.try_with(|depth| depth.get() > 0).unwrap_or(false)
        && UNWINDING.try_with(|flag| !flag.get()).unwrap_or(false)
}

pub(crate) fn install() {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let _install = HOOK_INSTALL
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if capturing() {
                let _ = UNWINDING.try_with(|flag| flag.set(true));
                let location = info.location().map(|location| PanicLocation {
                    file: location.file().to_owned(),
                    line: location.line(),
                    column: location.column(),
                });
                if CAPTURED
                    .try_with(|captured| {
                        *captured.borrow_mut() = Some((location, Backtrace::capture()));
                    })
                    .is_ok()
                {
                    return;
                }
            }
            // A second hook call before the catch returns is a destructor
            // panic during unwinding. Let the driver restore before std aborts.
            previous(info);
        }));
    });
}

pub(super) fn guard(enabled: bool, f: impl FnOnce()) -> Result<(), PanicError> {
    if !enabled {
        f();
        return Ok(());
    }
    let _ = DEPTH.try_with(|depth| depth.set(depth.get() + 1));
    // Ink abandons the failed subtree. The caller retires it before observing
    // it again, so no half-updated state is reused after AssertUnwindSafe.
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    let captured = CAPTURED
        .try_with(|captured| captured.borrow_mut().take())
        .ok()
        .flatten();
    let _ = UNWINDING.try_with(|flag| flag.set(false));
    let _ = DEPTH.try_with(|depth| depth.set(depth.get() - 1));
    result.map_err(|payload| {
        let (location, backtrace) = captured.unwrap_or_else(|| (None, Backtrace::disabled()));
        PanicError::new(payload, location, backtrace)
    })
}
