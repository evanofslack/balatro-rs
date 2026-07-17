use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ProfileError {
    #[error("missing expected field: {0}")]
    MissingField(&'static str),
    #[error("field {field} has wrong type, expected {expected}")]
    WrongType {
        field: &'static str,
        expected: &'static str,
    },
    #[error("unrecognized item id: {0}")]
    UnknownId(String),
}
