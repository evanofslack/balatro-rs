use crate::value::{LuaKey, LuaValue};

/// Renders a [`LuaValue`] back into Lua table-literal source text
/// (the inverse of [`crate::parser::parse`], modulo the `"return "` prefix).
pub fn print(value: &LuaValue) -> String {
    let mut out = String::new();
    write_value(value, &mut out);
    out
}

fn write_value(value: &LuaValue, out: &mut String) {
    match value {
        LuaValue::Nil => out.push_str("nil"),
        LuaValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        LuaValue::Num(n) => out.push_str(&n.to_string()),
        LuaValue::Str(s) => write_str(s, out),
        LuaValue::Table(entries) => write_table(entries, out),
    }
}

fn write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out.push('"');
}

fn write_table(entries: &[(LuaKey, LuaValue)], out: &mut String) {
    out.push('{');
    for (key, value) in entries {
        out.push('[');
        match key {
            LuaKey::Str(s) => write_str(s, out),
            LuaKey::Num(n) => out.push_str(&n.to_string()),
        }
        out.push_str("]=");
        write_value(value, out);
        out.push(',');
    }
    out.push('}');
}

/// Like [`print`], but indents nested tables across multiple lines instead
/// of writing everything on one line. Still valid Lua source.
pub fn print_pretty(value: &LuaValue) -> String {
    let mut out = String::new();
    write_value_pretty(value, &mut out, 0);
    out
}

fn write_value_pretty(value: &LuaValue, out: &mut String, indent: usize) {
    match value {
        LuaValue::Table(entries) if !entries.is_empty() => write_table_pretty(entries, out, indent),
        _ => write_value(value, out),
    }
}

fn write_table_pretty(entries: &[(LuaKey, LuaValue)], out: &mut String, indent: usize) {
    out.push_str("{\n");
    let inner = indent + 1;
    for (key, value) in entries {
        push_indent(out, inner);
        out.push('[');
        match key {
            LuaKey::Str(s) => write_str(s, out),
            LuaKey::Num(n) => out.push_str(&n.to_string()),
        }
        out.push_str("] = ");
        write_value_pretty(value, out, inner);
        out.push_str(",\n");
    }
    push_indent(out, indent);
    out.push('}');
}

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn roundtrip(src: &str) {
        let parsed = parse(src).unwrap();
        let printed = print(&parsed);
        let reparsed = parse(&printed).unwrap();
        assert_eq!(parsed, reparsed, "printed form: {printed}");
    }

    #[test]
    fn roundtrips_empty_table() {
        roundtrip("{}");
    }

    #[test]
    fn roundtrips_string_keys() {
        roundtrip(r#"{["a"]=1,["b"]=true,}"#);
    }

    #[test]
    fn roundtrips_numeric_keys() {
        roundtrip(r#"{[1]="x",[2]="y",}"#);
    }

    #[test]
    fn roundtrips_nested_tables() {
        roundtrip(r#"{["outer"]={["inner"]=nil,},}"#);
    }

    #[test]
    fn roundtrips_escaped_strings() {
        roundtrip(r#"{["quote"]="a\"b\\c",}"#);
    }

    #[test]
    fn roundtrips_negative_and_float_numbers() {
        roundtrip(r#"{["a"]=-4,["b"]=1.5,}"#);
    }

    #[test]
    fn integral_floats_print_without_decimal() {
        assert_eq!(print(&LuaValue::Num(4.0)), "4");
    }

    #[test]
    fn pretty_roundtrips_nested_tables() {
        let src = r#"{["outer"]={["inner"]=1,["list"]={[1]="a",[2]="b",},},["flag"]=true,}"#;
        let parsed = parse(src).unwrap();
        let printed = print_pretty(&parsed);
        let reparsed = parse(&printed).unwrap();
        assert_eq!(parsed, reparsed, "printed form: {printed}");
    }

    #[test]
    fn pretty_prints_empty_table_compact() {
        assert_eq!(print_pretty(&LuaValue::Table(vec![])), "{}");
    }
}
