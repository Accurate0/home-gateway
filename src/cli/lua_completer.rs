use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use serde_json::Value;

const KEYWORDS: [&str; 22] = [
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

const BUILTINS: [&str; 12] = [
    "print", "pairs", "ipairs", "type", "tostring", "tonumber", "select", "error", "pcall",
    "string", "table", "math",
];

pub struct LuaCompleter {
    candidates: Vec<Pair>,
}

impl LuaCompleter {
    pub fn from_api(api: &Value) -> Self {
        let mut candidates: Vec<Pair> = KEYWORDS
            .iter()
            .chain(BUILTINS.iter())
            .map(|word| plain(word))
            .collect();

        for global in api["globals"].as_array().into_iter().flatten() {
            if let Some(name) = global["name"].as_str() {
                candidates.push(plain(name));
            }
        }

        for namespace in api["namespaces"].as_array().into_iter().flatten() {
            let Some(prefix) = namespace["name"].as_str() else {
                continue;
            };

            candidates.push(plain(prefix));

            for field in namespace["fields"].as_array().into_iter().flatten() {
                if let Some(name) = field["name"].as_str() {
                    candidates.push(plain(&format!("{prefix}.{name}")));
                }
            }

            for function in namespace["functions"].as_array().into_iter().flatten() {
                let Some(name) = function["name"].as_str() else {
                    continue;
                };

                let replacement = format!("{prefix}.{name}(");
                let display = function["signature"]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| replacement.clone());

                candidates.push(Pair {
                    display,
                    replacement,
                });
            }
        }

        candidates.sort_by(|a, b| a.replacement.cmp(&b.replacement));
        candidates.dedup_by(|a, b| a.replacement == b.replacement);

        LuaCompleter { candidates }
    }
}

fn plain(word: &str) -> Pair {
    Pair {
        display: word.to_owned(),
        replacement: word.to_owned(),
    }
}

impl Completer for LuaCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start = line[..pos]
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
            .map_or(0, |index| index + 1);

        let word = &line[start..pos];

        if word.is_empty() {
            return Ok((pos, Vec::new()));
        }

        let matches = self
            .candidates
            .iter()
            .filter(|candidate| candidate.replacement.starts_with(word))
            .map(|candidate| Pair {
                display: candidate.display.clone(),
                replacement: candidate.replacement.clone(),
            })
            .collect();

        Ok((start, matches))
    }
}

impl Hinter for LuaCompleter {
    type Hint = String;
}

impl Highlighter for LuaCompleter {}

impl Validator for LuaCompleter {}

impl Helper for LuaCompleter {}

#[cfg(test)]
mod tests {
    use rustyline::Context;
    use rustyline::completion::Completer;
    use rustyline::history::DefaultHistory;
    use serde_json::json;

    use super::LuaCompleter;

    fn complete(completer: &LuaCompleter, line: &str) -> Vec<String> {
        let history = DefaultHistory::new();
        let (_, matches) = completer
            .complete(line, line.len(), &Context::new(&history))
            .expect("expected completion to succeed");

        matches.into_iter().map(|pair| pair.replacement).collect()
    }

    fn completer() -> LuaCompleter {
        LuaCompleter::from_api(&json!({
            "globals": [{ "name": "gw" }],
            "namespaces": [{
                "name": "light",
                "fields": [],
                "functions": [
                    { "name": "set", "signature": "light.set(id: string)" },
                    { "name": "state", "signature": "light.state(id: string)" },
                ],
            }],
        }))
    }

    #[test]
    fn a_namespace_prefix_lists_its_functions() {
        assert_eq!(
            complete(&completer(), "x = light.s"),
            vec!["light.set(", "light.state("]
        );
    }

    #[test]
    fn globals_and_keywords_complete_from_a_bare_word() {
        assert_eq!(complete(&completer(), "g"), vec!["goto", "gw"]);
    }
}
