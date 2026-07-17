/// A Lua table key. Balatro's serializer always writes explicit keys
/// (`["foo"]=...` or `[1]=...`), so string and numeric keys are distinguished
/// as real variants here rather than collapsed into a single string type.
#[derive(Debug, Clone, PartialEq)]
pub enum LuaKey {
    Str(String),
    Num(i64),
}

/// A parsed Lua value from a `.jkr` file's `return {...}` literal.
///
/// Tables are `Vec<(LuaKey, LuaValue)>` rather than a map to preserve the
/// original field order from the source file.
#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    Nil,
    Bool(bool),
    Num(f64),
    Str(String),
    Table(Vec<(LuaKey, LuaValue)>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_equality() {
        assert_eq!(LuaKey::Str("a".into()), LuaKey::Str("a".into()));
        assert_ne!(LuaKey::Str("a".into()), LuaKey::Str("b".into()));
        assert_eq!(LuaKey::Num(1), LuaKey::Num(1));
        assert_ne!(LuaKey::Num(1), LuaKey::Str("1".into()));
    }

    #[test]
    fn value_equality() {
        let a = LuaValue::Table(vec![
            (LuaKey::Str("x".into()), LuaValue::Num(1.0)),
            (LuaKey::Num(1), LuaValue::Bool(true)),
        ]);
        let b = LuaValue::Table(vec![
            (LuaKey::Str("x".into()), LuaValue::Num(1.0)),
            (LuaKey::Num(1), LuaValue::Bool(true)),
        ]);
        assert_eq!(a, b);

        let reordered = LuaValue::Table(vec![
            (LuaKey::Num(1), LuaValue::Bool(true)),
            (LuaKey::Str("x".into()), LuaValue::Num(1.0)),
        ]);
        assert_ne!(a, reordered, "field order is significant");
    }
}
