use std::collections::HashSet;
use std::rc::{Rc, Weak};
use std::thread::{self, ThreadId};

use super::Dirty;
use super::state::{Store, assert_not_rendering, resolve_store};

/// Options for [`super::Scope::use_focus`]. By default a hook is active,
/// does not request autofocus, and receives a generated ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusOptions {
    /// Whether this hook participates in navigation and holds raw mode on a TTY.
    /// Defaults to true. Deactivation clears focus if this hook's ID has focus.
    pub active: bool,
    /// Requests focus on registration when focus is enabled and no ID has focus.
    /// Defaults to false. The first active autofocus registrant wins.
    pub auto_focus: bool,
    /// An ID for [`FocusManager::focus`], including the empty string.
    /// Defaults to a stable per-hook ID, unique within its runtime, with an
    /// internal `ink-rs:focus:` prefix and counter; avoid this prefix for custom IDs.
    /// Duplicate IDs share the
    /// focused flag; navigation uses the first registration for each ID.
    pub id: Option<String>,
}

impl Default for FocusOptions {
    fn default() -> Self {
        Self {
            active: true,
            auto_focus: false,
            id: None,
        }
    }
}

/// A copyable UI-thread focus handle. Focus starts enabled with no active ID.
/// Mutations apply in call order and schedule focus consumers when the ID changes.
/// After app teardown begins, mutations do nothing and reads return `None`.
/// Access off the owning UI thread panics; mutations during render panic in
/// debug builds. Disabling clears focus; enabling does not restore it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusManager {
    runtime: u64,
    thread: ThreadId,
}

impl FocusManager {
    pub(super) fn new(store: &Store) -> Self {
        Self {
            runtime: store.id,
            thread: thread::current().id(),
        }
    }

    fn store(self) -> Option<Rc<Store>> {
        assert_eq!(
            self.thread,
            thread::current().id(),
            "focus access off the UI thread"
        );
        resolve_store(self.runtime).filter(|store| !store.unmounting.get())
    }

    fn update(self, f: impl FnOnce(&mut FocusState)) {
        assert_not_rendering();
        if let Some(store) = self.store() {
            store.update_focus(f);
        }
    }

    /// Enables Tab, Esc and registration-time autofocus (enabled by default).
    /// Does not restore the focus cleared by [`Self::disable_focus`].
    pub fn enable_focus(self) {
        self.update(|focus| focus.enabled = true);
    }

    /// Disables Tab, Esc and registration-time autofocus, and clears focus.
    /// Explicit focus and navigation methods remain available, as in Ink.
    pub fn disable_focus(self) {
        self.update(|focus| {
            focus.enabled = false;
            focus.active = None;
        });
    }

    /// Selects the next active ID in registration order, wrapping to the first.
    /// With no focus, selects the first active ID. Keyed moves do not change
    /// registration order; duplicate IDs use their first entry's active flag.
    /// Does not check whether keyboard focus management is enabled.
    pub fn focus_next(self) {
        self.update(|focus| focus.navigate(false));
    }

    /// Selects the previous active ID in registration order, wrapping to the last.
    /// With no focus, selects the last active ID, matching Ink's implementation.
    /// Duplicate IDs use their first entry. Works when focus management is disabled.
    pub fn focus_previous(self) {
        self.update(|focus| focus.navigate(true));
    }

    /// Focuses `id` if any entry with that ID is active; otherwise leaves focus
    /// unchanged. Accepts an empty ID and works when focus management is disabled.
    pub fn focus(self, id: &str) {
        self.update(|focus| {
            if focus
                .entries
                .iter()
                .any(|entry| entry.id == id && entry.active)
            {
                focus.active = Some(id.to_owned());
            }
        });
    }

    /// Reads the live focused ID, or `None` before focus is assigned or after
    /// teardown. Unlike Ink's render snapshot, handlers see preceding mutations.
    pub fn active_id(self) -> Option<String> {
        self.store()
            .and_then(|store| store.focus.borrow().active.clone())
    }
}

struct Entry {
    key: u64,
    id: String,
    active: bool,
}

pub(super) struct FocusState {
    entries: Vec<Entry>,
    pub(super) active: Option<String>,
    pub(super) enabled: bool,
    readers: Vec<Weak<Dirty>>,
    next_key: u64,
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            active: None,
            enabled: true,
            readers: Vec::new(),
            next_key: 0,
        }
    }
}

