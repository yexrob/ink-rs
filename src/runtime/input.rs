use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::Runtime;
use super::state::Store;
use crate::input::keypress::parse_keypress;
use crate::input::kitty::is_query_response;
use crate::input::parser::{InputEvent, InputParser, is_complete_control_sequence};
use crate::{KeyEvent, KeyEventType};

pub(super) type InputHandler = Rc<RefCell<Box<dyn FnMut(&str, KeyEvent)>>>;
pub(super) type PasteHandler = Rc<RefCell<Box<dyn FnMut(&str)>>>;

#[derive(Clone)]
pub(super) enum Handler {
    Input(InputHandler),
    Paste(PasteHandler),
    Focus(u64),
}

impl Handler {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Input(a), Self::Input(b)) => Rc::ptr_eq(a, b),
            (Self::Paste(a), Self::Paste(b)) => Rc::ptr_eq(a, b),
            (Self::Focus(a), Self::Focus(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Default)]
pub(super) struct InputState {
    parser: InputParser,
    deadline: Option<Instant>,
    handlers: Vec<Handler>,
}

pub(super) struct Registration {
    store: Rc<Store>,
    handler: Handler,
}

impl Registration {
    pub(super) fn new(store: Rc<Store>, handler: Handler) -> Self {
        store.input.borrow_mut().handlers.push(handler.clone());
        Self { store, handler }
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        let mut input = self.store.input.borrow_mut();
        input
            .handlers
            .retain(|handler| !handler.same(&self.handler));
        if input.handlers.is_empty() {
            // Ink App.clearInputState runs at zero holders, before the deferred
            // terminal switch, including during a same-commit component swap.
            input.parser.reset();
            input.deadline = None;
        }
    }
}

impl Runtime {
    pub(crate) fn take_stop_request(&mut self) -> bool {
        std::mem::take(&mut self.stop_requested)
    }

    pub(crate) fn pause_input(&mut self) {
        let mut input = self.instance.store.input.borrow_mut();
        input.parser.reset();
        input.deadline = None;
    }

    pub(crate) fn input_holders(&self) -> usize {
        self.instance.store.input.borrow().handlers.len()
    }

    pub(crate) fn paste_holders(&self) -> usize {
        self.instance
            .store
            .input
            .borrow()
            .handlers
            .iter()
            .filter(|handler| matches!(handler, Handler::Paste(_)))
            .count()
    }

    pub(crate) fn focus_holders(&self) -> usize {
        self.instance
            .store
            .input
            .borrow()
            .handlers
            .iter()
            .filter(|handler| matches!(handler, Handler::Focus(_)))
            .count()
    }

    pub(crate) fn input_deadline(&self) -> Option<Instant> {
        self.instance.store.input.borrow().deadline
    }

    pub(crate) fn input(&mut self, bytes: &[u8], now: Instant) -> bool {
        if self.input_holders() == 0 || self.queue().has_exit() {
            return false;
        }
        let mut query_response = false;
        let events = {
            let mut input = self.instance.store.input.borrow_mut();
            input.deadline = None;
            input.parser.push(bytes)
        };
        for event in events {
            if self.queue().has_exit() {
                break;
            }
            let pasted = matches!(&event, InputEvent::Paste(_));
            let text = match event {
                InputEvent::Text(text) => {
                    if is_query_response(&text) {
                        query_response = true;
                        continue;
                    }
                    // Ink App.handleReadable drops unmapped terminal reports,
                    // but neither paste fallback nor timed flush uses this filter.
                    if is_complete_control_sequence(&text) {
                        let key = parse_keypress(&text);
                        if !key.is_kitty_protocol && key.name.is_empty() {
                            continue;
                        }
                    }
                    text
                }
                InputEvent::Paste(text) => {
                    // Ink App.handleReadable emits paste on its own channel,
                    // without key decoding, Ctrl+C handling or the drop filter.
                    let handlers = self.instance.store.input.borrow().handlers.clone();
                    if handlers
                        .iter()
                        .any(|handler| matches!(handler, Handler::Paste(_)))
                    {
                        for handler in handlers {
                            if let Handler::Paste(handler) = handler {
                                handler.borrow_mut()(&text);
                            }
                        }
                        continue;
                    }
                    text
                }
                InputEvent::StringSequence(_) => continue,
            };
            self.emit_input(&text, pasted);
        }
        let mut input = self.instance.store.input.borrow_mut();
        if input.parser.has_pending_escape() {
            input.deadline = Some(now + Duration::from_millis(20));
        }
        query_response
    }

    pub(crate) fn flush_input(&mut self, now: Instant) {
        let text = {
            let mut input = self.instance.store.input.borrow_mut();
            if input.deadline.is_none_or(|deadline| now < deadline) {
                return;
            }
            input.deadline = None;
            input.parser.flush_pending_escape()
        };
        if let Some(text) = text
            && !self.queue().has_exit()
        {
            self.emit_input(&text, false);
            // String timeouts release the Alt introducer first; parse its tail
            // through the ordinary input path, including control-key splitting.
            self.input(&[], now);
        }
    }

    fn emit_input(&mut self, text: &str, pasted: bool) {
        let parsed = parse_keypress(text);
        if self.exit_on_ctrl_c
            && parsed.name == "c"
            && parsed.ctrl
            && parsed.event_type != Some(KeyEventType::Release)
        {
            self.queue().request_exit(Ok(None));
            return;
        }
        if self.suspend_on_ctrl_z
            && !pasted
            && parsed.name == "z"
            && parsed.ctrl
            && !parsed.shift
            && !parsed.meta
            && !parsed.super_key
            && !parsed.hyper
            && parsed.event_type != Some(KeyEventType::Release)
        {
            self.stop_requested = true;
            return;
        }
        // Ink's App effect installs Tab after mount-time input listeners.
        // Here focus precedes all handlers: a live active_id read sees the new
        // ID. Neither Tab nor Esc is consumed by focus management.
        if parsed.event_type != Some(KeyEventType::Release)
            && !parsed.ctrl
            && !parsed.meta
            && !parsed.super_key
            && !parsed.hyper
        {
            self.instance.store.update_focus(|focus| {
                if !focus.enabled {
                    return;
                }
                match parsed.name.as_str() {
                    "escape" if !parsed.shift => focus.active = None,
                    "tab" if focus.has_entries() => focus.navigate(parsed.shift),
                    _ => {}
                }
            });
        }
        let (text, key) = parsed.into();
        // EventEmitter.emit snapshots listeners. No shared Store borrow crosses
        // user code: handlers may update state, request exit or write stdout.
        let handlers = self.instance.store.input.borrow().handlers.clone();
        for handler in handlers {
            if let Handler::Input(handler) = handler {
                handler.borrow_mut()(&text, key);
            }
        }
    }
}
