// Ink examples/use-animation.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use ink_rs::{
    AnimationOptions, Color, Element, FlexDirection, Options, Scope, Text, View, component, render,
    rsx,
};

const COLORS: [Color; 6] = [
    Color::Red,
    Color::Yellow,
    Color::Green,
    Color::Cyan,
    Color::Blue,
    Color::Magenta,
];

fn scene(cx: &mut Scope, _: &()) -> Element {
    let paused = cx.use_state(|| false);
    // All three scenes derive from frame; late wakes skip to the elapsed phase.
    let frames = [80, 50, 400].map(|ms| {
        cx.use_animation(AnimationOptions {
            interval: Duration::from_millis(ms),
            active: !paused.get(),
        })
        .frame
    });
    let frozen = cx.use_memo((), || Rc::new(Cell::new([0_u64; 3])));
    if !paused.get() {
        frozen.set(frames);
    }
    let [fast, movement, slow] = frozen.get();
    cx.use_input(true, move |input, _| {
        if input == " " {
            paused.update(|value| *value = !*value);
        }
    });
    let title = rsx! {
        Text(bold: true) {
            "  "
            for (i, ch) in "Unicorns are magical!".chars().enumerate() {
                Text(color: COLORS[(fast as usize + i) % COLORS.len()]) { {ch.to_string()} }
            }
        }
    };
    let columns = (0..44).map(|column| {
        let position = movement as usize % 44;
        let behind = (position + 44 - column) % 44;
        (column, position, behind)
    });
    let track = rsx! {
        Text {
            "  "
            for (column, position, behind) in columns {
                if column == position {
                    Text { "\u{1f984}" }
                } else if (1..=18).contains(&behind) {
                    Text(color: COLORS[5 - (behind - 1) / 3]) { "\u{2501}" }
                } else {
                    Text { " " }
                }
            }
        }
    };
    let sparkle = |seed: usize| {
        let chars = ['\u{2726}', '\u{2727}', '\u{b7}', '\u{22c6}'];
        format!(
            "  {}",
            (0..48)
                .map(|i| if (i * 7 + seed * 13) % 19 < 3 {
                    chars[(slow as usize + i + seed) % chars.len()]
                } else {
                    ' '
                })
                .collect::<String>()
        )
    };
    let spinners = [
        '\u{280b}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283c}', '\u{2834}', '\u{2826}',
        '\u{2827}', '\u{2807}', '\u{280f}',
    ];
    let spinner = spinners[fast as usize % spinners.len()];
    rsx! {
        View(flex_direction: FlexDirection::Column, padding: 1) {
            {title}
            Text { " " }
            Text { {sparkle(0)} }
            {track}
            Text { {sparkle(5)} }
            Text { " " }
            Text(color: Color::Cyan) { "  {spinner} Loading more unicorns..." }
            Text { " " }
            Text(dim_color: true) {
                {format!(
                    "  Press <space> to {}",
                    if paused.get() { "resume" } else { "pause" }
                )}
            }
        }
    }
    .into()
}

fn main() -> Result<(), ink_rs::Error> {
    render(component(scene, ()), Options::default())
        .wait_until_exit()
        .map(|_| ())
}