impl FocusState {
    pub(super) fn allocate(&mut self) -> u64 {
        let key = self.next_key;
        self.next_key = self.next_key.checked_add(1).expect("focus key overflow");
        key
    }

    pub(super) fn add_reader(&mut self, reader: &Rc<Dirty>) {
        let reader = Rc::downgrade(reader);
        if !self.readers.iter().any(|entry| entry.ptr_eq(&reader)) {
            self.readers.push(reader);
        }
    }

    pub(super) fn activate(&mut self, key: u64, active: bool) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key == key) {
            entry.active = active;
            if !active && self.active.as_ref() == Some(&entry.id) {
                self.active = None;
            }
        }
    }

    pub(super) fn navigate(&mut self, previous: bool) {
        // Ink App.tsx uniqueFocusables keeps the first entry, including its
        // active flag. Deduplicate before filtering inactive entries.
        let mut seen = HashSet::new();
        let entries: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| seen.insert(&entry.id))
            .collect();
        let index = entries
            .iter()
            .position(|entry| Some(&entry.id) == self.active.as_ref());
        let next = if previous {
            entries[..index.unwrap_or(0)]
                .iter()
                .rev()
                .find(|entry| entry.active)
                .or_else(|| entries.iter().rev().find(|entry| entry.active))
        } else {
            entries[index.map_or(0, |index| index + 1)..]
                .iter()
                .find(|entry| entry.active)
                .or_else(|| entries.iter().find(|entry| entry.active))
        };
        self.active = next.map(|entry| entry.id.clone());
    }

    pub(super) fn has_entries(&self) -> bool {
        !self.entries.is_empty()
    }
}

impl Store {
    pub(super) fn update_focus(&self, f: impl FnOnce(&mut FocusState)) {
        let mut focus = self.focus.borrow_mut();
        let previous = focus.active.clone();
        f(&mut focus);
        if focus.active != previous {
            focus.readers.retain(|reader| {
                let Some(reader) = reader.upgrade().filter(|reader| reader.mounted.get()) else {
                    return false;
                };
                reader.mark();
                true
            });
            drop(focus);
            self.queue.notify();
        }
    }
}

pub(super) struct FocusRegistration {
    store: Rc<Store>,
    key: u64,
}

impl FocusRegistration {
    pub(super) fn new(store: Rc<Store>, key: u64, id: String, auto_focus: bool) -> Self {
        store.update_focus(|focus| {
            focus.entries.push(Entry {
                key,
                id: id.clone(),
                active: true,
            });
            if auto_focus && focus.enabled && focus.active.is_none() {
                focus.active = Some(id);
            }
        });
        Self { store, key }
    }
}

