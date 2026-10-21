use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn default_data_dir() -> PathBuf {
    PathBuf::from("data")
}

pub fn default_record_path(replays_dir: &Path, seed: u64) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    replays_dir.join(format!("{seed}_{nanos}.log"))
}
