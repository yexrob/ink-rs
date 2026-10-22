//! Pure stdin decoding, input tokenization and key mapping. The driver owns I/O
//! and Runtime owns dispatch and escape-flush deadlines.

pub(crate) mod keypress;
pub(crate) mod kitty;
pub(crate) mod parser;

use keypress::{ParsedKey, is_non_alphanumeric_key};

/// A mouse button reported by the terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    /// Left button.
    Left,
    /// Middle button.
    Middle,
    /// Right button.
    Right,
}

/// The action in an SGR mouse report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseKind {
    /// A button was pressed.
    Press(MouseButton),
    /// A button was released. Reports with no button identity use Left.
    Release(MouseButton),
    /// Pointer motion while a button is held.
    Drag(MouseButton),
    /// Pointer motion with no button held; needs [`MouseTracking::Motion`].
    Move,
    /// The wheel moved up.
    WheelUp,
    /// The wheel moved down.
    WheelDown,
    /// The wheel moved left.
    WheelLeft,
    /// The wheel moved right.
    WheelRight,
}

/// Mouse action, cell coordinates and modifiers delivered to `use_mouse`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    /// The reported action.
    pub kind: MouseKind,
    /// Zero-based cells from the live frame's left edge, as in `ElementMetrics`.
    pub x: i32,
    /// Zero-based cells from the live frame's top, the origin of `ElementMetrics`.
    /// Negative over Static rows replayed above the live frame.
    pub y: i32,
    /// Set when the Control modifier is present.
    pub ctrl: bool,
    /// Set when Shift is present.
    pub shift: bool,
    /// Set for Meta or Alt/Option.
    pub meta: bool,
}

/// How much the terminal reports. Each level includes the ones before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseTracking {
    /// Press, release and the wheel (mode 1000).
    Click,
    /// Adds motion while a button is held (mode 1002).
    Drag,
    /// Adds motion with no button held, for hover (mode 1003).
    Motion,
}

impl MouseTracking {
    pub(crate) fn mode(self) -> u16 {
        match self {
            Self::Click => 1000,
            Self::Drag => 1002,
            Self::Motion => 1003,
        }
    }
}

/// The event kind reported by the kitty keyboard protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventType {
    /// The key was pressed.
    Press,
    /// The held key repeated.
    Repeat,
    /// The key was released.
    Release,
}

/// Key identity and modifiers delivered with input text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyEvent {
    /// Set for the Up arrow key.
    pub up_arrow: bool,
    /// Set for the Down arrow key.
    pub down_arrow: bool,
    /// Set for the Left arrow key.
    pub left_arrow: bool,
    /// Set for the Right arrow key.
    pub right_arrow: bool,
    /// Set for the Page Down key.
    pub page_down: bool,
    /// Set for the Page Up key.
    pub page_up: bool,
    /// Set for the Home key.
    pub home: bool,
    /// Set for the End key.
    pub end: bool,
    /// Set for Return or keypad Enter. A line feed has Ink's `enter` key name
    /// and does not set this flag.
    pub enter: bool,
    /// Set for the Escape key.
    pub escape: bool,
    /// Set when the Control modifier is present.
    pub ctrl: bool,
    /// Set when Shift is present, or legacy input is one ASCII capital letter.
    pub shift: bool,
    /// Set for the Tab key, including Shift+Tab.
    pub tab: bool,
    /// Set for Backspace (legacy BS or DEL, or kitty Backspace).
    pub backspace: bool,
    /// Set for the Delete key.
    pub delete: bool,
    /// Set for Meta or Alt/Option, including legacy escaped characters.
    pub meta: bool,
    /// Set for Super (Cmd/Win); available with kitty events.
    pub super_key: bool,
    /// Set for Hyper; available with kitty events.
    pub hyper: bool,
    /// Set when Caps Lock is active; available with kitty events.
    pub caps_lock: bool,
    /// Set when Num Lock is active; available with kitty events.
    pub num_lock: bool,
    /// Set for kitty events with a valid key code; absent for legacy input
    /// and rejected kitty code points.
    pub event_type: Option<KeyEventType>,
}

impl From<ParsedKey> for (String, KeyEvent) {
    /// Port of Ink src/hooks/use-input.ts:179-237. Keep text runs intact;
    /// functional keys and legacy control shortcuts choose different text.
    fn from(parsed: ParsedKey) -> Self {
        let mut key = KeyEvent {
            up_arrow: parsed.name == "up",
            down_arrow: parsed.name == "down",
            left_arrow: parsed.name == "left",
            right_arrow: parsed.name == "right",
            page_down: parsed.name == "pagedown",
            page_up: parsed.name == "pageup",
            home: parsed.name == "home",
            end: parsed.name == "end",
            enter: parsed.name == "return",
            escape: parsed.name == "escape",
            ctrl: parsed.ctrl,
            shift: parsed.shift,
            tab: parsed.name == "tab",
            backspace: parsed.name == "backspace",
            delete: parsed.name == "delete",
            meta: parsed.meta,
            super_key: parsed.super_key,
            hyper: parsed.hyper,
            caps_lock: parsed.caps_lock,
            num_lock: parsed.num_lock,
            event_type: parsed.event_type,
        };
        let mut input = if parsed.is_kitty_protocol {
            if parsed.is_printable {
                parsed.text.unwrap_or(parsed.name)
            } else {
                String::new()
            }
        } else if is_non_alphanumeric_key(&parsed.name) {
            String::new()
        } else if parsed.ctrl {
            if parsed.name == "space" {
                " ".into()
            } else {
                parsed.name
            }
        } else {
            parsed.sequence
        };
        if input.starts_with('\x1b') {
            input.remove(0);
        }
        // Replaces /[A-Z]/ with the length guard (use-input.ts:232-237).
        if !parsed.is_kitty_protocol && input.len() == 1 && input.as_bytes()[0].is_ascii_uppercase()
        {
            key.shift = true;
        }
        (input, key)
    }
}
