use crate::error::LuaError;
use crate::lexer::{Token, tokenize};
use crate::value::{LuaKey, LuaValue};

/// Parses a `.jkr` file's decompressed Lua source (the `return {...}` text)
/// into a [`LuaValue`].
pub fn parse(input: &str) -> Result<LuaValue, LuaError> {
    let input = input.strip_prefix("return ").unwrap_or(input);
    let tokens = tokenize(input)?;
    let mut pos = 0;
    let value = parse_value(&tokens, &mut pos)?;
    if pos != tokens.len() {
        return Err(LuaError::TrailingContent);
    }
    Ok(value)
}

fn parse_value(tokens: &[Token], pos: &mut usize) -> Result<LuaValue, LuaError> {
    match tokens.get(*pos) {
        Some(Token::LBrace) => parse_table(tokens, pos),
        Some(Token::Str(s)) => {
            let v = s.clone();
            *pos += 1;
            Ok(LuaValue::Str(v))
        }
        Some(Token::Num(n)) => {
            let v = *n;
            *pos += 1;
            Ok(LuaValue::Num(v))
        }
        Some(Token::Bool(b)) => {
            let v = *b;
            *pos += 1;
            Ok(LuaValue::Bool(v))
        }
        Some(Token::Nil) => {
            *pos += 1;
            Ok(LuaValue::Nil)
        }
        Some(t) => Err(LuaError::UnexpectedToken {
            found: format!("{t:?}"),
        }),
        None => Err(LuaError::UnexpectedEof),
    }
}

fn expect(tokens: &[Token], pos: &mut usize, expected: &Token) -> Result<(), LuaError> {
    match tokens.get(*pos) {
        Some(t) if t == expected => {
            *pos += 1;
            Ok(())
        }
        Some(t) => Err(LuaError::UnexpectedToken {
            found: format!("{t:?}"),
        }),
        None => Err(LuaError::UnexpectedEof),
    }
}

fn parse_table(tokens: &[Token], pos: &mut usize) -> Result<LuaValue, LuaError> {
    expect(tokens, pos, &Token::LBrace)?;
    let mut entries = Vec::new();

    while !matches!(tokens.get(*pos), Some(Token::RBrace) | None) {
        expect(tokens, pos, &Token::LBracket)?;
        let key = match tokens.get(*pos) {
            Some(Token::Str(s)) => {
                let k = LuaKey::Str(s.clone());
                *pos += 1;
                k
            }
            Some(Token::Num(n)) => {
                let k = LuaKey::Num(*n as i64);
                *pos += 1;
                k
            }
            Some(t) => {
                return Err(LuaError::UnexpectedToken {
                    found: format!("{t:?}"),
                });
            }
            None => return Err(LuaError::UnexpectedEof),
        };
        expect(tokens, pos, &Token::RBracket)?;
        expect(tokens, pos, &Token::Equals)?;
        let value = parse_value(tokens, pos)?;
        entries.push((key, value));

        if matches!(tokens.get(*pos), Some(Token::Comma)) {
            *pos += 1;
        }
    }

    expect(tokens, pos, &Token::RBrace)?;
    Ok(LuaValue::Table(entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_table() {
        assert_eq!(parse("{}").unwrap(), LuaValue::Table(vec![]));
    }

    #[test]
    fn strips_return_prefix() {
        assert_eq!(parse("return {}").unwrap(), LuaValue::Table(vec![]));
    }

    #[test]
    fn string_keyed_table() {
        let v = parse(r#"{["a"]=1,["b"]=true,}"#).unwrap();
        assert_eq!(
            v,
            LuaValue::Table(vec![
                (LuaKey::Str("a".into()), LuaValue::Num(1.0)),
                (LuaKey::Str("b".into()), LuaValue::Bool(true)),
            ])
        );
    }

    #[test]
    fn numeric_keyed_table() {
        let v = parse(r#"{[1]="x",[2]="y",}"#).unwrap();
        assert_eq!(
            v,
            LuaValue::Table(vec![
                (LuaKey::Num(1), LuaValue::Str("x".into())),
                (LuaKey::Num(2), LuaValue::Str("y".into())),
            ])
        );
    }

    #[test]
    fn nested_tables() {
        let v = parse(r#"{["outer"]={["inner"]=nil,},}"#).unwrap();
        assert_eq!(
            v,
            LuaValue::Table(vec![(
                LuaKey::Str("outer".into()),
                LuaValue::Table(vec![(LuaKey::Str("inner".into()), LuaValue::Nil)])
            )])
        );
    }

    #[test]
    fn mixed_key_table() {
        let v = parse(r#"{["a"]=1,[1]=2,}"#).unwrap();
        assert_eq!(
            v,
            LuaValue::Table(vec![
                (LuaKey::Str("a".into()), LuaValue::Num(1.0)),
                (LuaKey::Num(1), LuaValue::Num(2.0)),
            ])
        );
    }

    #[test]
    fn trailing_comma_optional() {
        assert_eq!(
            parse(r#"{["a"]=1}"#).unwrap(),
            parse(r#"{["a"]=1,}"#).unwrap()
        );
    }

    #[test]
    fn unterminated_table_errors() {
        assert_eq!(parse(r#"{["a"]=1,"#), Err(LuaError::UnexpectedEof));
    }

    #[test]
    fn missing_equals_errors() {
        assert!(parse(r#"{["a"]1}"#).is_err());
    }
}
