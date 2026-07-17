use std::fs;
use std::process::ExitCode;

use balatro_jkr::LuaValue;
use balatro_profile::{Profile, SaveSnapshot};

enum Kind {
    Save,
    MetaHalf,
    ProfileHalf,
}

/// Detects the `.jkr` file kind by content shape, not filename.
fn detect(v: &LuaValue) -> Option<Kind> {
    let LuaValue::Table(entries) = v else {
        return None;
    };
    let has = |key: &str| {
        entries
            .iter()
            .any(|(k, _)| matches!(k, balatro_jkr::LuaKey::Str(s) if s == key))
    };
    if has("GAME") && has("cardAreas") {
        Some(Kind::Save)
    } else if has("unlocked") && has("discovered") {
        Some(Kind::MetaHalf)
    } else if has("high_scores") && has("career_stats") {
        Some(Kind::ProfileHalf)
    } else {
        None
    }
}

fn load(path: &str) -> Result<LuaValue, String> {
    let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    balatro_jkr::decode(&bytes).map_err(|e| format!("{path}: {e}"))
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let short = if let Some(i) = args.iter().position(|a| a == "-s" || a == "--short") {
        args.remove(i);
        true
    } else {
        false
    };

    if args.is_empty() || args.len() > 2 {
        eprintln!("usage: profile [-s|--short] <file.jkr> [file.jkr]");
        eprintln!("  one path -> a save.jkr, or one half of a meta+profile pair");
        eprintln!("  two paths -> a meta.jkr + profile.jkr pair (either order)");
        return ExitCode::from(2);
    }

    let loaded: Vec<(String, LuaValue)> = match args
        .iter()
        .map(|p| load(p).map(|v| (p.clone(), v)))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let kinds: Vec<(Kind, &LuaValue)> = loaded
        .iter()
        .filter_map(|(path, v)| match detect(v) {
            Some(k) => Some((k, v)),
            None => {
                eprintln!("{path}: unrecognized .jkr shape (not a save, meta, or profile half)");
                None
            }
        })
        .collect();

    if kinds.len() != loaded.len() {
        return ExitCode::FAILURE;
    }

    let result = match kinds.as_slice() {
        [(Kind::Save, save)] => SaveSnapshot::from_lua(save).map(|s| {
            if short {
                s.summary().to_string()
            } else {
                s.to_string()
            }
        }),
        [(Kind::MetaHalf, meta), (Kind::ProfileHalf, profile)]
        | [(Kind::ProfileHalf, profile), (Kind::MetaHalf, meta)] => {
            Profile::from_lua(meta, profile).map(|p| {
                if short {
                    p.summary().to_string()
                } else {
                    p.to_string()
                }
            })
        }
        [(Kind::MetaHalf, _)] => {
            eprintln!("only the meta.jkr half was given — need the matching profile.jkr too");
            return ExitCode::FAILURE;
        }
        [(Kind::ProfileHalf, _)] => {
            eprintln!("only the profile.jkr half was given — need the matching meta.jkr too");
            return ExitCode::FAILURE;
        }
        _ => {
            eprintln!("two files given, but they aren't a meta.jkr + profile.jkr pair");
            return ExitCode::FAILURE;
        }
    };

    match result {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
