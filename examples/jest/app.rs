// Ink examples/jest/{jest,summary,test}.

use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::time::{Duration, Instant};

use ink_rs::{Color, Element, FlexDirection, Scope, Static, Text, View, component, rsx};

const PATHS: [&str; 10] = [
    "tests/login.js",
    "tests/signup.js",
    "tests/forgot-password.js",
    "tests/reset-password.js",
    "tests/view-profile.js",
    "tests/edit-profile.js",
    "tests/delete-profile.js",
    "tests/posts.js",
    "tests/post.js",
    "tests/comments.js",
];

#[derive(Clone)]
struct TestResult {
    path: &'static str,
    status: &'static str,
}

pub fn app() -> Element {
    component(jest, ())
}

fn jest(cx: &mut Scope, _: &()) -> Element {
    let start_time = cx.use_state(Instant::now);
    let completed = cx.use_state(Vec::<TestResult>::new);
    let running = cx.use_state(Vec::<TestResult>::new);
    cx.use_future((), move || async move {
        let index = Cell::new(0);
        let random = Cell::new(RandomState::new().hash_one(()).max(1));
        let next_random = || {
            let mut value = random.get();
            value ^= value << 13;
            value ^= value >> 7;
            value ^= value << 17;
            random.set(value);
            value
        };
        let worker = || async {
            while let Some(&path) = PATHS.get(index.get()) {
                index.set(index.get() + 1);
                running.update(|results| {
                    results.push(TestResult {
                        path,
                        status: "runs",
                    })
                });
                tokio::time::sleep(Duration::from_millis(next_random() % 1000)).await;
                running.update(|results| results.retain(|result| result.path != path));
                completed.update(|results| {
                    results.push(TestResult {
                        path,
                        status: if next_random() & 1 == 0 {
                            "pass"
                        } else {
                            "fail"
                        },
                    });
                });
            }
        };
        tokio::join!(worker(), worker(), worker(), worker());
    });
    let completed = completed.get();
    let running = running.get();
    let passed = completed
        .iter()
        .filter(|result| result.status == "pass")
        .count();
    let failed = completed.len() - passed;
    let finished = completed.len() == PATHS.len();
    rsx! {
        View(flex_direction: FlexDirection::Column) {
            Static(completed, |result, _| test(result).key(result.path))
            if !running.is_empty() {
                View(flex_direction: FlexDirection::Column, margin_top: 1) {
                    for result in &running {
                        {test(result).key(result.path)}
                    }
                }
            }
            {summary(
                finished,
                passed,
                failed,
                short_time(start_time.get().elapsed()),
            )}
        }
    }
    .into()
}

fn short_time(elapsed: Duration) -> String {
    let millis = elapsed.as_millis();
    for (unit, suffix) in [
        (86_400_000, "d"),
        (3_600_000, "h"),
        (60_000, "m"),
        (1000, "s"),
    ] {
        if millis >= unit {
            return format!("{}{suffix}", (millis + unit / 2) / unit);
        }
    }
    format!("{millis}ms")
}

fn summary(finished: bool, passed: usize, failed: usize, time: String) -> Element {
    rsx! {
        View(flex_direction: FlexDirection::Column, margin_top: 1) {
            View {
                View(width: 14) {
                    Text(bold: true) { "Test Suites:" }
                }
                if failed > 0 {
                    Text(bold: true, color: Color::Red) { "{failed} failed, " }
                }
                if passed > 0 {
                    Text(bold: true, color: Color::Green) { "{passed} passed, " }
                }
                Text { {format!("{} total", passed + failed)} }
            }
            View {
                View(width: 14) {
                    Text(bold: true) { "Time:" }
                }
                Text { {time} }
            }
            if finished {
                View {
                    Text(dim_color: true) { "Ran all test suites." }
                }
            }
        }
    }
    .into()
}

fn test(result: &TestResult) -> Element {
    let background = match result.status {
        "runs" => Color::Yellow,
        "pass" => Color::Green,
        "fail" => Color::Red,
        _ => unreachable!("test status"),
    };
    let (directory, filename) = result.path.split_once('/').expect("test path");
    rsx! {
        View {
            Text(color: Color::Black, background_color: background) {
                {format!(" {} ", result.status.to_uppercase())}
            }
            View(margin_left: 1) {
                Text(dim_color: true) { "{directory}/" }
                Text(bold: true, color: Color::White) { {filename} }
            }
        }
    }
    .into()
}
