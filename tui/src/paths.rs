//! Replays directory resolution. File I/O stays in the TUI/CLI, not core.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Resolve the replays directory: `BALATRO_REPLAYS_DIR`, else a `replays/` found
/// walking up from cwd, else `cwd/replays`.
pub fn default_replays_dir() -> PathBuf {
    if let Ok(dir) = env::var("BALATRO_REPLAYS_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(cwd) = env::current_dir() {
        let mut dir = cwd.as_path();
        loop {
            let candidate = dir.join("replays");
            if candidate.is_dir() {
                return candidate;
            }
            dir = match dir.parent() {
                Some(p) => p,
                None => break,
            };
        }
        return cwd.join("replays");
    }
    PathBuf::from("replays")
}

pub fn ensure_replays_dir(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)
}

/// A small TUI preference kept beside its replay library.
pub fn recording_enabled(dir: &Path) -> std::io::Result<bool> {
    match fs::read_to_string(dir.join(".tui-recording")) {
        Ok(value) => Ok(value.trim() != "off"),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(err) => Err(err),
    }
}

pub fn set_recording_enabled(dir: &Path, enabled: bool) -> std::io::Result<()> {
    ensure_replays_dir(dir)?;
    fs::write(
        dir.join(".tui-recording"),
        if enabled { "on\n" } else { "off\n" },
    )
}

/// Default record path: `{replays_dir}/{seed}_{unix_nanos}.log`.
pub fn default_record_path(replays_dir: &Path, seed: u64) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    replays_dir.join(format!("{seed}_{nanos}.log"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balatro_replays_dir_env_wins() {
        let prev = env::var_os("BALATRO_REPLAYS_DIR");
        env::set_var("BALATRO_REPLAYS_DIR", "/tmp/balatro-test-replays");
        assert_eq!(
            default_replays_dir(),
            PathBuf::from("/tmp/balatro-test-replays")
        );
        match prev {
            Some(v) => env::set_var("BALATRO_REPLAYS_DIR", v),
            None => env::remove_var("BALATRO_REPLAYS_DIR"),
        }
    }

    #[test]
    fn default_record_path_includes_seed_and_log_ext() {
        let path = default_record_path(Path::new("/tmp/replays"), 42);
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("log"));
        assert!(path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap()
            .starts_with("42_"));
    }
}
