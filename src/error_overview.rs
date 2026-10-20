use std::fs;
use std::path::Path;

use crate::{Color, Element, FlexDirection, Styled, Text, View};

#[cfg(test)]
mod tests;

// Ink 8.0.0 ErrorOverview.tsx: a host-only tree keeps the error display out of
// user component code. Message and stack are separate in Rust.
pub(crate) fn build(
    message: &str,
    location: Option<(&str, u32, u32)>,
    backtrace: Option<&str>,
) -> Element {
    let mut root = View::new()
        .flex_direction(FlexDirection::Column)
        .padding(1)
        .child(
            View::new()
                .child(
                    Text::new(" ERROR ")
                        .background_color(Color::Red)
                        .color(Color::White),
                )
                .child(Text::new(format!(" {message}"))),
        );
    if let Some((file, line, column)) = location {
        root =
            root.child(View::new().margin_top(1).child(
                Text::new(format!("{}:{line}:{column}", display_path(file))).dim_color(true),
            ));
        if let Ok(source) = fs::read_to_string(file) {
            let lines = excerpt(&source, line);
            if let Some((last, _)) = lines.last() {
                let width = last.to_string().len();
                let mut block = View::new()
                    .margin_top(1)
                    .flex_direction(FlexDirection::Column);
                for (number, value) in lines {
                    let mut label = Text::new(format!("{number:>width$}:"));
                    let mut content = Text::new(format!(" {value}"));
                    if number == line {
                        label = label.background_color(Color::Red).color(Color::White);
                        content = content.background_color(Color::Red).color(Color::White);
                    } else {
                        label = label.dim_color(true);
                    }
                    block = block.child(
                        View::new()
                            .child(View::new().width((width + 1) as u16).child(label))
                            .child(content),
                    );
                }
                root = root.child(block);
            }
        }
    }
    if let Some(backtrace) = backtrace {
        let frames = frames(backtrace);
        if !frames.is_empty() {
            let mut stack = View::new()
                .margin_top(1)
                .flex_direction(FlexDirection::Column);
            for (symbol, location) in frames {
                let mut row = Text::default()
                    .child(Text::new("- ").dim_color(true))
                    .child(Text::new(symbol).dim_color(true).bold(true));
                if let Some(location) = location {
                    row = row.child(
                        Text::new(format!(" ({})", display_path(&location)))
                            .dim_color(true)
                            .color(Color::Gray),
                    );
                }
                stack = stack.child(row);
            }
            root = root.child(stack);
        }
    }
    root.into()
}

fn display_path(file: &str) -> String {
    let path = Path::new(file);
    std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            path.strip_prefix(cwd)
                .ok()
                .map(|path| path.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| file.to_owned())
}

fn excerpt(source: &str, line: u32) -> Vec<(u32, String)> {
    let lines: Vec<_> = source.split('\n').collect();
    let line = line as usize;
    if line == 0 || line > lines.len() {
        return Vec::new();
    }
    let start = line.saturating_sub(4);
    lines[start..(line + 3).min(lines.len())]
        .iter()
        .enumerate()
        .map(|(offset, value)| {
            let value = if start + offset + 1 < lines.len() {
                value.strip_suffix('\r').unwrap_or(value)
            } else {
                value
            };
            let tabs = value.bytes().take_while(|byte| *byte == b'\t').count();
            (
                (start + offset + 1) as u32,
                format!("{}{}", "  ".repeat(tabs), &value[tabs..]),
            )
        })
        .collect()
}

fn frames(backtrace: &str) -> Vec<(String, Option<String>)> {
    let mut frames: Vec<(String, Option<String>)> = Vec::new();
    for line in backtrace
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if let Some(location) = line.strip_prefix("at ")
            && source_location(location)
            && let Some(frame) = frames.last_mut()
        {
            frame.1 = Some(location.to_owned());
            continue;
        }
        let symbol = line
            .split_once(':')
            .and_then(|(number, symbol)| number.parse::<usize>().ok().map(|_| symbol.trim()))
            .unwrap_or(line);
        frames.push((symbol.to_owned(), None));
    }
    frames.retain(|(symbol, _)| {
        let symbol = symbol.trim_start_matches('<');
        ![
            "std::",
            "core::",
            "alloc::",
            "panic_unwind::",
            "__rustc::",
            "ink_rs::runtime::panic::install",
            "ink_rs::driver::process::install_panic_hook",
        ]
        .iter()
        .any(|prefix| symbol.starts_with(prefix))
    });
    // Keep API frames above the component, but stop at its runtime caller.
    let mut user_frame = false;
    let end = frames.iter().position(|(symbol, _)| {
        if symbol.trim_start_matches('<').starts_with("ink_rs::") {
            user_frame
        } else {
            user_frame = true;
            false
        }
    });
    if let Some(end) = end {
        frames.truncate(end);
    }
    frames
}

fn source_location(location: &str) -> bool {
    let mut parts = location.rsplitn(3, ':');
    let column = parts.next().and_then(|value| value.parse::<u32>().ok());
    let line = parts.next().and_then(|value| value.parse::<u32>().ok());
    column.is_some_and(|column| column > 0)
        && line.is_some_and(|line| line > 0)
        && parts.next().is_some_and(|file| !file.is_empty())
}
