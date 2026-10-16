use anyhow::{Result, ensure};

pub(in crate::infrastructure::filesystem) fn prepare_run_command(
    command: &str,
    arguments: &str,
    prefix: &str,
    template: Option<&str>,
) -> Result<String> {
    let arguments = parse(arguments)?
        .into_iter()
        .map(|argument| quote_argument(&argument))
        .collect::<Vec<_>>()
        .join(" ");
    if arguments.is_empty() {
        return Ok(command.to_owned());
    }
    if let Some(template) = template {
        return Ok(template.replace("__GPUI_PROGRAM_ARGUMENTS__", &arguments));
    }
    Ok(format!(
        "{command} {}{arguments}",
        if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix} ")
        }
    ))
}

#[cfg(not(windows))]
fn quote_argument(argument: &str) -> String {
    format!("'{}'", argument.replace('\'', "'\"'\"'"))
}

#[cfg(windows)]
fn quote_argument(argument: &str) -> String {
    // First encode one CRT argv item, including trailing backslashes and quotes.
    let mut encoded = String::from("\"");
    let mut backslashes = 0;
    for ch in argument.chars() {
        if ch == '\\' {
            backslashes += 1;
            continue;
        }
        encoded.extend(std::iter::repeat_n(
            '\\',
            backslashes * if ch == '"' { 2 } else { 1 },
        ));
        if ch == '"' {
            encoded.push('\\');
        }
        encoded.push(ch);
        backslashes = 0;
    }
    encoded.extend(std::iter::repeat_n('\\', backslashes * 2));
    encoded.push('"');
    // Then protect it from batch parsing. Delayed expansion is disabled by Launch.
    let mut escaped = String::new();
    for ch in encoded.chars() {
        if ch == '%' {
            escaped.push('%');
        } else if matches!(ch, '^' | '"' | '&' | '|' | '<' | '>' | '(' | ')') {
            escaped.push('^');
        }
        escaped.push(ch);
    }
    escaped
}

fn parse(arguments: &str) -> Result<Vec<String>> {
    ensure!(
        arguments.len() <= 16_384 && !arguments.contains(['\0', '\r', '\n']),
        "启动参数过长或包含非法字符"
    );
    let mut result = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = arguments.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\'' | '"' if quote.is_none() => {
                quote = Some(ch);
                started = true;
            }
            ch if quote == Some(ch) => quote = None,
            '\\' if quote != Some('\'')
                && chars.peek().is_some_and(|next| {
                    *next == '"' || *next == '\\' || (quote.is_none() && next.is_whitespace())
                }) =>
            {
                token.push(chars.next().unwrap());
                started = true;
            }
            ch if quote.is_none() && ch.is_whitespace() => {
                if started {
                    result.push(std::mem::take(&mut token));
                    started = false;
                }
            }
            ch => {
                token.push(ch);
                started = true;
            }
        }
    }
    ensure!(quote.is_none(), "启动参数的引号未闭合");
    if started {
        result.push(token);
    }
    Ok(result)
}
