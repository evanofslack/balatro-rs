use balatro_rs::game::Game;
use balatro_rs::replay_log::{ReplaySource, ReplayWriter};
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::path::Path;

pub type FileReplayWriter = ReplayWriter<BufWriter<File>>;

pub fn create(path: &Path, game: &Game) -> std::io::Result<FileReplayWriter> {
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    ReplayWriter::new(BufWriter::new(file), game, ReplaySource::Player, None)
}
