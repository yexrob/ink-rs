// Not an Ink port: shows the paint rate that `max_fps` holds.
//
// `cargo run --example frame-rate -- 60` paints at up to 60 frames per
// second, 120 by default. The interval starts at the whole milliseconds that
// fit in one window, an interval that paints at the full rate; ↑/↓ change it.

use std::collections::VecDeque;
use std::time::Duration;

use ink_rs::{
    AnimationOptions, BorderStyle, Color, Element, FlexDirection, Options, Scope, Text, View,
    component, render, rsx,
};

const WIDTH: usize = 48;
/// Columns the bar grows per second.
const SPEED: f64 = 24.0;
const EIGHTHS: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];

fn meter(cx: &mut Scope, max_fps: &u32) -> Element {
    let interval = cx.use_state({
        let window = 1000_u32.checked_div(*max_fps).unwrap_or(1);
        move || u64::from(window.max(1))
    });
    // Frame intervals of the last second.
    let deltas = cx.use_state(VecDeque::<Duration>::new);
    let app = cx.use_app();
    let clock = cx.use_animation(AnimationOptions {
        interval: Duration::from_millis(interval.get()),
        ..AnimationOptions::default()
    });
    // The bar derives from time; the intervals step once per delivered frame.
    cx.use_effect(clock.frame, move || {
        if !clock.delta.is_zero() {
            deltas.update(|deltas| {
                deltas.push_back(clock.delta);
                let mut span: Duration = deltas.iter().sum();
                while span > Duration::from_secs(1)
                    && let Some(oldest) = deltas.pop_front()
                {
                    span -= oldest;
                }
            });
        }
    });
    cx.use_input(true, move |input, key| {
        if key.up_arrow {
            interval.update(|ms| *ms = (*ms + 1).min(100));
        } else if key.down_arrow {
            interval.update(|ms| *ms = (*ms - 1).max(1));
        } else if input == "q" || key.escape {
            app.exit();
        }
    });

    let eighths = (clock.time.as_secs_f64() * SPEED * 8.0) as usize % (WIDTH * 8);
    let bar = format!("{}{}", "█".repeat(eighths / 8), EIGHTHS[eighths % 8]);
    let deltas = deltas.get();
    let rate = if deltas.is_empty() {
        "measuring…".to_owned()
    } else {
        let mut sorted: Vec<Duration> = deltas.iter().copied().collect();
        sorted.sort();
        let ms = |q: f64| {
            let index = ((sorted.len() - 1) as f64 * q).round() as usize;
            sorted[index].as_secs_f64() * 1000.0
        };
        let span: Duration = sorted.iter().sum();
        format!(
            "{:.1} fps · frame interval p50 {:.2} ms · p90 {:.2} ms",
            sorted.len() as f64 / span.as_secs_f64(),
            ms(0.5),
            ms(0.9)
        )
    };
    rsx! {
        View(flex_direction: FlexDirection::Column, padding: 1) {
            Text { {format!("max_fps {max_fps} · interval {} ms (↑/↓)", interval.get())} }
            View(border_style: BorderStyle::Single, width: WIDTH as u16 + 2) {
                Text(color: Color::Green) { {bar} }
            }
            Text { {rate} }
            Text(dim_color: true) { "Press q to exit" }
        }
    }
    .into()
}

fn main() -> Result<(), ink_rs::Error> {
    let max_fps = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(120);
    render(
        component(meter, max_fps),
        Options {
            max_fps,
            ..Options::default()
        },
    )
    .wait_until_exit()
    .map(|_| ())
}
