//! Load replay JSON and drive deterministic playback.

use anyhow::{bail, Context, Result};
use balatro_rs::{config::Config, game::Game};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct ReplayConfig {
    pub ante_end: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplayAgent {
    #[serde(rename = "type")]
    pub agent_type: String,
    pub checkpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplayOutcome {
    pub won: bool,
    pub score: usize,
    pub steps: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplayFile {
    pub format_version: u32,
    #[serde(default)]
    pub recorded_at: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub seed: u64,
    #[serde(default)]
    pub seed_str: Option<String>,
    pub config: ReplayConfig,
    #[serde(default)]
    pub agent: Option<ReplayAgent>,
    pub actions: Vec<usize>,
    #[serde(default)]
    pub outcome: Option<ReplayOutcome>,
}

#[derive(Debug, Clone)]
pub struct ReplayEntry {
    pub path: PathBuf,
    pub name: Option<String>,
    pub seed: u64,
    pub agent_label: String,
    pub ante_end: usize,
    pub won: bool,
    pub score: usize,
    pub steps: usize,
    pub recorded_at: String,
}

impl ReplayEntry {
    /// Matches replay filename convention: `{seed}_{agent}_ante{N}`.
    pub fn default_name(seed: u64, agent_label: &str, ante_end: usize) -> String {
        format!("{seed}_{agent_label}_ante{ante_end}")
    }

    pub fn display_name(&self) -> String {
        self.name
            .as_ref()
            .map(|n| n.trim())
            .filter(|n| !n.is_empty())
            .map(|n| n.to_string())
            .unwrap_or_else(|| Self::default_name(self.seed, &self.agent_label, self.ante_end))
    }

    pub fn has_custom_name(&self) -> bool {
        self.name
            .as_ref()
            .map(|n| !n.trim().is_empty())
            .unwrap_or(false)
    }
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

pub fn set_replay_name(path: &Path, name: &str) -> Result<()> {
    validate_replay_name(name)?;
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read replay {}", path.display()))?;
    let mut value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse replay {}", path.display()))?;
    let obj = value
        .as_object_mut()
        .context("replay JSON root must be an object")?;
    obj.insert(
        "name".to_string(),
        serde_json::Value::String(name.trim().to_string()),
    );
    let out = serde_json::to_string_pretty(&value)? + "\n";
    fs::write(path, out).with_context(|| format!("failed to write replay {}", path.display()))?;
    Ok(())
}

pub fn clear_replay_name(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read replay {}", path.display()))?;
    let mut value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse replay {}", path.display()))?;
    if let Some(obj) = value.as_object_mut() {
        obj.remove("name");
    }
    let out = serde_json::to_string_pretty(&value)? + "\n";
    fs::write(path, out).with_context(|| format!("failed to write replay {}", path.display()))?;
    Ok(())
}

pub struct ReplayDriver {
    pub actions: Vec<usize>,
    cursor: usize,
}

impl ReplayDriver {
    pub fn new(actions: Vec<usize>) -> Self {
        Self {
            actions,
            cursor: 0,
        }
    }

    pub fn next_action(&mut self) -> Option<usize> {
        if self.cursor >= self.actions.len() {
            return None;
        }
        let action = self.actions[self.cursor];
        self.cursor += 1;
        Some(action)
    }

    pub fn finished(&self) -> bool {
        self.cursor >= self.actions.len()
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.cursor, self.actions.len())
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn seek(&mut self, step: usize) {
        self.cursor = step.min(self.actions.len());
    }

    pub fn can_step_back(&self) -> bool {
        self.cursor > 0
    }

    pub fn peek_next(&self) -> Option<usize> {
        self.actions.get(self.cursor).copied()
    }
}

pub fn load_replay(path: &Path) -> Result<ReplayFile> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read replay {}", path.display()))?;
    let replay: ReplayFile = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse replay {}", path.display()))?;
    if replay.format_version != 2 {
        bail!(
            "unsupported replay format_version {} (expected 2; v1 used a pre-core action layout)",
            replay.format_version
        );
    }
    if replay.actions.is_empty() {
        bail!("replay has no actions: {}", path.display());
    }
    Ok(replay)
}

pub fn init_game_from_replay(replay: &ReplayFile) -> Game {
    reset_game_to_step(replay, 0).expect("step 0 always valid")
}

pub fn reset_game_to_step(replay: &ReplayFile, step: usize) -> Result<Game> {
    let mut game = {
        let mut config = Config::default();
        config.ante_end = replay.config.ante_end;
        if let Some(s) = &replay.seed_str {
            config.seed_str = Some(s.clone());
        } else {
            config.seed = Some(replay.seed);
        }
        let mut g = Game::new(config);
        g.start();
        g
    };
    let step = step.min(replay.actions.len());
    for &action in &replay.actions[..step] {
        crate::agent::apply_gym_action(&mut game, action)?;
    }
    Ok(game)
}

pub const REPLAY_BASE_MS: u64 = 400;
pub const CHROME_TOP: u16 = 3;
pub const CHROME_BOTTOM: u16 = 4;
pub const MIN_GAME_W: u16 = 60;
pub const MIN_GAME_H: u16 = 22;

/// Split terminal area into top banner, game, and bottom transport when replay chrome is active.
pub fn split_replay_areas(area: ratatui::layout::Rect) -> (ratatui::layout::Rect, ratatui::layout::Rect, ratatui::layout::Rect) {
    use ratatui::layout::Rect;

    let top_h = CHROME_TOP.min(area.height);
    let bottom_h = CHROME_BOTTOM.min(area.height.saturating_sub(top_h));
    let game_h = area.height.saturating_sub(top_h).saturating_sub(bottom_h);

    let top = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: top_h,
    };
    let game = Rect {
        x: area.x,
        y: area.y.saturating_add(top_h),
        width: area.width,
        height: game_h.max(1),
    };
    let bottom = Rect {
        x: area.x,
        y: area.y.saturating_add(top_h).saturating_add(game_h),
        width: area.width,
        height: bottom_h,
    };
    (top, game, bottom)
}

pub fn game_area_too_small(area: ratatui::layout::Rect) -> bool {
    area.width < MIN_GAME_W || area.height < MIN_GAME_H
}

pub fn agent_label(agent: &Option<ReplayAgent>) -> String {
    match agent {
        Some(a) if a.agent_type == "policy" => a
            .checkpoint
            .as_deref()
            .and_then(|p| Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .map(|s| format!("policy:{s}"))
            .unwrap_or_else(|| "policy".to_string()),
        Some(a) => a.agent_type.clone(),
        None => "unknown".to_string(),
    }
}

pub fn scan_replays(dir: &Path) -> Result<Vec<ReplayEntry>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let replay = match load_replay(&path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("skip {}: {e:#}", path.display());
                continue;
            }
        };
        let outcome = replay.outcome.unwrap_or(ReplayOutcome {
            won: false,
            score: 0,
            steps: replay.actions.len(),
        });
        entries.push(ReplayEntry {
            path,
            name: replay.name,
            seed: replay.seed,
            agent_label: agent_label(&replay.agent),
            ante_end: replay.config.ante_end,
            won: outcome.won,
            score: outcome.score,
            steps: outcome.steps,
            recorded_at: replay.recorded_at.unwrap_or_default(),
        });
    }
    entries.sort_by(|a, b| b.path.cmp(&a.path));
    Ok(entries)
}

