// Ink examples/alternate-screen.

mod app;

use ink_rs::{Options, render};

fn main() -> Result<(), ink_rs::Error> {
    render(
        app::app(),
        Options {
            alternate_screen: true,
            ..Options::default()
        },
    )
    .wait_until_exit()
    .map(|_| ())
}
