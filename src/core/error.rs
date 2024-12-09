use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum PlayHandError {
    #[error("Played hand contains more than 5 cards")]
    TooManyCards,
    #[error("Played hand contains no cards")]
    NoCards,
    #[error("Played hand could not determine best hand")]
    UnknownHand,
}

#[derive(Error, Debug, Clone)]
pub enum GameError {
    #[error("No remaining discards")]
    NoRemainingDiscards,
    #[error("No remaining plays")]
    NoRemainingPlays,
    #[error("Invalid hand played")]
    InvalidHand(#[from] PlayHandError),
    #[error("Invalid stage")]
    InvalidStage,
    #[error("Invalid action")]
    InvalidAction,
    #[error("No card match")]
    InvalidBlind,
    #[error("No card match")]
    NoCardMatch,
    #[error("No joker match")]
    NoJokerMatch,
    #[error("Invalid move direction")]
    InvalidMoveDirection,
}
