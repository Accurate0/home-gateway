use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(from = "String")]
#[schemars(with = "String")]
pub enum TemplateString {
    Plain(String),
    Template(String),
}

impl TemplateString {
    pub fn render(&self, vars: &HashMap<String, String>) -> String {
        match self {
            TemplateString::Plain(s) => s.clone(),
            TemplateString::Template(s) => render_template(s, vars),
        }
    }

    pub fn raw(&self) -> &str {
        match self {
            TemplateString::Plain(s) | TemplateString::Template(s) => s,
        }
    }

    pub fn placeholders(&self) -> Vec<&str> {
        let mut names = Vec::new();
        let mut rest = match self {
            TemplateString::Plain(_) => return names,
            TemplateString::Template(s) => s.as_str(),
        };

        while let Some(start) = rest.find("${") {
            let after = &rest[start + 2..];
            match after.find('}') {
                Some(end) => {
                    names.push(&after[..end]);
                    rest = &after[end + 1..];
                }
                None => break,
            }
        }

        names
    }
}

impl From<String> for TemplateString {
    fn from(s: String) -> Self {
        if s.contains("${") {
            TemplateString::Template(s)
        } else {
            TemplateString::Plain(s)
        }
    }
}

impl std::fmt::Display for TemplateString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.raw())
    }
}

fn render_template(input: &str, vars: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) => {
                let key = &after[..end];
                match vars.get(key) {
                    Some(value) => out.push_str(value),
                    None => {
                        out.push_str("${");
                        out.push_str(key);
                        out.push('}');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }
    }

    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> HashMap<String, String> {
        HashMap::from([
            ("name".to_owned(), "Milk".to_owned()),
            ("old_price".to_owned(), "4.50".to_owned()),
            ("new_price".to_owned(), "3.00".to_owned()),
        ])
    }

    #[test]
    fn plain_string_has_no_placeholders() {
        let t: TemplateString = "just text".to_owned().into();
        assert_eq!(t, TemplateString::Plain("just text".to_owned()));
        assert_eq!(t.render(&vars()), "just text");
    }

    #[test]
    fn substitutes_known_vars() {
        let t: TemplateString = "${name}: ${old_price} -> ${new_price}".to_owned().into();
        assert!(matches!(t, TemplateString::Template(_)));
        assert_eq!(t.render(&vars()), "Milk: 4.50 -> 3.00");
    }

    #[test]
    fn unknown_var_left_literal() {
        let t: TemplateString = "${name} ${bogus}".to_owned().into();
        assert_eq!(t.render(&vars()), "Milk ${bogus}");
    }

    #[test]
    fn unterminated_placeholder_left_literal() {
        let t: TemplateString = "drop ${name".to_owned().into();
        assert_eq!(t.render(&vars()), "drop ${name");
    }

    #[test]
    fn placeholders_lists_referenced_vars() {
        let t: TemplateString = "${name}: ${old_price} ${name}".to_owned().into();
        assert_eq!(t.placeholders(), vec!["name", "old_price", "name"]);

        let plain: TemplateString = "no vars".to_owned().into();
        assert!(plain.placeholders().is_empty());
    }
}
