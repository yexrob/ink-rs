// Ink examples/subprocess-output.

mod app;

use std::io::Read;
use std::process::{Command, Stdio};

use app::Chunk;
use ink_rs::Options;

#[tokio::main(flavor = "local")]
async fn main() -> Result<(), ink_rs::Error> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    std::thread::spawn(move || {
        let mut child = match Command::new("cargo")
            .args(["run", "--quiet", "--example", "jest"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                let _ = sender.send(Chunk::Failed(error.to_string()));
                return;
            }
        };
        let mut stdout = child.stdout.take().expect("piped stdout");
        let mut buffer = [0; 4096];
        loop {
            match stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if sender.send(Chunk::Bytes(buffer[..n].to_vec())).is_err() {
                        let _ = child.kill();
                        break;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        let _ = child.wait();
    });
    ink_rs::tokio::render(app::app(receiver), Options::default())
        .wait_until_exit()
        .await
        .map(|_| ())
}
