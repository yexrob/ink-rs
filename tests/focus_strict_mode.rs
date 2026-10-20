//! Ported from ink test/focus-strict-mode.tsx @ 26d2c3f.
//! FS1: concurrent false, StrictMode false. Calls between steps form one batch.

use std::cell::RefCell;
use std::rc::Rc;

use ink_rs::testing::render;
use ink_rs::*;

#[test]
// Ink: focus updates preserve call order (concurrent: false, StrictMode: false).
fn focus_updates_preserve_call_order() {
    fn field(cx: &mut Scope, id: &&str) -> Element {
        let focused = cx.use_focus(FocusOptions {
            id: Some((*id).into()),
            ..FocusOptions::default()
        });
        Text::new(if focused {
            format!("[{id}]")
        } else {
            (*id).into()
        })
        .into()
    }
    let capture = Rc::new(RefCell::new(None));
    let mut app = render(component(
        |cx: &mut Scope, capture: &Rc<RefCell<Option<FocusManager>>>| {
            *capture.borrow_mut() = Some(cx.use_focus_manager());
            View::new()
                .children(["a", "b", "c"].map(|id| component(field, id)))
                .into()
        },
        capture.clone(),
    ));
    app.settle();
    let manager = capture.borrow().unwrap();
    manager.focus_next();
    manager.focus_next();
    app.settle();
    assert_eq!(
        manager.active_id().as_deref(),
        Some("b"),
        "Two batched focusNext calls must advance twice"
    );
    manager.focus_next();
    manager.focus("a");
    app.settle();
    assert_eq!(
        manager.active_id().as_deref(),
        Some("a"),
        "An explicit focus after navigation wins"
    );
    manager.focus("c");
    manager.focus_previous();
    app.settle();
    assert_eq!(
        manager.active_id().as_deref(),
        Some("b"),
        "Navigation starts from the latest explicit focus"
    );
    for expected in ["c", "a", "b", "c"] {
        app.input("\t");
        assert_eq!(manager.active_id().as_deref(), Some(expected));
    }
    for method in ["focusNext", "focusPrevious", "focus"] {
        manager.enable_focus();
        manager.focus("b");
        app.settle();
        match method {
            "focusNext" => manager.focus_next(),
            "focusPrevious" => manager.focus_previous(),
            _ => manager.focus("a"),
        }
        manager.disable_focus();
        app.settle();
        assert_eq!(
            manager.active_id(),
            None,
            "disableFocus must take effect after {method}"
        );
        assert_eq!(app.last_frame(), "abc");
        app.input("\t");
        assert_eq!(manager.active_id(), None, "Tab must remain disabled");
    }
    manager.enable_focus();
    app.input("\t");
    assert_eq!(
        manager.active_id().as_deref(),
        Some("a"),
        "Re-enabling focus restores keyboard navigation"
    );
    assert_eq!(app.last_frame(), "[a]bc");
}
