# ink-rs

Build terminal user interfaces from components, in the spirit of
[Ink](https://github.com/vadimdemedes/ink). Layout is flexbox (through
[taffy](https://github.com/DioxusLabs/taffy)), output is diffed cell by cell,
and Ink's test suite serves as the specification.

Not related to [ink!](https://use.ink) smart contracts.

```rust
use std::time::Duration;

use ink_rs::*;

fn counter(cx: &mut Scope, _: &()) -> Element {
    let count = cx
        .use_animation(AnimationOptions {
            interval: Duration::from_millis(100),
            active: true,
        })
        .frame;
    rsx! {
        Text(color: Color::Green) { "{count} tests passed" }
    }
    .into()
}

fn main() -> Result<(), Error> {
    render(component(counter, ()), Options::default())
        .wait_until_exit()
        .map(drop)
}
```

## Features

- **Components:** `View` (Ink's `Box`), `Text`, `Static`, `Transform`, `Newline`
  and `Spacer`. A component is a function `fn(&mut Scope, &Props) -> Element`;
  props can hold children (`Element`) and callbacks (`Callback`).
- **Hooks:** `use_state`, `use_memo`, `use_callback`, `use_effect`,
  `use_layout_effect`, `provide`, `use_context`, `use_input`, `use_paste`,
  `use_focus`, `use_focus_manager`, `use_app`, `use_stdout`, `use_stderr`,
  `use_window_size`, `use_box_metrics`, `use_node_ref`, `use_cursor`,
  `use_animation` and, with tokio, `use_future`.
- **Two ways to build trees:** the `rsx!` macro, or the builder API it expands to.
- **Rendering:** interactive output with a throttle, non-interactive output for
  CI and pipes, the alternate screen, Ctrl+Z suspension, resize handling, and an
  error screen for panics.
- **Testing:** `render_to_string` renders one frame; the `testing` module mounts
  an app on an in-memory terminal with a clock you advance.

ink-rs also has a few things Ink does not:

- a tokio driver, `ink_rs::tokio::render`, and `Scope::use_future`;
- `write_control` for bytes that should bypass the frame, such as kitty images,
  window titles or OSC 52 clipboard writes;
- a cell diff instead of line rewrites, and a live frame capped at the screen
  height so cut rows never leak into the scrollback.

Ink's screen reader mode is not ported yet.

## Cargo features

| Feature  | Default | Provides                                   |
|----------|---------|--------------------------------------------|
| `macros` | yes     | `rsx!` and `#[derive(Props)]`              |
| `tokio`  | yes     | `ink_rs::tokio` and `Scope::use_future`    |

## Examples

`examples/` ports 25 of Ink's examples under the same names:

```sh
cargo run --example use-input
```

## Platform support

Linux, macOS and Windows 10 1809 (build 17763) or later. The minimum supported Rust
version is 1.88.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

The tests port Ink's tests and their expected output. Ink is copyright Vadym
Demedes and Sindre Sorhus and licensed under the MIT license; see
[LICENSE-INK](LICENSE-INK).
