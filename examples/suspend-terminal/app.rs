// Ink examples/suspend-terminal.

use std::ffi::OsStr;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};

use ink_rs::{BorderStyle, Color, Element, FlexDirection, Scope, Styled, Text, View, component};

fn run_child(command: &str, args: &[&OsStr]) -> Result<(), String> {
    let status = Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else if let Some(signal) = status.signal() {
        let name = signal_hook::low_level::signal_name(signal)
            .map(str::to_owned)
            .unwrap_or_else(|| signal.to_string());
        Err(format!("Child process terminated by signal {name}"))
    } else {
        Err(format!(
            "Child process exited with code {}",
            status.code().expect("child exit code")
        ))
    }
}

pub fn app() -> Element {
    component(example, ())
}

fn example(cx: &mut Scope, _: &()) -> Element {
    let app = cx.use_app();
    let counter = cx.use_state(|| 0);
    let status = cx.use_state(|| String::from("ready"));
    cx.use_input(true, move |input, _| {
        if input == "q" {
            app.exit();
            return;
        }
        if input == "+" {
            counter.update(|value| *value += 1);
            return;
        }
        if input == "e" || input == "r" {
            status.set("suspended \u{2014} child owns the terminal".into());
            let editor = input == "e";
            let handle = app.clone();
            let _ = app.suspend_terminal(move || {
                let result = if editor {
                    let editor = std::env::var_os("EDITOR").unwrap_or_else(|| "vi".into());
                    run_child("sh", &[OsStr::new("-c"), &editor])
                } else {
                    run_child("sh", &[
                        OsStr::new("-c"),
                        OsStr::new(r#"printf "Child process owns the terminal.\nType something and press Enter: "; read -r line; printf "You typed: %s\n" "$line""#),
                    ])
                };
                handle.update(move || {
                    status.set(match result {
                        Ok(()) => "resumed \u{2014} Ink redrew and the counter is preserved".into(),
                        Err(message) => format!("child failed: {message}"),
                    });
                });
            });
        }
    });
    View::new()
        .flex_direction(FlexDirection::Column)
        .border_style(BorderStyle::Round)
        .padding_x(1)
        .child(Text::new("suspendTerminal() demo").bold(true))
        .child(
            Text::new("Counter: ").child(Text::new(counter.get().to_string()).color(Color::Green)),
        )
        .child(Text::new(status.get()).dim_color(true))
        .child(
            View::new()
                .margin_top(1)
                .flex_direction(FlexDirection::Column)
                .child(Text::new("e \u{2014} open $EDITOR (defaults to vi)"))
                .child(Text::new("r \u{2014} run a shell read prompt"))
                .child(Text::new("+ \u{2014} increment the counter"))
                .child(Text::new("q \u{2014} quit")),
        )
        .into()
}
