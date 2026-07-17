use crate::error::LuaError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Equals,
    Comma,
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LuaError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '{' => {
                tokens.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                i += 1;
            }
            '[' => {
                tokens.push(Token::LBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                i += 1;
            }
            '=' => {
                tokens.push(Token::Equals);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '"' => {
                let mut s = String::new();
                i += 1;
                loop {
                    if i >= chars.len() {
                        return Err(LuaError::UnterminatedString);
                    }
                    match chars[i] {
                        '\\' if i + 1 < chars.len() => {
                            s.push(chars[i + 1]);
                            i += 2;
                        }
                        '"' => {
                            i += 1;
                            break;
                        }
                        c => {
                            s.push(c);
                            i += 1;
                        }
                    }
                }
                tokens.push(Token::Str(s));
            }
            '-' | '.' | '0'..='9' => {
                let start = i;
                while i < chars.len() && matches!(chars[i], '-' | '.' | '0'..='9' | 'e' | 'E' | '+')
                {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let n: f64 = num_str
                    .parse()
                    .map_err(|_| LuaError::InvalidNumber(num_str.clone()))?;
                tokens.push(Token::Num(n));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                while i < chars.len() && matches!(chars[i], 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
                {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.as_str() {
                    "true" => tokens.push(Token::Bool(true)),
                    "false" => tokens.push(Token::Bool(false)),
                    "nil" => tokens.push(Token::Nil),
                    _ => return Err(LuaError::UnknownIdentifier(word)),
                }
            }
            c => return Err(LuaError::UnexpectedChar { ch: c, pos: i }),
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_whitespace() {
        assert_eq!(
            tokenize("  {\n\t}\r").unwrap(),
            vec![Token::LBrace, Token::RBrace]
        );
    }

    #[test]
    fn brackets_and_punctuation() {
        assert_eq!(
            tokenize("[]={,}").unwrap(),
            vec![
                Token::LBracket,
                Token::RBracket,
                Token::Equals,
                Token::LBrace,
                Token::Comma,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn string_with_escapes() {
        let tokens = tokenize(r#""a\"b\\c""#).unwrap();
        assert_eq!(tokens, vec![Token::Str("a\"b\\c".to_string())]);
    }

    #[test]
    fn unterminated_string_errors() {
        assert_eq!(tokenize(r#"["a"#), Err(LuaError::UnterminatedString));
    }

    #[test]
    fn negative_and_scientific_numbers() {
        assert_eq!(tokenize("-4").unwrap(), vec![Token::Num(-4.0)]);
        assert_eq!(tokenize("1.5e3").unwrap(), vec![Token::Num(1500.0)]);
    }

    #[test]
    fn keywords() {
        assert_eq!(
            tokenize("true false nil").unwrap(),
            vec![Token::Bool(true), Token::Bool(false), Token::Nil]
        );
    }

    #[test]
    fn unknown_identifier_errors() {
        assert_eq!(
            tokenize("maybe"),
            Err(LuaError::UnknownIdentifier("maybe".to_string()))
        );
    }

    #[test]
    fn unexpected_character_errors() {
        assert_eq!(
            tokenize("@"),
            Err(LuaError::UnexpectedChar { ch: '@', pos: 0 })
        );
    }
}
