use std::{iter::Peekable, str::Chars};

// Keep the editor's plain-text insertion contract. Expand initial placeholder
// values, discard tab stops, and reject variables/transforms we cannot resolve.
pub(super) fn plain_text(value: &str) -> Option<String> {
    if value.len() > 64 * 1024 {
        return None;
    }
    expand(&mut value.chars().peekable(), false, 0)
}

fn expand(chars: &mut Peekable<Chars<'_>>, nested: bool, depth: usize) -> Option<String> {
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
                    while chars.peek().is_some_and(char::is_ascii_digit) {
                        chars.next();
                    }
                }
                Some('{') => {
                    chars.next();
                    if !chars.peek().is_some_and(char::is_ascii_digit) {
                        return None;
                    }
                    while chars.peek().is_some_and(char::is_ascii_digit) {
                        chars.next();
                    }
                    match chars.next()? {
                        '}' => {}
                        ':' => result.push_str(&expand(chars, true, depth + 1)?),
                        '|' => result.push_str(&choice(chars)?),
                        _ => return None,
                    }
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
