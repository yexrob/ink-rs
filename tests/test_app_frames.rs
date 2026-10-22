use ink_rs::testing::{TestTerminal, mount, render};
use ink_rs::*;
use std::time::Duration;

fn fixture(cx: &mut Scope, _: &()) -> Element {
    let count = cx.use_state(|| 0);
    cx.use_input(true, move |_, _| count.set(count.get() + 1));
    let (key, items) = match count.get() {
        0 => ("first", vec!["one", "", "three"]),
        1 => ("first", vec!["one", "", "three", "added"]),
        _ => ("replacement", vec!["replacement"]),
    };
    Element::fragment::<[Element; 2]>([
        Static::new(items, |item, _| {
            if item.is_empty() {
                View::new().height(1).into()
            } else {
                Text::new(*item).into()
            }
        })
        .key(key),
        Text::new(format!("live {}", count.get())).bold(true).into(),
    ])
}

fn assert_frames(alternate_screen: bool) {
    let terminal = TestTerminal::new(40, 2);
    let mut app = mount(
        component(fixture, ()),
        Options {
            alternate_screen,
            ..Options::default()
        },
        &terminal,
    );
    assert_eq!(app.static_output(), "one\n\nthree");
    assert_eq!(
        app.live_frame(),
        render(Text::new("live 0").bold(true)).last_frame()
    );

    for (count, history) in [
        (1, "one\n\nthree\nadded"),
        (2, "one\n\nthree\nadded\nreplacement"),
        (3, "one\n\nthree\nadded\nreplacement"),
    ] {
        app.input(b"a");
        app.advance(Duration::from_millis(34));
        assert_eq!(app.static_output(), history);
        assert_eq!(
            app.live_frame(),
            render(Text::new(format!("live {count}")).bold(true)).last_frame()
        );
    }
}

#[test]
fn inline_frames_separate_static_history_from_live_rows() {
    assert_frames(false);
}

#[test]
fn alternate_frames_separate_static_history_from_live_rows() {
    assert_frames(true);
}
