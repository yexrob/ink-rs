use taffy::{NodeId, Point};

use crate::components::TextStyle;
use crate::host::{HostTree, LineTransform, text_style};
use crate::style::BorderSide;
use crate::text::measure::measure_text;
use crate::text::styled::{self, CharStyle, StyledChar, StyledLine, StyledText};
use crate::text::width::{grapheme_width, is_default_ignorable};
use crate::text::wrap::wrap_text;
use crate::{ColorLevel, Display, Overflow, Style};

/// Continuations belong to the preceding glyph, not to an empty string cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Cell {
    Glyph(StyledChar),
    Continuation(CharStyle),
}

impl Cell {
    fn style(&self) -> &CharStyle {
        match self {
            Self::Glyph(cell) => &cell.style,
            Self::Continuation(style) => style,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Frame {
    pub(crate) live: Vec<Vec<Cell>>,
    pub(crate) static_lines: Vec<Vec<Cell>>,
    /// The first Static instance differs from the one at the previous present.
    pub(crate) static_changed: bool,
}

impl Frame {
    pub(crate) fn serialize(&self) -> String {
        self.static_lines
            .iter()
            .chain(&self.live)
            .map(|row| serialize_row(row))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub(crate) fn serialize_row(row: &[Cell]) -> String {
    styled::serialize(&row_to_line(row))
}

fn row_to_line(row: &[Cell]) -> StyledLine {
    // Ink src/output.ts uses trimEnd(): styled blanks retain their escape codes.
    let end = row
        .iter()
        .rposition(|cell| {
            !matches!(cell, Cell::Glyph(c) if c.grapheme == " " && c.style == CharStyle::default())
        })
        .map_or(0, |index| index + 1);
    row[..end]
        .iter()
        .filter_map(|cell| match cell {
            Cell::Glyph(cell) => Some(cell.clone()),
            Cell::Continuation(_) => None,
        })
        .collect()
}

fn blank(style: CharStyle) -> StyledChar {
    StyledChar {
        grapheme: " ".to_owned(),
        width: 1,
        style,
    }
}

fn grid(width: f32, height: f32) -> Vec<Vec<Cell>> {
    vec![
        vec![Cell::Glyph(blank(CharStyle::default())); width.max(0.0) as usize];
        height.max(0.0) as usize
    ]
}

#[derive(Clone, Copy, Default)]
struct Clip {
    x: Option<(i32, i32)>,
    y: Option<(i32, i32)>,
}

impl Clip {
    fn intersect(self, other: Self) -> Self {
        fn axis(a: Option<(i32, i32)>, b: Option<(i32, i32)>) -> Option<(i32, i32)> {
            match (a, b) {
                (Some((a1, a2)), Some((b1, b2))) => Some((a1.max(b1), a2.min(b2))),
                _ => a.or(b),
            }
        }
        Self {
            x: axis(self.x, other.x),
            y: axis(self.y, other.y),
        }
    }

    fn empty(self) -> bool {
        [self.x, self.y]
            .into_iter()
            .flatten()
            .any(|(start, end)| start >= end)
    }
}

fn slice(line: &StyledLine, from: i32, to: i32) -> StyledLine {
    let mut result = Vec::new();
    let mut column = 0;
    for cell in line {
        if cell.grapheme.chars().all(is_default_ignorable) {
            continue;
        }
        let end = column + cell.width.max(1) as i32;
        let visible = end.min(to) - column.max(from);
        if visible > 0 {
            if column < from || end > to {
                result.extend((0..visible).map(|_| blank(cell.style.clone())));
            } else {
                result.push(cell.clone());
            }
        }
        column = end;
    }
    result
}

/// Clear every cell of an old glyph when a write touches one of its columns.
fn clear_glyph(row: &mut [Cell], column: usize) {
    if column >= row.len() {
        return;
    }
    let mut start = column;
    while start > 0 && matches!(row[start], Cell::Continuation(_)) {
        start -= 1;
    }
    let width = match &row[start] {
        Cell::Glyph(cell) => cell.width.max(1),
        Cell::Continuation(_) => 1,
    };
    for index in start..(start + width).min(row.len()) {
        row[index] = Cell::Glyph(blank(row[index].style().clone()));
    }
}

fn write_row(row: &mut Vec<Cell>, mut x: i32, line: &StyledLine) {
    for cell in line {
        if cell.grapheme.chars().all(is_default_ignorable) {
            continue;
        }
        let width = cell.width.max(1) as i32;
        let end = x + width;
        if end > 0 {
            row.resize(
                row.len().max(end as usize),
                Cell::Glyph(blank(CharStyle::default())),
            );
            for column in x.max(0)..end {
                clear_glyph(row, column as usize);
            }
            for column in x.max(0)..end {
                row[column as usize] = if x < 0 {
                    Cell::Glyph(blank(cell.style.clone()))
                } else if column == x {
                    let mut glyph = cell.clone();
                    glyph.width = width as usize;
                    Cell::Glyph(glyph)
                } else {
                    Cell::Continuation(cell.style.clone())
                };
            }
        }
        x = end;
    }
}

fn write(
    output: &mut [Vec<Cell>],
    origin: Point<i32>,
    lines: &StyledText,
    clip: Clip,
    transform: Option<&LineTransform>,
) {
    if clip.empty() {
        return;
    }
    for (index, line) in lines.iter().enumerate() {
        let y = origin.y + index as i32;
        if y < 0 || clip.y.is_some_and(|(start, end)| y < start || y >= end) {
            continue;
        }
        let Some(row) = output.get_mut(y as usize) else {
            continue;
        };
        let (x, line) = if let Some((start, end)) = clip.x {
            (
                origin.x.max(start),
                slice(line, (start - origin.x).max(0), end - origin.x),
            )
        } else {
            (origin.x, line.clone())
        };
        // Ink output.ts transforms after clipping, with the original line index.
        let transformed = transform.map_or_else(
            || vec![line.clone()],
            |transform| transform.apply(&line, index),
        );
        let line = transformed.into_iter().flatten().collect();
        write_row(row, x, &line);
    }
}

pub(crate) fn paint(tree: &HostTree, root: NodeId, level: ColorLevel) -> Frame {
    let (layout, _) = crate::layout::get(tree, root, Point::ZERO);
    let mut frame = Frame {
        live: grid(layout.size.width, layout.size.height),
        static_lines: Vec::new(),
        static_changed: false,
    };
    walk(
        tree,
        root,
        Offset::default(),
        Clip::default(),
        &mut frame.live,
        level,
        true,
    );
    // renderer.ts selects the first Static in tree order, then checks ancestors.
    if let Some((node, parent_absolute)) = find_static(tree, root, Point::ZERO) {
        let mut ancestor = Some(node);
        while let Some(id) = ancestor {
            if tree.props(id).style.display == Display::None {
                return frame;
            }
            ancestor = tree.parent(id);
        }
        let (layout, _) = crate::layout::get(tree, node, parent_absolute);
        frame.static_lines = grid(
            layout.location.x + layout.size.width + layout.margin.right,
            layout.location.y + layout.size.height + layout.margin.bottom,
        );
        walk(
            tree,
            node,
            Offset {
                layout: parent_absolute,
                ..Offset::default()
            },
            Clip::default(),
            &mut frame.static_lines,
            level,
            false,
        );
    }
    frame
}

fn find_static(
    tree: &HostTree,
    node: NodeId,
    parent_absolute: Point<f32>,
) -> Option<(NodeId, Point<f32>)> {
    if tree.props(node).is_static {
        return Some((node, parent_absolute));
    }
    let location = tree.unrounded_layout(node).location;
    let absolute = Point {
        x: parent_absolute.x + location.x,
        y: parent_absolute.y + location.y,
    };
    tree.children(node)
        .iter()
        .copied()
        .find_map(|child| find_static(tree, child, absolute))
}

#[derive(Clone, Copy, Default)]
struct Offset {
    // Ink src/render-node-to-output.ts scrolls paint coordinates, not layout.
    paint: Point<i32>,
    layout: Point<f32>,
}

fn walk(
    tree: &HostTree,
    node: NodeId,
    offset: Offset,
    clip: Clip,
    output: &mut [Vec<Cell>],
    level: ColorLevel,
    skip_static: bool,
) {
    let props = tree.props(node);
    let style = &props.style;
    if style.display == Display::None || (skip_static && props.is_static) {
        return;
    }
    let (layout, absolute) = crate::layout::get(tree, node, offset.layout);
    let origin = Point {
        x: offset.paint.x + layout.location.x as i32,
        y: offset.paint.y + layout.location.y as i32,
    };
    if let Some(text) = &props.text {
        let mut lines = text.squash();
        let width = (layout.size.width
            - layout.padding.left
            - layout.padding.right
            - layout.border.left
            - layout.border.right)
            .max(0.0) as usize;
        if measure_text(&lines).width > width {
            lines = wrap_text(&lines, width, text.wrap);
        }
        write(output, origin, &lines, clip, Some(&text.transform));
        return;
    }
    decoration(
        output,
        origin,
        layout.size.width as i32,
        layout.size.height as i32,
        style,
        level,
        clip,
    );
    let clip = clip.intersect(Clip {
        x: (style.overflow_x.or(style.overflow) == Some(Overflow::Hidden)).then_some((
            origin.x + layout.border.left as i32,
            origin.x + layout.size.width as i32 - layout.border.right as i32,
        )),
        y: (style.overflow_y.or(style.overflow) == Some(Overflow::Hidden)).then_some((
            origin.y + layout.border.top as i32,
            origin.y + layout.size.height as i32 - layout.border.bottom as i32,
        )),
    });
    let offset = Offset {
        paint: Point {
            x: origin.x - i32::from(style.content_offset_x),
            y: origin.y - i32::from(style.content_offset_y),
        },
        layout: absolute,
    };
    for &child in tree.children(node) {
        walk(tree, child, offset, clip, output, level, skip_static);
    }
}

fn border_style(style: &Style, side: BorderSide, level: ColorLevel) -> CharStyle {
    text_style(
        TextStyle {
            color: side.color.or(style.border_color),
            background_color: side.background_color.or(style.border_background_color),
            dim_color: side.dim_color.unwrap_or(style.border_dim_color),
            ..TextStyle::default()
        },
        level,
    )
}

fn glyph(value: char, style: &CharStyle) -> StyledChar {
    let grapheme = value.to_string();
    StyledChar {
        width: grapheme_width(&grapheme),
        grapheme,
        style: style.clone(),
    }
}

fn decoration(
    output: &mut [Vec<Cell>],
    origin: Point<i32>,
    width: i32,
    height: i32,
    style: &Style,
    level: ColorLevel,
    clip: Clip,
) {
    let bordered = style.border_style.is_some();
    let left = i32::from(bordered && style.border.left.visible);
    let right = i32::from(bordered && style.border.right.visible);
    let top = i32::from(bordered && style.border.top.visible);
    let bottom = i32::from(bordered && style.border.bottom.visible);
    let content_width = (width - left - right).max(0) as usize;
    if let Some(color) = style.background_color {
        let background = CharStyle {
            bg: color.at_level(level),
            ..CharStyle::default()
        };
        let line = vec![blank(background); content_width];
        for y in top..height - bottom {
            write(
                output,
                Point {
                    x: origin.x + left,
                    y: origin.y + y,
                },
                &vec![line.clone()],
                clip,
                None,
            );
        }
    }
    let Some(border) = style.border_style else {
        return;
    };
    let chars = border.chars();
    for (visible, y, start, middle, end, side) in [
        (
            top,
            0,
            chars.top_left,
            chars.top,
            chars.top_right,
            style.border.top,
        ),
        (
            bottom,
            height - 1,
            chars.bottom_left,
            chars.bottom,
            chars.bottom_right,
            style.border.bottom,
        ),
    ] {
        if visible == 0 {
            continue;
        }
        let style = border_style(style, side, level);
        let mut line = Vec::new();
        if left > 0 {
            line.push(glyph(start, &style));
        }
        line.extend((0..content_width).map(|_| glyph(middle, &style)));
        if right > 0 {
            line.push(glyph(end, &style));
        }
        write(
            output,
            Point {
                x: origin.x,
                y: origin.y + y,
            },
            &vec![line],
            clip,
            None,
        );
    }
    for (visible, x, value, side) in [
        (left, 0, chars.left, style.border.left),
        (right, width - 1, chars.right, style.border.right),
    ] {
        if visible == 0 {
            continue;
        }
        let style = border_style(style, side, level);
        let lines = vec![vec![glyph(value, &style)]; (height - top - bottom).max(0) as usize];
        write(
            output,
            Point {
                x: origin.x + x,
                y: origin.y + top,
            },
            &lines,
            clip,
            None,
        );
    }
}
