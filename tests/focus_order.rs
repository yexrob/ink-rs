//! Ported from ink test/focus-order.tsx @ 26d2c3f.

use std::cell::RefCell;
use std::rc::Rc;

use ink_rs::testing::{TestInstance, render};
use ink_rs::*;

type Capture = Rc<RefCell<Option<FocusManager>>>;

fn field(cx: &mut Scope, id: &&str) -> Element {
    let focused = cx.use_focus(FocusOptions {
        id: Some((*id).into()),
        ..FocusOptions::default()
    });
    Text::new(format!(
        "{id}: {}",
        if focused { "focused" } else { "blurred" }
    ))
    .into()
}

fn form(cx: &mut Scope, (capture, fields): &(Capture, Vec<(&'static str, String)>)) -> Element {
    *capture.borrow_mut() = Some(cx.use_focus_manager());
    View::new()
        .flex_direction(FlexDirection::Column)
        .children(
            fields
                .iter()
                .map(|(id, key)| component(field, *id).key(key.clone())),
        )
        .into()
}

#[test]
fn focus_navigation_keeps_registration_order_after_keyed_reordering() {
    let capture = Capture::default();
    let fields = |ids: [&'static str; 3]| {
        ids.into_iter()
            .map(|id| (id, id.into()))
            .collect::<Vec<_>>()
    };
    let mut app = render(component(form, (capture.clone(), fields(["a", "b", "c"]))));
    app.settle();
    app.rerender(component(form, (capture.clone(), fields(["c", "a", "b"]))));
    app.settle();
    assert_eq!(app.last_frame(), "c: blurred\na: blurred\nb: blurred");
    app.input("\t");
    assert_eq!(capture.borrow().unwrap().active_id().as_deref(), Some("a"));
}

fn duplicates(ids: &[&'static str]) -> (TestInstance, FocusManager) {
    let capture = Capture::default();
    let fields = ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, format!("{id}-{index}")))
        .collect();
    let mut app = render(component(form, (capture.clone(), fields)));
    app.settle();
    let manager = capture.borrow().unwrap();
    (app, manager)
}

fn press(app: &mut TestInstance, manager: FocusManager, input: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            app.input(input);
            manager.active_id().expect("focused ID")
        })
        .collect()
}

#[test]
fn tab_visits_a_duplicated_focus_id_once_and_reaches_the_components_after_it() {
    let (mut app, manager) = duplicates(&["a", "x", "x", "b"]);
    assert_eq!(press(&mut app, manager, "\t", 5), ["a", "x", "b", "a", "x"]);
    assert_eq!(
        app.last_frame(),
        "a: blurred\nx: focused\nx: focused\nb: blurred"
    );
    assert_eq!(press(&mut app, manager, "\x1b[Z", 4), ["a", "b", "x", "a"]);
}

#[test]
fn tab_visits_a_non_adjacent_duplicated_focus_id_at_its_first_registration() {
    let (mut app, manager) = duplicates(&["x", "a", "x", "b"]);
    assert_eq!(press(&mut app, manager, "\t", 4), ["x", "a", "b", "x"]);
    assert_eq!(press(&mut app, manager, "\x1b[Z", 4), ["b", "a", "x", "b"]);
}