impl Drop for FocusRegistration {
    fn drop(&mut self) {
        self.store.update_focus(|focus| {
            if let Some(index) = focus.entries.iter().position(|entry| entry.key == self.key) {
                let entry = focus.entries.remove(index);
                if focus.active.as_ref() == Some(&entry.id) {
                    focus.active = None;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::{FocusManager, FocusOptions};
    use crate::testing::{TestTerminal, mount, render};
    use crate::{Element, Error, Options, Scope, Text, component};

    type Capture = Rc<RefCell<Option<FocusManager>>>;

    fn field(cx: &mut Scope, options: &FocusOptions) -> Element {
        Text::new(cx.use_focus(options.clone()).to_string()).into()
    }

    fn controller(cx: &mut Scope, capture: &Capture) -> Element {
        *capture.borrow_mut() = Some(cx.use_focus_manager());
        Element::fragment(Vec::<Element>::new())
    }

    fn named(id: &str, active: bool) -> Element {
        component(
            field,
            FocusOptions {
                id: Some(id.into()),
                active,
                auto_focus: false,
            },
        )
    }

    // The ports read settled frames; this pins the layout-effect timing choice.
    #[test]
    fn autofocus_paints_in_the_first_frame() {
        let app = render(component(
            field,
            FocusOptions {
                auto_focus: true,
                ..FocusOptions::default()
            },
        ));
        assert_eq!(app.frames(), ["true"]);
    }

    // Ink's render snapshot cannot observe our before-handler ordering.
    #[test]
    fn handlers_read_live_focus_and_receive_tab_and_escape() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut app = render(component(
            |cx: &mut Scope, seen: &Rc<RefCell<Vec<Option<String>>>>| {
                let manager = cx.use_focus_manager();
                cx.use_focus(FocusOptions {
                    id: Some("a".into()),
                    ..FocusOptions::default()
                });
                let seen = seen.clone();
                cx.use_input(true, move |_, _| {
                    seen.borrow_mut().push(manager.active_id())
                });
                Text::new("").into()
            },
            seen.clone(),
        ));
        app.input("\t");
        app.input("\x1b[27u");
        assert_eq!(*seen.borrow(), [Some("a".into()), None]);
    }

    // No port covers backward navigation from none, disabled explicit calls,
    // or handles retained beyond runtime teardown.
    #[test]
    fn manager_navigation_without_focus_and_after_teardown() {
        let capture = Capture::default();
        let mut app = render(Element::fragment([
            component(controller, capture.clone()),
            named("a", true),
            named("b", true),
        ]));
        let manager = capture.borrow().unwrap();
        manager.focus_previous();
        assert_eq!(manager.active_id().as_deref(), Some("b"));
        manager.disable_focus();
        manager.focus("a");
        assert_eq!(manager.active_id().as_deref(), Some("a"));
        manager.focus_next();
        assert_eq!(manager.active_id().as_deref(), Some("b"));
        manager.focus("missing");
        assert_eq!(manager.active_id().as_deref(), Some("b"));
        app.settle();
        app.unmount();
        manager.enable_focus();
        manager.focus("a");
        manager.focus_next();
        manager.focus_previous();
        manager.disable_focus();
        assert_eq!(manager.active_id(), None);
    }

    // FO2/FO3 cover duplicates with matching activity, not hook-keyed cleanup,
    // first-entry activity in navigation, or any-entry activity in focus(id).
    #[test]
    fn duplicate_lifecycle_is_keyed_by_hook_instance() {
        let capture = Capture::default();
        let tree = |first_active, second, third| {
            Element::fragment([
                component(controller, capture.clone()),
                named("x", first_active).key("first"),
                if second {
                    named("x", true).key("second")
                } else {
                    Element::fragment(Vec::<Element>::new()).key("second")
                },
                named("b", third).key("third"),
            ])
        };
        let mut app = render(tree(false, true, true));
        let manager = capture.borrow().unwrap();
        manager.focus_next();
        assert_eq!(manager.active_id().as_deref(), Some("b"));
        manager.focus("x");
        assert_eq!(manager.active_id().as_deref(), Some("x"));
        app.rerender(tree(false, false, true));
        app.settle();
        assert_eq!(manager.active_id(), None);
        manager.focus("x");
        assert_eq!(manager.active_id(), None);
        app.rerender(tree(true, false, true));
        app.settle();
        manager.focus("x");
        assert_eq!(manager.active_id().as_deref(), Some("x"));
        app.rerender(tree(false, false, true));
        app.settle();
        assert_eq!(manager.active_id(), None);
    }

    // Ink's focus suite uses TTY stdin; it does not cover the holder distinction.
    #[test]
    fn focus_holders_skip_non_tty_raw_mode_without_hiding_input_errors() {
        fn form(cx: &mut Scope, (active, input, paste): &(bool, bool, bool)) -> Element {
            cx.use_focus(FocusOptions {
                active: *active,
                ..FocusOptions::default()
            });
            cx.use_input(*input, |_, _| {});
            cx.use_paste(*paste, |_| {});
            Text::new("").into()
        }
        for tty in [false, true] {
            let terminal = TestTerminal::new(100, 24).with_stdin_tty(tty);
            let mut app = mount(
                component(form, (true, false, false)),
                Options::default(),
                &terminal,
            );
            assert!(app.exit_result().is_none());
            assert_eq!(
                terminal.raw_mode_calls(),
                if tty { vec![true] } else { vec![] }
            );
            app.rerender(component(form, (false, false, false)));
            assert_eq!(
                terminal.raw_mode_calls(),
                if tty { vec![true, false] } else { vec![] }
            );
        }
        for (input, paste) in [(true, false), (false, true)] {
            let terminal = TestTerminal::new(100, 24).with_stdin_tty(false);
            let app = mount(
                component(form, (true, input, paste)),
                Options::default(),
                &terminal,
            );
            assert!(
                matches!(app.exit_result(), Some(Err(Error::Io(error))) if error.kind() == std::io::ErrorKind::Unsupported)
            );
            assert!(terminal.raw_mode_calls().is_empty());
        }
    }
}
