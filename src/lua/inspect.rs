use std::ffi::c_void;
use std::fmt::Write;

use mlua::Value as LuaValue;

const MAX_DEPTH: usize = 6;
const INDENT: &str = "  ";

pub fn inspect(value: &LuaValue) -> String {
    let mut out = String::new();
    let mut seen = Vec::new();

    write_value(value, 0, &mut seen, &mut out);

    out
}

fn write_value(value: &LuaValue, depth: usize, seen: &mut Vec<*const c_void>, out: &mut String) {
    match value {
        LuaValue::Nil => out.push_str("nil"),
        LuaValue::Boolean(flag) => write!(out, "{flag}").unwrap_or_default(),
        LuaValue::Integer(number) => write!(out, "{number}").unwrap_or_default(),
        LuaValue::Number(number) if number.fract() == 0.0 && number.is_finite() => {
            write!(out, "{number:.1}").unwrap_or_default()
        }
        LuaValue::Number(number) => write!(out, "{number}").unwrap_or_default(),
        LuaValue::String(text) => write!(out, "{:?}", text.to_string_lossy()).unwrap_or_default(),
        LuaValue::Table(table) => {
            let pointer = value.to_pointer();

            if seen.contains(&pointer) {
                out.push_str("<cycle>");
                return;
            }

            if depth >= MAX_DEPTH {
                out.push_str("{...}");
                return;
            }

            let mut sequence = Vec::new();
            let mut fields = Vec::new();

            for (index, entry) in table.clone().pairs::<LuaValue, LuaValue>().enumerate() {
                let Ok((key, field)) = entry else {
                    continue;
                };

                match key {
                    LuaValue::Integer(position) if position == index as i64 + 1 => {
                        sequence.push(field)
                    }
                    other => fields.push((key_label(&other), field)),
                }
            }

            if sequence.is_empty() && fields.is_empty() {
                out.push_str("{}");
                return;
            }

            fields.sort_by(|a, b| a.0.cmp(&b.0));
            seen.push(pointer);

            let inner = INDENT.repeat(depth + 1);
            out.push_str("{\n");

            for field in &sequence {
                out.push_str(&inner);
                write_value(field, depth + 1, seen, out);
                out.push_str(",\n");
            }

            for (label, field) in &fields {
                write!(out, "{inner}{label} = ").unwrap_or_default();
                write_value(field, depth + 1, seen, out);
                out.push_str(",\n");
            }

            out.push_str(&INDENT.repeat(depth));
            out.push('}');
            seen.pop();
        }
        LuaValue::Function(_) => out.push_str("function"),
        other => match other.to_string() {
            Ok(text) => out.push_str(&text),
            Err(_) => out.push_str(other.type_name()),
        },
    }
}

fn key_label(key: &LuaValue) -> String {
    match key {
        LuaValue::String(text) => {
            let text = text.to_string_lossy();
            let identifier = text
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
                && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

            if identifier {
                text
            } else {
                format!("[{text:?}]")
            }
        }
        other => {
            let mut label = String::new();
            write_value(other, MAX_DEPTH, &mut Vec::new(), &mut label);

            format!("[{label}]")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::inspect;

    fn render(source: &str) -> String {
        let lua = mlua::Lua::new();
        let value = lua.load(source).eval().expect("expected the chunk to run");

        inspect(&value)
    }

    #[test]
    fn scalars_print_like_lua() {
        assert_eq!(render("return 2"), "2");
        assert_eq!(render("return 2.0"), "2.0");
        assert_eq!(render("return 'hi'"), "\"hi\"");
        assert_eq!(render("return nil"), "nil");
    }

    #[test]
    fn tables_list_the_sequence_then_sorted_fields() {
        assert_eq!(
            render("return { 'a', 'b', z = 1, on = function() end, ['two words'] = true }"),
            "{\n  \"a\",\n  \"b\",\n  [\"two words\"] = true,\n  on = function,\n  z = 1,\n}"
        );
    }

    #[test]
    fn a_cycle_does_not_recurse_forever() {
        assert_eq!(
            render("local t = {} t.me = t return t"),
            "{\n  me = <cycle>,\n}"
        );
    }
}
