use balatro_jkr::{LuaKey, LuaValue};

use crate::error::ProfileError;

/// Looks up a string key in a `LuaValue::Table`.
pub fn get<'a>(v: &'a LuaValue, key: &str) -> Option<&'a LuaValue> {
    match v {
        LuaValue::Table(entries) => entries.iter().find_map(|(k, val)| match k {
            LuaKey::Str(s) if s == key => Some(val),
            _ => None,
        }),
        _ => None,
    }
}

pub fn require<'a>(v: &'a LuaValue, key: &'static str) -> Result<&'a LuaValue, ProfileError> {
    get(v, key).ok_or(ProfileError::MissingField(key))
}

pub fn as_table(v: &LuaValue) -> Option<&Vec<(LuaKey, LuaValue)>> {
    match v {
        LuaValue::Table(entries) => Some(entries),
        _ => None,
    }
}

pub fn as_str(v: &LuaValue) -> Option<&str> {
    match v {
        LuaValue::Str(s) => Some(s),
        _ => None,
    }
}

pub fn as_num(v: &LuaValue) -> Option<f64> {
    match v {
        LuaValue::Num(n) => Some(*n),
        _ => None,
    }
}

/// Iterates a table's string-keyed entries, skipping numeric-keyed ones.
pub fn str_entries(v: &LuaValue) -> impl Iterator<Item = (&str, &LuaValue)> {
    as_table(v)
        .into_iter()
        .flatten()
        .filter_map(|(k, val)| match k {
            LuaKey::Str(s) => Some((s.as_str(), val)),
            LuaKey::Num(_) => None,
        })
}
