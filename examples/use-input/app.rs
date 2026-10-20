// Ink examples/use-input.

use ink_rs::{Element, FlexDirection, Scope, Text, View, component, rsx};

pub fn app() -> Element {
    component(robot, ())
}

fn robot(cx: &mut Scope, _: &()) -> Element {
    let position = cx.use_state(|| (1_u16, 1_u16));
    let app = cx.use_app();
    cx.use_input(true, move |input, key| {
        if input == "q" {
            app.exit();
        }
        position.update(|(x, y)| {
            if key.left_arrow {
                *x = x.saturating_sub(1).max(1);
            }
            if key.right_arrow {
                *x = (*x + 1).min(20);
            }
            if key.up_arrow {
                *y = y.saturating_sub(1).max(1);
            }
            if key.down_arrow {
                *y = (*y + 1).min(10);
            }
        });
    });
    let (x, y) = position.get();
    rsx! {
        View(flex_direction: FlexDirection::Column) {
            Text { "Use arrow keys to move the face. Press 'q' to exit." }
            View(height: 12, padding_left: x, padding_top: y) {
                Text { "^_^" }
            }
        }
    }
    .into()
}
