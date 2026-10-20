// Ink examples/select-input.

use ink_rs::{Color, Element, FlexDirection, Options, Scope, Text, View, component, render, rsx};

fn select(cx: &mut Scope, _: &()) -> Element {
    let items = ["Red", "Green", "Blue", "Yellow", "Magenta", "Cyan"];
    let selected = cx.use_state(|| 0_usize);
    cx.use_input(true, move |_, key| {
        selected.update(|index| {
            if key.up_arrow {
                *index = (*index + items.len() - 1) % items.len();
            }
            if key.down_arrow {
                *index = (*index + 1) % items.len();
            }
        });
    });
    let items = items
        .into_iter()
        .enumerate()
        .map(|(i, item)| (item, selected.get() == i));
    // The deferred screen-reader mode would add aria list/listitem props and the
    // numbered-input branch.
    rsx! {
        View(flex_direction: FlexDirection::Column) {
            Text { "Select a color:" }
            for (item, active) in items {
                Text(color: active.then_some(Color::Blue), key: item) {
                    {format!("{} {item}", if active { ">" } else { " " })}
                }
            }
        }
    }
    .into()
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(select, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
