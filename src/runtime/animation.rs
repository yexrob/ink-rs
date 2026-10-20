use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::Runtime;
use super::state::{State, Store};

/// Timing and activity for [`super::Scope::use_animation`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationOptions {
    /// Tick interval, clamped to 1..=2_147_483_647 milliseconds. Defaults to 100 ms.
    pub interval: Duration,
    /// Whether to subscribe to ticks. Defaults to true.
    pub active: bool,
}

impl Default for AnimationOptions {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),
            active: true,
        }
    }
}

/// Animation values from one render, with a handle to reset the live hook.
#[derive(Clone, Copy)]
pub struct Animation {
    /// Elapsed time divided by the interval, rounded down.
    pub frame: u64,
    /// Time since this subscription started.
    pub time: Duration,
    /// Time since the previous delivered tick, or since start for the first tick.
    pub delta: Duration,
    values: State<Values>,
    reset_key: State<u64>,
}

impl Animation {
    pub(super) fn new(values: State<Values>, reset_key: State<u64>) -> Self {
        let current = values.get();
        Self {
            frame: current.frame,
            time: current.time,
            delta: current.delta,
            values,
            reset_key,
        }
    }

    /// Zeroes the live values and restarts timing at the next step.
    /// A paused animation stays paused. Copies from earlier renders remain valid;
    /// calls after unmount are ignored, as with [`State::set`].
    pub fn reset(&self) {
        self.values.set(Values::default());
        self.reset_key.update(|key| *key = key.wrapping_add(1));
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Values {
    frame: u64,
    time: Duration,
    delta: Duration,
}

struct Entry {
    interval: Duration,
    start: Instant,
    next_due: Instant,
    last_render: Instant,
    values: State<Values>,
}

#[derive(Default)]
pub(super) struct AnimationState {
    now: Option<Instant>,
    entries: BTreeMap<usize, Entry>,
}

pub(super) struct Subscription {
    store: Rc<Store>,
    slot: usize,
}

impl Subscription {
    pub(super) fn new(store: Rc<Store>, interval: Duration, values: State<Values>) -> Self {
        {
            let mut animations = store.animation.borrow_mut();
            // Commits outside a step use its last timestamp; the core reads no clock.
            let start = animations
                .now
                .expect("animation subscription after first step");
            animations.entries.insert(
                values.slot,
                Entry {
                    interval,
                    start,
                    next_due: start + interval,
                    last_render: start,
                    values,
                },
            );
        }
        Self {
            store,
            slot: values.slot,
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.store.animation.borrow_mut().entries.remove(&self.slot);
    }
}

impl Runtime {
    pub(crate) fn tick_animations(&self, now: Instant, throttle: Duration) {
        let writes = {
            let mut animations = self.instance.store.animation.borrow_mut();
            animations.now = Some(now);
            let mut writes = Vec::new();
            for entry in animations.entries.values_mut() {
                if now < entry.next_due {
                    continue;
                }
                let elapsed = now - entry.start;
                let remainder = duration_remainder(elapsed, entry.interval);
                // Ink App.tsx advances from start + (floor(elapsed / interval) + 1)
                // * interval. The remainder form avoids multiplying a tick count.
                entry.next_due = now + (entry.interval - remainder);
                if now < entry.last_render + throttle {
                    continue;
                }
                writes.push((
                    entry.values,
                    Values {
                        frame: (elapsed.as_nanos() / entry.interval.as_nanos()) as u64,
                        time: elapsed,
                        delta: now - entry.last_render,
                    },
                ));
                entry.last_render = now;
            }
            writes
        };
        // State writes notify the driver; do not hold the registry borrow across them.
        for (state, values) in writes {
            state.set(values);
        }
    }

    pub(crate) fn animation_deadline(&self, throttle: Duration) -> Option<Instant> {
        self.instance
            .store
            .animation
            .borrow()
            .entries
            .values()
            .map(|entry| {
                let earliest = entry.next_due.max(entry.last_render + throttle);
                let remainder = duration_remainder(earliest - entry.start, entry.interval);
                // Keep Ink's delivered ticks, but omit wakes inside the throttle window.
                if remainder.is_zero() {
                    earliest
                } else {
                    earliest + (entry.interval - remainder)
                }
            })
            .min()
    }
}

fn duration_remainder(elapsed: Duration, interval: Duration) -> Duration {
    // The normalized interval is at most i32::MAX milliseconds, so this fits u64.
    Duration::from_nanos((elapsed.as_nanos() % interval.as_nanos()) as u64)
}
