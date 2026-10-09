use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("profile number must be 1, 2, or 3; got {0}")]
    InvalidProfile(u8),

    #[error("could not determine home directory")]
    NoHomeDir,

    #[error("plist not found at {0}")]
    PlistNotFound(PathBuf),

    #[error("failed to read plist: {0}")]
    PlistRead(String),

    #[error("failed to write plist: {0}")]
    PlistWrite(String),

    #[error("plist root value is not a dictionary")]
    PlistNotDict,

    #[error("plist key '{0}' is missing")]
    PlistKeyMissing(String),

    #[error("plist key '{0}' has unexpected type (expected Data)")]
    PlistKeyWrongType(String),

    #[error("failed to copy {src} → {dst}: {source}")]
    FileCopy {
        src: PathBuf,
        dst: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write file {path}: {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read file {path}: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Balatro is currently running; quit the game before importing saves")]
    GameRunning,
}
