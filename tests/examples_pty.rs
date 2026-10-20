//! Ink 8.0.0 test/render.tsx:370 (resize), :453 (suspend), commit 26d2c3f.
//! One executable serves as harness and fixture so Cargo builds both together.

use std::thread;
use std::time::Duration;

use ink_rs::{Options, render};

#[path = "common/pty.rs"]
mod session;
#[path = "../examples/suspend-terminal/app.rs"]
mod suspend_terminal;
#[path = "../examples/terminal-resize/app.rs"]
mod terminal_resize;
use session::Session;

const FIXTURE: &str = "INK_RS_EXAMPLE_FIXTURE";

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy)]
enum Case {
    Resize,
    Suspend(&'static str, &'static str),
}

const CASES: &[(Case, &str, &str)] = &[
    (
        Case::Resize,
        "resize",
        "terminal resize example stays open to report resized dimensions",
    ),
    (
        Case::Suspend("false", "child failed:"),
        "false",
        "suspend example reports the result of false",
    ),
    (
        Case::Suspend("true", "resumed"),
        "true",
        "suspend example reports the result of true",
    ),
    (
        Case::Suspend("printf editor-command-ran", "resumed"),
        "printf",
        "suspend example reports the result of printf editor-command-ran",
    ),
];

fn fixture(case: Case) -> Result {
    let element = match case {
        Case::Resize => terminal_resize::app(),
        Case::Suspend(..) => suspend_terminal::app(),
    };
    render(element, Options::default()).wait_until_exit()?;
    Ok(())
}

fn wait_screen(
    session: &mut Session,
    parser: &mut vt100::Parser,
    consumed: &mut usize,
    texts: &[&str],
) -> Result {
    session
        .wait_for(|output| {
            parser.process(&output[*consumed..]);
            *consumed = output.len();
            let contents = parser.screen().contents();
            texts.iter().all(|text| contents.contains(text))
        })
        .map_err(|error| {
            format!(
                "waiting for {texts:?}: {error}; screen: {:?}",
                parser.screen().contents()
            )
            .into()
        })
}

fn run_pty(case: Case, id: &str) -> Result {
    let executable = std::env::current_exe()?;
    let mut env = vec![(FIXTURE, id), ("FORCE_COLOR", "0")];
    if let Case::Suspend(editor, _) = case {
        env.push(("EDITOR", editor));
    }
    let mut session = Session::spawn(executable.as_os_str(), &[], &env, 24, 100)?;
    let mut parser = vt100::Parser::new(24, 100, 0);
    let mut consumed = 0;
    match case {
        Case::Resize => {
            wait_screen(
                &mut session,
                &mut parser,
                &mut consumed,
                &["Columns: 100", "Rows: 24"],
            )?;
            thread::sleep(Duration::from_millis(100));
            if !session.is_running()? {
                return Err("resize example exited before resize".into());
            }
            session.resize(20, 60)?;
            parser.screen_mut().set_size(20, 60);
            wait_screen(
                &mut session,
                &mut parser,
                &mut consumed,
                &["Columns: 60", "Rows: 20"],
            )?;
            session.send(b"\x03")?;
        }
        Case::Suspend(editor, status) => {
            wait_screen(&mut session, &mut parser, &mut consumed, &["ready"])?;
            let start = session.output().len();
            session.send(b"e")?;
            wait_screen(&mut session, &mut parser, &mut consumed, &[status])?;
            if editor.starts_with("printf ")
                && !String::from_utf8_lossy(&session.output()[start..])
                    .contains("editor-command-ran")
            {
                return Err("missing editor output".into());
            }
            session.send(b"+")?;
            wait_screen(
                &mut session,
                &mut parser,
                &mut consumed,
                &["Counter: 1", status],
            )?;
            session.send(b"q")?;
        }
    }
    let status = session.wait_exit()?;
    if !status.success() {
        return Err(format!(
            "child status {status:?}; output: {:?}",
            String::from_utf8_lossy(session.output())
        )
        .into());
    }
    Ok(())
}

fn main() {
    if let Ok(id) = std::env::var(FIXTURE) {
        let result = CASES
            .iter()
            .find(|(_, key, _)| *key == id)
            .ok_or_else(|| "unknown fixture".into())
            .and_then(|(case, _, _)| fixture(*case));
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    let mut failed = 0;
    for &(case, id, name) in CASES {
        match run_pty(case, id) {
            Ok(()) => println!("ok - {name}"),
            Err(error) => {
                failed += 1;
                eprintln!("FAILED - {name}: {error}");
            }
        }
    }
    if failed > 0 {
        std::process::exit(1);
    }
}
