//! Ported from ink test/focus.tsx @ 26d2c3f.
//! Debug steps replace act/delay/waitUntilRenderFlush; Esc uses the fake clock.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ink_rs::testing::{TestInstance, render};
use ink_rs::*;

#[derive(Clone, PartialEq)]
struct Props {
    show_first: bool,
    disabled_items: [bool; 3],
    auto_focus: bool,
    disabled: bool,
    focus_next: bool,
    focus_previous: bool,
    unmount_children: bool,
}
impl Default for Props {
    fn default() -> Self {
        Self {
            show_first: true,
            disabled_items: [false; 3],
            auto_focus: false,
            disabled: false,
            focus_next: false,
            focus_previous: false,
            unmount_children: false,
        }
    }
}
fn item(cx: &mut Scope, (label, options): &(&'static str, FocusOptions)) -> Element {
    let focused = cx.use_focus(options.clone());
    Text::new(format!("{label} {}", if focused { "✔" } else { "" })).into()
}
fn example(cx: &mut Scope, props: &Props) -> Element {
    let manager = cx.use_focus_manager();
    let p = props.clone();
    cx.use_effect(p.disabled, move || {
        if p.disabled {
            manager.disable_focus()
        } else {
            manager.enable_focus()
        }
    });
    cx.use_effect(p.focus_next, move || {
        if p.focus_next {
            manager.focus_next()
        }
    });
    cx.use_effect(p.focus_previous, move || {
        if p.focus_previous {
            manager.focus_previous()
        }
    });
    if p.unmount_children {
        return Element::fragment(Vec::<Element>::new());
    }
    View::new()
        .flex_direction(FlexDirection::Column)
        .children(
            ["First", "Second", "Third"]
                .into_iter()
                .enumerate()
                .map(|(i, label)| {
                    // React's conditional null retains the first child's slot.
                    if i == 0 && !p.show_first {
                        return Element::fragment(Vec::<Element>::new());
                    }
                    component(
                        item,
                        (
                            label,
                            FocusOptions {
                                active: !p.disabled_items[i],
                                auto_focus: p.auto_focus,
                                id: None,
                            },
                        ),
                    )
                }),
        )
        .into()
}
fn start(props: Props) -> TestInstance {
    let mut app = render(component(example, props));
    app.settle();
    app
}
fn change(app: &mut TestInstance, props: Props) {
    app.rerender(component(example, props));
    app.settle();
}
fn auto() -> Props {
    Props {
        auto_focus: true,
        ..Props::default()
    }
}

#[test]
fn focus_navigation_handles_kitty() {
    // F1-F16: failure messages retain each Ink case name.
    for (name, input, expected) in [
        ("Tab", "\x1b[9u", "First\nSecond ✔\nThird"),
        ("Shift+Tab", "\x1b[9;2u", "First\nSecond\nThird ✔"),
        ("Tab repeat", "\x1b[9;1:2u", "First\nSecond ✔\nThird"),
        ("Tab release", "\x1b[9;1:3u", "First ✔\nSecond\nThird"),
        ("Ctrl+Tab", "\x1b[9;5u", "First ✔\nSecond\nThird"),
        ("Alt+Tab", "\x1b[9;3u", "First ✔\nSecond\nThird"),
        ("Super+Tab", "\x1b[9;9u", "First ✔\nSecond\nThird"),
        ("Hyper+Tab", "\x1b[9;17u", "First ✔\nSecond\nThird"),
        ("Escape", "\x1b[27u", "First\nSecond\nThird"),
        ("Escape repeat", "\x1b[27;1:2u", "First\nSecond\nThird"),
        ("Escape release", "\x1b[27;1:3u", "First ✔\nSecond\nThird"),
        ("Shift+Escape", "\x1b[27;2u", "First ✔\nSecond\nThird"),
        ("Ctrl+Escape", "\x1b[27;5u", "First ✔\nSecond\nThird"),
        ("Alt+Escape", "\x1b[27;3u", "First ✔\nSecond\nThird"),
        ("Super+Escape", "\x1b[27;9u", "First ✔\nSecond\nThird"),
        ("Hyper+Escape", "\x1b[27;17u", "First ✔\nSecond\nThird"),
    ] {
        let mut app = start(auto());
        assert_eq!(
            app.last_frame(),
            "First ✔\nSecond\nThird",
            "{name}: initial"
        );
        app.input(input);
        assert_eq!(app.last_frame(), expected, "{name}");
    }
}

#[test]
fn do_not_focus_on_register_when_auto_focus_is_off() {
    let app = start(Props::default());
    assert_eq!(app.last_frame(), "First\nSecond\nThird");
}

#[test]
fn focus_the_first_component_to_register() {
    let app = start(auto());
    assert_eq!(app.last_frame(), "First ✔\nSecond\nThird");
}

#[test]
fn unfocus_active_component_on_esc() {
    let mut app = start(auto());
    assert_eq!(app.last_frame(), "First ✔\nSecond\nThird");
    app.input("\x1b");
    app.advance(Duration::from_millis(20));
    assert_eq!(app.last_frame(), "First\nSecond\nThird");
}

#[test]
fn switch_focus_to_first_component_on_tab() {
    let mut app = start(Props::default());
    app.input("\t");
    assert_eq!(app.last_frame(), "First ✔\nSecond\nThird");
}

#[test]
fn switch_focus_to_the_next_component_on_tab() {
    let mut app = start(Props::default());
    app.input("\t");
    app.input("\t");
    assert_eq!(app.last_frame(), "First\nSecond ✔\nThird");
}

#[test]
fn switch_focus_to_the_first_component_if_currently_focused_component_is_the_last_one_on_tab() {
    let mut app = start(auto());
    app.input("\t");
    app.input("\t");
    assert_eq!(app.last_frame(), "First\nSecond\nThird ✔");
    app.input("\t");
    assert_eq!(app.last_frame(), "First ✔\nSecond\nThird");
}

#[test]
fn skip_disabled_component_on_tab() {
    let mut app = start(Props {
        disabled_items: [false, true, false],
        ..auto()
    });
    app.input("\t");
    assert_eq!(app.last_frame(), "First\nSecond\nThird ✔");
}

#[test]
fn switch_focus_to_the_previous_component_on_shift_tab() {
    let mut app = start(auto());
    app.input("\t");
    assert_eq!(app.last_frame(), "First\nSecond ✔\nThird");
    app.input("\x1b[Z");
    assert_eq!(app.last_frame(), "First ✔\nSecond\nThird");
}

#[test]
fn switch_focus_to_the_last_component_if_currently_focused_component_is_the_first_one_on_shift_tab()
{
    let mut app = start(auto());
    app.input("\x1b[Z");
    assert_eq!(app.last_frame(), "First\nSecond\nThird ✔");
}

#[test]
fn skip_disabled_component_on_shift_tab() {
    let mut app = start(Props {
        disabled_items: [false, true, false],
        ..auto()
    });
    app.input("\x1b[Z");
    app.input("\x1b[Z");
    assert_eq!(app.last_frame(), "First ✔\nSecond\nThird");
}

#[test]
fn reset_focus_when_focused_component_unregisters() {
    let mut app = start(auto());
    change(
        &mut app,
        Props {
            show_first: false,
            ..auto()
        },
    );
    assert_eq!(app.last_frame(), "Second\nThird");
}

#[test]
fn focus_first_component_after_focused_component_unregisters() {
    let mut app = start(auto());
    change(
        &mut app,
        Props {
            show_first: false,
            ..auto()
        },
    );
    assert_eq!(app.last_frame(), "Second\nThird");
    app.input("\t");
    assert_eq!(app.last_frame(), "Second ✔\nThird");
}

#[test]
fn toggle_focus_management() {
    let mut app = start(auto());
    assert_eq!(app.last_frame(), "First ✔\nSecond\nThird");
    change(
        &mut app,
        Props {
            disabled: true,
            ..auto()
        },
    );
    assert_eq!(app.last_frame(), "First\nSecond\nThird");
    app.input("\t");
    assert_eq!(app.last_frame(), "First\nSecond\nThird");
    change(&mut app, auto());
    app.input("\t");
    assert_eq!(app.last_frame(), "First ✔\nSecond\nThird");
}

type Capture = Rc<RefCell<Option<FocusManager>>>;
type ActiveId = Rc<RefCell<Option<String>>>;
fn controller(cx: &mut Scope, capture: &Capture) -> Element {
    *capture.borrow_mut() = Some(cx.use_focus_manager());
    Element::fragment(Vec::<Element>::new())
}
fn reader(cx: &mut Scope, active: &ActiveId) -> Element {
    *active.borrow_mut() = cx.use_focus_manager().active_id();
    Element::fragment(Vec::<Element>::new())
}
fn named(label: &'static str, id: &str, auto_focus: bool, active: bool) -> Element {
    component(
        item,
        (
            label,
            FocusOptions {
                id: Some(id.into()),
                auto_focus,
                active,
            },
        ),
    )
}
fn column(children: impl IntoIterator<Item = Element>) -> Element {
    View::new()
        .flex_direction(FlexDirection::Column)
        .children(children)
        .into()
}

#[test]
fn new_components_do_not_auto_focus_while_focus_management_is_disabled() {
    let capture = Capture::default();
    let mut app = render(Element::fragment([component(controller, capture.clone())]));
    app.settle();
    let manager = capture.borrow().unwrap();
    manager.disable_focus();
    app.settle();
    app.rerender(Element::fragment([
        component(controller, capture.clone()),
        named("New", "new", true, true),
    ]));
    app.settle();
    assert_eq!(app.last_frame(), "New");
    manager.enable_focus();
    app.settle();
    assert_eq!(app.last_frame(), "New");
    app.input("\t");
    assert_eq!(app.last_frame(), "New ✔");
}

#[test]
fn manually_focus_next_component() {
    let mut app = start(auto());
    change(
        &mut app,
        Props {
            focus_next: true,
            ..auto()
        },
    );
    assert_eq!(app.last_frame(), "First\nSecond ✔\nThird");
}

#[test]
fn manually_focus_previous_component() {
    let mut app = start(auto());
    change(
        &mut app,
        Props {
            focus_previous: true,
            ..auto()
        },
    );
    assert_eq!(app.last_frame(), "First\nSecond\nThird ✔");
}

#[test]
fn does_not_crash_when_focusing_next_on_unmounted_children() {
    let mut app = start(auto());
    change(
        &mut app,
        Props {
            focus_next: true,
            unmount_children: true,
            ..Props::default()
        },
    );
    assert_eq!(app.last_frame(), "");
}

#[test]
fn does_not_crash_when_focusing_previous_on_unmounted_children() {
    let mut app = start(auto());
    change(
        &mut app,
        Props {
            focus_previous: true,
            unmount_children: true,
            ..Props::default()
        },
    );
    assert_eq!(app.last_frame(), "");
}

#[test]
fn focuses_first_non_disabled_component() {
    let app = start(Props {
        disabled_items: [true, true, false],
        ..auto()
    });
    assert_eq!(app.last_frame(), "First\nSecond\nThird ✔");
}

#[test]
fn changing_auto_focus_does_not_reactivate_disabled_components() {
    let mut app = start(Props {
        disabled_items: [true, false, false],
        ..Props::default()
    });
    assert_eq!(app.last_frame(), "First\nSecond\nThird");
    change(
        &mut app,
        Props {
            disabled_items: [true, false, false],
            ..auto()
        },
    );
    assert_eq!(app.last_frame(), "First\nSecond ✔\nThird");
    app.input("\x1b[Z");
    assert_eq!(app.last_frame(), "First\nSecond\nThird ✔");
    change(
        &mut app,
        Props {
            disabled_items: [true, false, false],
            ..Props::default()
        },
    );
    app.input("\t");
    assert_eq!(app.last_frame(), "First\nSecond ✔\nThird");
}

#[test]
fn skips_disabled_elements_when_wrapping_around() {
    let mut app = start(Props {
        disabled_items: [true, false, false],
        ..auto()
    });
    app.input("\t");
    app.input("\t");
    assert_eq!(app.last_frame(), "First\nSecond ✔\nThird");
}

#[test]
fn skips_disabled_elements_when_wrapping_around_from_the_front() {
    let mut app = start(Props {
        disabled_items: [false, false, true],
        ..auto()
    });
    app.input("\x1b[Z");
    assert_eq!(app.last_frame(), "First\nSecond ✔\nThird");
}

#[test]
fn active_id_from_use_focus_manager_reflects_currently_focused_component() {
    let active = ActiveId::default();
    let mut app = render(column([
        component(reader, active.clone()),
        named("First", "first", false, true),
        named("Second", "second", false, true),
    ]));
    app.settle();
    assert_eq!(*active.borrow(), None);
    app.input("\t");
    assert_eq!(active.borrow().as_deref(), Some("first"));
    app.input("\t");
    assert_eq!(active.borrow().as_deref(), Some("second"));
}

#[test]
fn active_id_resets_to_undefined_on_esc() {
    let active = ActiveId::default();
    let mut app = render(column([
        component(reader, active.clone()),
        named("First", "first", false, true),
    ]));
    app.settle();
    app.input("\t");
    assert_eq!(active.borrow().as_deref(), Some("first"));
    app.input("\x1b");
    app.advance(Duration::from_millis(20));
    assert_eq!(*active.borrow(), None);
}

#[test]
fn active_id_is_set_immediately_when_component_uses_auto_focus() {
    let active = ActiveId::default();
    let mut app = render(column([
        component(reader, active.clone()),
        named("First", "first", true, true),
        named("Second", "second", false, true),
    ]));
    app.settle();
    assert_eq!(active.borrow().as_deref(), Some("first"));
}

#[test]
fn active_id_updates_when_focus_is_changed_programmatically() {
    let active = ActiveId::default();
    let capture = Capture::default();
    let mut app = render(column([
        component(reader, active.clone()),
        component(controller, capture.clone()),
        named("First", "first", false, true),
        named("Second", "second", false, true),
    ]));
    app.settle();
    assert_eq!(*active.borrow(), None);
    let manager = capture.borrow().unwrap();
    manager.focus("second");
    app.settle();
    assert_eq!(active.borrow().as_deref(), Some("second"));
    manager.focus("first");
    app.settle();
    assert_eq!(active.borrow().as_deref(), Some("first"));
}

#[test]
fn programmatic_focus_skips_inactive_components_until_they_are_re_enabled() {
    fn form(cx: &mut Scope, (capture, active): &(Capture, bool)) -> Element {
        *capture.borrow_mut() = Some(cx.use_focus_manager());
        column([
            named("First", "first", true, true),
            named("Second", "second", false, *active),
        ])
    }
    let capture = Capture::default();
    let mut app = render(component(form, (capture.clone(), false)));
    app.settle();
    assert_eq!(app.last_frame(), "First ✔\nSecond");
    let manager = capture.borrow().unwrap();
    manager.focus("second");
    app.settle();
    assert_eq!(app.last_frame(), "First ✔\nSecond");
    app.rerender(component(form, (capture.clone(), true)));
    app.settle();
    manager.focus("second");
    app.settle();
    assert_eq!(app.last_frame(), "First\nSecond ✔");
}

#[test]
fn active_id_resets_to_undefined_when_focused_component_unmounts() {
    let active = ActiveId::default();
    let mut app = render(column([
        component(reader, active.clone()),
        named("First", "first", true, true),
        named("Second", "second", false, true),
    ]));
    app.settle();
    assert_eq!(active.borrow().as_deref(), Some("first"));
    app.rerender(column([
        component(reader, active.clone()),
        named("Second", "second", false, true),
    ]));
    app.settle();
    assert_eq!(*active.borrow(), None);
}
