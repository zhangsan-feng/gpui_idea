use anyhow::{Result, ensure};

pub(in crate::infrastructure::filesystem) fn prepare_run_command(
    command: &str,
    arguments: &str,
    prefix: &str,
    template: Option<&str>,
) -> Result<String> {
    let arguments = parse(arguments)?
        .into_iter()
        .map(|argument| format!("'{}'", argument.replace('\'', "''")))
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
