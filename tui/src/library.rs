//! Scan / rename / delete line-delimited replay logs. I/O stays out of core.

use anyhow::{bail, Context, Result};
use balatro_rs::replay_log::{parse_line, ReplayRecord, ReplayResult, ReplaySeed, ReplaySource};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct LibraryEntry {
    pub path: PathBuf,
    pub seed: ReplaySeed,
    pub agent: Option<String>,
    pub source: ReplaySource,
    pub result: Option<ReplayResult>,
    pub score: Option<usize>,
    pub steps: Option<usize>,
    pub mtime: SystemTime,
}

impl LibraryEntry {
    pub fn display_name(&self) -> String {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("replay")
            .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct LibraryState {
    pub dir: PathBuf,
    pub entries: Vec<LibraryEntry>,
    pub cursor: usize,
    pub rename_input: Option<String>,
    pub confirm_delete: bool,
    pub status: String,
}

impl LibraryState {
    pub fn open(dir: PathBuf) -> Self {
        let mut state = Self {
            dir,
            entries: Vec::new(),
            cursor: 0,
            rename_input: None,
            confirm_delete: false,
            status: String::new(),
        };
        state.refresh();
        state
    }

    pub fn refresh(&mut self) {
        match scan_replays(&self.dir) {
            Ok(entries) => {
                self.entries = entries;
                if self.cursor >= self.entries.len() && !self.entries.is_empty() {
                    self.cursor = self.entries.len() - 1;
                }
                if self.entries.is_empty() {
                    self.cursor = 0;
                }
                self.status = format!("{} replays in {}", self.entries.len(), self.dir.display());
            }
            Err(err) => {
                self.entries.clear();
                self.cursor = 0;
                self.status = format!("scan failed: {err:#}");
            }
        }
        self.rename_input = None;
        self.confirm_delete = false;
    }

    pub fn selected(&self) -> Option<&LibraryEntry> {
        self.entries.get(self.cursor)
    }
}

pub fn scan_replays(dir: &Path) -> Result<Vec<LibraryEntry>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    if !dir.is_dir() {
        bail!("{} is not a directory", dir.display());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        match load_entry(&path) {
            Ok(e) => entries.push(e),
            Err(_) => {}
        }
    }
    entries.sort_by(|a, b| b.mtime.cmp(&a.mtime));
    Ok(entries)
}

fn load_entry(path: &Path) -> Result<LibraryEntry> {
    let meta = fs::metadata(path)?;
    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header_line = lines.next().context("missing header")?;
    let ReplayRecord::Header(header) = parse_line(header_line)? else {
        bail!("first line is not a header");
    };

    let mut result = None;
    let mut score = None;
    let mut steps = None;
    // Prefer the last non-empty line if it is an outcome; otherwise scan.
    if let Some(last) = text.lines().rev().find(|l| !l.trim().is_empty()) {
        if let Ok(ReplayRecord::Outcome(outcome)) = parse_line(last) {
            result = Some(outcome.result);
            score = Some(outcome.score);
            steps = Some(outcome.steps);
        }
    }
    if result.is_none() {
        for line in text.lines() {
            if let Ok(ReplayRecord::Outcome(outcome)) = parse_line(line) {
                result = Some(outcome.result);
                score = Some(outcome.score);
                steps = Some(outcome.steps);
            }
        }
    }

    Ok(LibraryEntry {
        path: path.to_path_buf(),
        seed: header.seed,
        agent: header.agent.clone(),
        source: header.source,
        result,
        score,
        steps,
        mtime,
    })
}

pub fn validate_replay_name(name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        bail!("name cannot be empty");
    }
    if name.len() > 48 {
        bail!("name too long (max 48 characters)");
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.')
    {
        bail!("use letters, numbers, spaces, and - _ . only");
    }
    Ok(())
}

pub fn rename_replay(path: &Path, new_stem: &str) -> Result<PathBuf> {
    validate_replay_name(new_stem)?;
    let parent = path.parent().context("replay has no parent directory")?;
    let new_path = parent.join(format!("{}.log", new_stem.trim()));
    if new_path == path {
        return Ok(path.to_path_buf());
    }
    if new_path.exists() {
        bail!("a replay named '{}' already exists", new_stem.trim());
    }
    fs::rename(path, &new_path)
        .with_context(|| format!("rename {} -> {}", path.display(), new_path.display()))?;
    Ok(new_path)
}

pub fn delete_replay(path: &Path) -> Result<()> {
    fs::remove_file(path).with_context(|| format!("delete {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use balatro_rs::config::Config;
    use balatro_rs::game::Game;
    use balatro_rs::replay_log::{header_line, move_line, outcome_line, ReplayMove};
    use std::fs::File;
    use std::io::Write;
    use std::time::UNIX_EPOCH;

    fn write_sample_log(dir: &Path, name: &str, seed: u64) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        let config = Config {
            seed: Some(seed),
            ante_end: 1,
            ..Config::default()
        };
        let mut game = Game::new(config);
        game.start();
        let mut file = File::create(&path).unwrap();
        writeln!(
            file,
            "{}",
            header_line(
                &game.config,
                game.seed,
                game.seed_str.as_deref(),
                ReplaySource::Player,
                None
            )
            .unwrap()
        )
        .unwrap();
        writeln!(file, "{}", move_line(ReplayMove::Action(78))).unwrap();
        game.handle_action_index(78).unwrap();
        writeln!(file, "{}", outcome_line(&game, 1).unwrap()).unwrap();
        path
    }

    #[test]
    fn scan_skips_junk_and_sorts_newest_first() {
        let dir = std::env::temp_dir().join(format!(
            "balatro-lib-scan-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        write_sample_log(&dir, "older.log", 1);
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_sample_log(&dir, "newer.log", 2);
        fs::write(dir.join("junk.log"), "not json\n").unwrap();
        fs::write(dir.join("ignore.txt"), "nope\n").unwrap();

        let entries = scan_replays(&dir).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].display_name(), "newer");
        assert_eq!(entries[1].display_name(), "older");
        assert_eq!(entries[0].seed, ReplaySeed::Number(2));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_and_delete_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "balatro-lib-rename-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        let path = write_sample_log(&dir, "run.log", 9);
        let renamed = rename_replay(&path, "flush run").unwrap();
        assert_eq!(renamed.file_name().unwrap(), "flush run.log");
        assert!(!path.exists());
        delete_replay(&renamed).unwrap();
        assert!(!renamed.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_replay_name_rejects_bad_chars() {
        assert!(validate_replay_name("ok-name_1").is_ok());
        assert!(validate_replay_name("bad/name").is_err());
        assert!(validate_replay_name("").is_err());
    }
}
