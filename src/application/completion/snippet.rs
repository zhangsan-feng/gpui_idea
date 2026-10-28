use std::{iter::Peekable, ops::Range, str::Chars};

// Expand placeholder defaults and retain the first editable selection.
// Reject variables/transforms that cannot be resolved by this editor.
pub(super) fn parse(value: &str) -> Option<(String, Option<Range<usize>>)> {
    if value.len() > 64 * 1024 {
        return None;
    }
    let mut stops = Vec::new();
    let text = expand(&mut value.chars().peekable(), false, 0, 0, &mut stops)?;
    stops.sort_by_key(|(index, range)| (*index == 0, *index, range.start));
    Some((text, stops.into_iter().next().map(|(_, range)| range)))
}

fn number(chars: &mut Peekable<Chars<'_>>) -> Option<u32> {
    let mut digits = String::new();
    while chars.peek().is_some_and(char::is_ascii_digit) {
        digits.push(chars.next()?);
    }
    digits.parse().ok()
}

fn expand(
    chars: &mut Peekable<Chars<'_>>,
    nested: bool,
    depth: usize,
    base: usize,
    stops: &mut Vec<(u32, Range<usize>)>,
) -> Option<String> {
    if depth > 16 {
        return None;
    }
    let mut result = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '}' if nested => return Some(result),
            '\\' => {
                if chars
                    .peek()
                    .is_some_and(|ch| matches!(ch, '$' | '}' | '\\'))
                {
                    result.push(chars.next()?);
                } else {
                    result.push(ch);
                }
            }
            '$' => match chars.peek().copied() {
                Some('0'..='9') => {
                    let index = number(chars)?;
                    let start = base + result.len();
                    stops.push((index, start..start));
                }
                Some('{') => {
                    chars.next();
                    if !chars.peek().is_some_and(char::is_ascii_digit) {
                        return None;
                    }
                    let index = number(chars)?;
                    let start = base + result.len();
                    match chars.next()? {
                        '}' => {}
                        ':' => result.push_str(&expand(chars, true, depth + 1, start, stops)?),
                        '|' => result.push_str(&choice(chars)?),
                        _ => return None,
                    }
                    stops.push((index, start..base + result.len()));
                }
                Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => return None,
                _ => result.push('$'),
            },
            _ => result.push(ch),
        }
    }
    (!nested).then_some(result)
}

fn choice(chars: &mut Peekable<Chars<'_>>) -> Option<String> {
    let mut first = String::new();
    let mut keep = true;
    while let Some(ch) = chars.next() {
        match ch {
            '|' => return (chars.next()? == '}').then_some(first),
            ',' => keep = false,
            '\\' => {
                let escaped = chars.next()?;
                if keep {
                    first.push(escaped);
                }
            }
            _ if keep => first.push(ch),
            _ => {}
        }
    }
    None
}
