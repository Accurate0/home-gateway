use super::{Expr, Segment};

pub fn parse_segments(input: &str) -> Result<Vec<Segment>, String> {
    let mut segments = Vec::new();
    let mut rest = input;

    while let Some(start) = rest.find("${") {
        if start > 0 {
            segments.push(Segment::Text(rest[..start].to_owned()));
        }

        let body = &rest[start + 2..];
        let end = find_outside_quotes(body, '}')
            .ok_or_else(|| format!("unterminated `${{` in template `{input}`"))?;

        let expr =
            Expr::parse(&body[..end]).map_err(|error| format!("in template `{input}`: {error}"))?;

        segments.push(Segment::Expr(expr));
        rest = &body[end + 1..];
    }

    if !rest.is_empty() {
        segments.push(Segment::Text(rest.to_owned()));
    }

    Ok(segments)
}

pub fn split_outside_quotes(input: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = input;

    while let Some(index) = find_outside_quotes(rest, separator) {
        parts.push(&rest[..index]);
        rest = &rest[index + separator.len_utf8()..];
    }

    parts.push(rest);
    parts
}

fn find_outside_quotes(input: &str, target: char) -> Option<usize> {
    let mut quote = None;

    for (index, c) in input.char_indices() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == target => return Some(index),
            None => {}
        }
    }

    None
}
