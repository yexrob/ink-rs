use super::width_data as data;

fn contains(ranges: &[(u32, u32)], c: char) -> bool {
    let cp = u32::from(c);
    let index = ranges.partition_point(|&(_, end)| end < cp);
    ranges.get(index).is_some_and(|&(start, _)| start <= cp)
}

pub(crate) fn is_mark(c: char) -> bool {
    contains(data::MARK, c)
}

fn visible(c: char) -> bool {
    !contains(data::INVISIBLE, c)
}

fn east_asian_width(c: char) -> usize {
    // get-east-asian-width: W/F are wide; ambiguous characters are narrow.
    if contains(data::WIDE, c) { 2 } else { 1 }
}

fn leading(c: char) -> bool {
    matches!(c, '\u{1100}'..='\u{115f}' | '\u{a960}'..='\u{a97c}')
}

fn vowel(c: char) -> bool {
    matches!(c, '\u{1160}'..='\u{11a7}' | '\u{d7b0}'..='\u{d7c6}')
}

fn trailing(c: char) -> bool {
    matches!(c, '\u{11a8}'..='\u{11ff}' | '\u{d7cb}'..='\u{d7fb}')
}

fn emoji(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let second = chars.next();
    let third = chars.next();
    match (second, third) {
        (None, _) => return contains(data::EMOJI, first),
        (Some('\u{fe0f}'), None) if contains(data::EMOJI_VARIATION, first) => return true,
        (Some('\u{1f3fb}'..='\u{1f3ff}'), None) if contains(data::EMOJI_MODIFIER_BASE, first) => {
            return true;
        }
        (Some(last @ '\u{1f1e6}'..='\u{1f1ff}'), None)
            if matches!(first, '\u{1f1e6}'..='\u{1f1ff}') =>
        {
            let flag = (u32::from(first) - 0x1f1e6) * 26 + u32::from(last) - 0x1f1e6;
            return contains(data::FLAGS, char::from_u32(flag).unwrap());
        }
        _ => {}
    }
    if matches!(first, '0'..='9' | '#' | '*')
        && (segment.ends_with('\u{20e3}')
            && (second == Some('\u{20e3}') && third.is_none()
                || second == Some('\u{fe0f}')
                    && third == Some('\u{20e3}')
                    && chars.next().is_none()))
    {
        return true;
    }
    if data::EMOJI_TAGS.contains(&segment) {
        return true;
    }
    segment.encode_utf16().count() <= 50
        && segment.contains('\u{200d}')
        && segment
            .chars()
            .filter(|&c| contains(data::PICTOGRAPHIC, c))
            .take(2)
            .count()
            == 2
}

/// Width of one grapheme, following string-width 8.3 rather than script ligatures.
pub(crate) fn grapheme_width(segment: &str) -> usize {
    let Some((base, first)) = segment.char_indices().find(|&(_, c)| visible(c)) else {
        return 0;
    };
    if emoji(segment) {
        return 2;
    }
    if leading(first) || vowel(first) || trailing(first) {
        let mut chars = segment[base..].chars().filter(|&c| visible(c)).peekable();
        let mut width = 0;
        while let Some(c) = chars.next() {
            if !leading(c) && !vowel(c) && !trailing(c) {
                // string-width stops shaping after the first non-jamo scalar.
                return width + east_asian_width(c) + chars.map(east_asian_width).sum::<usize>();
            }
            if leading(c) && chars.peek().is_some_and(|&c| vowel(c)) {
                chars.next();
                if chars.peek().is_some_and(|&c| trailing(c)) {
                    chars.next();
                }
                width += 2;
            } else {
                width += east_asian_width(c);
            }
        }
        return width;
    }
    east_asian_width(first)
        + segment[base..]
            .chars()
            .skip(1)
            .filter(|&c| contains(data::SPACING_MARK, c) || ('\u{ff00}'..='\u{ffef}').contains(&c))
            .map(east_asian_width)
            .sum::<usize>()
}

/// Ink output.ts drops these even when they form a standalone grapheme.
pub(crate) fn is_default_ignorable(c: char) -> bool {
    matches!(c as u32,
        0x00ad | 0x034f | 0x061c | 0x115f..=0x1160 | 0x17b4..=0x17b5 |
        0x180b..=0x180f | 0x200b..=0x200f | 0x202a..=0x202e |
        0x2060..=0x206f | 0x3164 | 0xfe00..=0xfe0f | 0xfeff |
        0xffa0 | 0xfff0..=0xfff8 | 0x1bca0..=0x1bca3 |
        0x1d173..=0x1d17a | 0xe0000..=0xe0fff)
}