/// Resolve replays directory: env `BALATRO_REPLAYS_DIR`, else walk up for `replays/`.
pub fn default_replays_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BALATRO_REPLAYS_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(cwd) = std::env::current_dir() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::apply_gym_action;

    #[test]
    fn split_replay_areas_never_zero_game() {
        for (w, h) in [(80, 30), (60, 20), (40, 15), (20, 10)] {
            let area = ratatui::layout::Rect::new(0, 0, w, h);
            let (_, game, _) = split_replay_areas(area);
            assert!(game.height >= 1, "game height zero at {w}x{h}");
            assert!(game.width >= 1, "game width zero at {w}x{h}");
        }
    }

    #[test]
    #[ignore = "regenerate tests/fixtures/minimal_replay.json"]
    fn gen_minimal_fixture() {
        let mut config = Config::default();
        config.ante_end = 1;
        config.seed = Some(10037);
        let mut game = Game::new(config);
        game.start();
        let mut actions = Vec::new();
        while !game.is_over() && actions.len() < 10_000 {
            let space = game.gen_action_space();
            let vec = space.to_vec();
            if space.is_empty() {
                break;
            }
            let valid: Vec<usize> = vec
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| (v == 1).then_some(i))
                .collect();
            if valid.is_empty() {
                break;
            }
            let i = valid[actions.len() % valid.len()];
            game.handle_action_index(i).expect("valid action");
            actions.push(i);
        }
        assert!(game.is_over(), "fixture generator failed to finish game");
        let replay = serde_json::json!({
            "format_version": 2,
            "seed": 10037,
            "config": { "ante_end": 1 },
            "agent": { "type": "expert" },
            "actions": actions,
            "outcome": {
                "won": game.is_over(),
                "score": game.score,
                "steps": actions.len(),
            }
        });
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_replay.json");
        fs::write(&path, serde_json::to_string_pretty(&replay).unwrap() + "\n").unwrap();
        eprintln!("wrote {} actions, score {}", actions.len(), game.score);
    }

    #[test]
    fn replay_name_helpers() {
        let entry = ReplayEntry {
            path: PathBuf::from("x.json"),
            name: None,
            seed: 10037,
            agent_label: "expert".to_string(),
            ante_end: 3,
            won: true,
            score: 100,
            steps: 50,
            recorded_at: String::new(),
        };
        assert_eq!(entry.display_name(), "10037_expert_ante3");
        assert!(!entry.has_custom_name());

        let named = ReplayEntry {
            name: Some("flush run".to_string()),
            ..entry.clone()
        };
        assert_eq!(named.display_name(), "flush run");
        assert!(named.has_custom_name());
    }

    #[test]
    fn set_and_clear_replay_name_roundtrip() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_replay.json");
        let backup = fs::read_to_string(&path).expect("read fixture");
        let result = (|| {
            set_replay_name(&path, "test rename")?;
            let replay = load_replay(&path)?;
            assert_eq!(replay.name.as_deref(), Some("test rename"));
            clear_replay_name(&path)?;
            let replay = load_replay(&path)?;
            assert!(replay.name.is_none());
            Ok::<(), anyhow::Error>(())
        })();
        fs::write(&path, backup).expect("restore fixture");
        result.expect("name roundtrip");
    }

    #[test]
    fn fixture_replay_matches_outcome() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_replay.json");
        let replay = load_replay(&path).expect("load fixture");
        let mut game = init_game_from_replay(&replay);
        for &action in &replay.actions {
            apply_gym_action(&mut game, action).expect("apply action");
        }
        let outcome = replay.outcome.expect("outcome");
        assert_eq!(game.score, outcome.score);
        assert!(game.is_over());
    }
}
