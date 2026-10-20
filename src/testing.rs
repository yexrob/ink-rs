//! A mounted tree with frame history, driven by calls to [`TestInstance::settle`].

use std::time::Instant;

use crate::{ColorLevel, Element, RenderOptions, paint::serialize_row, runtime::Runtime};

/// Mounts at 100 columns with basic colors, matching Ink's debug test helper.
pub fn render(element: impl Into<Element>) -> TestInstance {
    render_with(
        element,
        RenderOptions {
            columns: 100,
            color_level: ColorLevel::Basic,
        },
    )
}

/// Mounts with the given options and presents one frame.
/// Passive-effect state updates remain pending until [`TestInstance::settle`].
pub fn render_with(element: impl Into<Element>, options: RenderOptions) -> TestInstance {
    let mut instance = TestInstance {
        runtime: Runtime::new(element.into(), options),
        frames: Vec::new(),
        static_output: String::new(),
    };
    instance.step();
    instance
}

/// A tree that retains state across frames. Dropping it runs unmount cleanups.
pub struct TestInstance {
    runtime: Runtime,
    frames: Vec<String>,
    static_output: String,
}

impl TestInstance {
    /// The last presented frame: accumulated Static rows, each followed by a
    /// newline, then live rows joined by newlines, matching Ink's debug output.
    /// Static history resets when the first Static instance changes.
    pub fn last_frame(&self) -> &str {
        self.frames.last().map_or("", String::as_str)
    }

    /// One debug-output frame per present, including unchanged output. Each
    /// contains accumulated Static rows and their newlines before the live rows.
    /// Static history resets when the first Static instance changes.
    pub fn frames(&self) -> &[String] {
        &self.frames
    }

    /// Reconciles a new root against the mounted tree and runs one pass.
    pub fn rerender(&mut self, element: impl Into<Element>) {
        self.runtime.rerender(element.into());
        self.step();
    }

    /// Runs pending state updates and effects until the runtime is idle.
    pub fn settle(&mut self) {
        while self.step().is_some() {}
    }

    /// Unmounts the tree and runs effect cleanups.
    pub fn unmount(self) {
        drop(self);
    }

    fn step(&mut self) -> Option<Instant> {
        self.runtime.render_if_due(Instant::now(), |frame| {
            // Ink handleStaticChange resets history before onRender appends it.
            if frame.static_changed {
                self.static_output.clear();
            }
            for row in &frame.static_lines {
                self.static_output.push_str(&serialize_row(row));
                // renderer.ts preserves the newline even for a blank Static row.
                self.static_output.push('\n');
            }
            let live = frame
                .live
                .iter()
                .map(|row| serialize_row(row))
                .collect::<Vec<_>>()
                .join("\n");
            self.frames.push(format!("{}{live}", self.static_output));
        })
    }
}
