use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum LuaError {
    #[error("unexpected character '{ch}' at position {pos}")]
    UnexpectedChar { ch: char, pos: usize },
    #[error("unterminated string literal")]
    UnterminatedString,
    #[error("invalid number literal: {0}")]
    InvalidNumber(String),
    #[error("unknown identifier: {0}")]
    UnknownIdentifier(String),
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("unexpected token: {found}")]
    UnexpectedToken { found: String },
    #[error("trailing content after top-level value")]
    TrailingContent,
}

#[derive(Debug, Error)]
pub enum JkrError {
    #[error("failed to decompress: {0}")]
    Decompress(String),
    #[error("decompressed data is not valid utf-8")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Lua(#[from] LuaError),
}
