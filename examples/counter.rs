use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use ink_rs::{Color, Element, Options, Scope, Text, component, render, rsx};

struct Cancel(Arc<AtomicBool>);

impl Drop for Cancel {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

// Ink examples/counter: the interval is a worker holding an AppHandle.
fn counter(cx: &mut Scope, _: &()) -> Element {
    let count = cx.use_state(|| 0_u64);
    let handle = cx.use_app();
    cx.use_effect((), move || {
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100));
                if worker_cancelled.load(Ordering::Relaxed) {
                    break;
                }
                handle.update(move || count.update(|n| *n += 1));
            }
        });
        Cancel(cancelled)
    });
    let count = count.get();
    rsx! {
        Text(color: Color::Green) { "{count} tests passed" }
    }
    .into()
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(counter, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
