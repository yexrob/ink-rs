//! Ported from ink test/focus-empty-id-regression.tsx @ 26d2c3f.

use std::cell::RefCell;
use std::rc::Rc;

use ink_rs::testing::render;
use ink_rs::*;

fn field(cx: &mut Scope, (id, auto_focus): &(&str, bool)) -> Element {
    let focused = cx.use_focus(FocusOptions {
        id: Some((*id).into()),
        auto_focus: *auto_focus,
        ..FocusOptions::default()
    });
    let label = if id.is_empty() { "empty" } else { id };
    Text::new(format!(
        "{label}: {}",
        if focused { "focused" } else { "blurred" }
    ))
    .into()
}

fn navigate(programmatic: bool) {
    let capture = Rc::new(RefCell::new(None));
    let mut app = render(component(
        |cx: &mut Scope, capture: &Rc<RefCell<Option<FocusManager>>>| {
            *capture.borrow_mut() = Some(cx.use_focus_manager());
            component(field, ("", false))
        },
        capture.clone(),
    ));
    app.settle();
    assert_eq!(app.last_frame(), "empty: blurred");
    let manager = capture.borrow().unwrap();
    if programmatic {
        manager.focus("");
        app.settle();
    } else {
        app.input("\t");
    }
    assert_eq!(manager.active_id().as_deref(), Some(""));
    assert_eq!(app.last_frame(), "empty: focused");
}

#[test]
fn empty_focus_id_reports_focus_after_programmatic_navigation() {
    navigate(true);
}

#[test]
fn empty_focus_id_reports_focus_after_tab_navigation() {
    navigate(false);
}

#[test]
fn autofocus_does_not_replace_an_already_focused_empty_id() {
    let active = Rc::new(RefCell::new(None));
    let mut app = render(component(
        |cx: &mut Scope, active: &Rc<RefCell<Option<String>>>| {
            *active.borrow_mut() = cx.use_focus_manager().active_id();
            View::new()
                .flex_direction(FlexDirection::Column)
                .children([
                    component(field, ("", true)),
                    component(field, ("second", true)),
                ])
                .into()
        },
        active.clone(),
    ));
    app.settle();
    assert_eq!(active.borrow().as_deref(), Some(""));
    assert_eq!(app.last_frame(), "empty: focused\nsecond: blurred");
}
