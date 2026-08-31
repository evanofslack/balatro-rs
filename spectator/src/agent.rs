//! Python policy sidecar: serialize Game → action index → highlight + apply.

use crate::app::{AppState, FocusZone};
use anyhow::{anyhow, bail, Context, Result};
use balatro_rs::action::{Action, MoveDirection, SortBy};
use balatro_rs::stage::{Blind, Stage};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

pub struct AgentSidecar {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl AgentSidecar {
    /// Spawn `sh -c "<agent_cmd> --checkpoint <path>"`.
    pub fn spawn(agent_cmd: &str, checkpoint: &str) -> Result<Self> {
        let full = format!("{agent_cmd} --checkpoint {checkpoint}");
        Self::spawn_cmd(&full)
    }

    /// Spawn `sh -c "<agent_cmd>"` as-is (e.g. expert mode with `--expert`).
    pub fn spawn_cmd(agent_cmd: &str) -> Result<Self> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(agent_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("failed to spawn agent: {agent_cmd}"))?;

        let stdin = child.stdin.take().context("agent stdin missing")?;
        let stdout = child.stdout.take().context("agent stdout missing")?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    pub fn request_action(&mut self, game_json: &str) -> Result<Option<usize>> {
        writeln!(self.stdin, "{game_json}").context("write game JSON to agent")?;
        self.stdin.flush().context("flush agent stdin")?;

        let mut line = String::new();
        let n = self
            .stdout
            .read_line(&mut line)
            .context("read action from agent")?;
        if n == 0 {
            bail!("agent closed stdout");
        }
        parse_action_line(line.trim())
    }
}

impl Drop for AgentSidecar {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn parse_action_line(line: &str) -> Result<Option<usize>> {
    if let Some(err_start) = line.find("\"error\"") {
        let msg = line[err_start..].to_string();
        bail!("agent error: {msg}");
    }
    let key = "\"action\"";
    let pos = line
        .find(key)
        .ok_or_else(|| anyhow!("agent reply missing action: {line}"))?;
    let after = &line[pos + key.len()..];
    let after = after.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
    if after.starts_with("null") {
        return Ok(None);
    }
    let digits: String = after
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        bail!("could not parse action from: {line}");
    }
    Ok(Some(digits.parse()?))
}

fn card_focus_zone(stage: &Stage) -> FocusZone {
    match stage {
        Stage::TarotHand(_) => FocusZone::TarotCards,
        Stage::SpectralHand(_) => FocusZone::SpectralCards,
        _ => FocusZone::Cards,
    }
}

fn card_cursor(game: &balatro_rs::game::Game, card_id: usize) -> Option<usize> {
    game.available
        .cards()
        .iter()
        .position(|c| c.id == card_id)
}

/// Point focus/cursor at the widget that corresponds to a gym action index (0–108).
pub fn focus_for_action(app: &mut AppState, index: usize) {
    let space = app.game.gen_action_space();
    let action = match space.to_action(index, &app.game) {
        Ok(a) => a,
        Err(_) => return,
    };

    match action {
        Action::SelectCard(card) => {
            if let Some(pos) = card_cursor(&app.game, card.id) {
                app.focus = card_focus_zone(&app.game.stage);
                app.cursor = pos;
            }
        }
        Action::MoveCard(dir, card) => {
            if let Some(pos) = card_cursor(&app.game, card.id) {
                app.focus = card_focus_zone(&app.game.stage);
                app.cursor = match dir {
                    MoveDirection::Left => pos.saturating_sub(1),
                    MoveDirection::Right => pos + 1,
                };
            }
        }
        Action::Play() => {
            app.focus = FocusZone::ActionButtons;
            app.cursor = 0;
        }
        Action::Discard() => {
            app.focus = FocusZone::ActionButtons;
            app.cursor = 3;
        }
        Action::CashOut(_) => {
            app.focus = FocusZone::CashOutButton;
            app.cursor = 0;
        }
        Action::BuyJoker(joker) => {
            app.focus = FocusZone::ShopJokers;
            app.cursor = app
                .game
                .shop
                .jokers
                .iter()
                .position(|j| j == &joker)
                .unwrap_or(0);
        }
        Action::NextRound() => {
            app.focus = FocusZone::ShopNextRound;
            app.cursor = 0;
        }
        Action::SelectBlind(blind) => {
            app.focus = FocusZone::BlindSelect;
            app.cursor = blind_index(blind);
        }
        Action::SkipBlind(blind) => {
            app.focus = FocusZone::BlindSkip;
            app.cursor = blind_index(blind);
        }
        Action::BuyConsumable(consumable) => {
            app.focus = FocusZone::ShopJokers;
            let joker_count = app.game.shop.jokers.len();
            app.cursor = joker_count
                + app
                    .game
                    .shop
                    .consumables
                    .iter()
                    .position(|c| c == &consumable)
                    .unwrap_or(0);
        }
        Action::BuyVoucher(_) => {
            app.focus = FocusZone::ShopPacks;
            let pack_count = app.game.shop.packs.len();
            app.cursor = pack_count;
        }
        Action::BuyPlayingCard(card) => {
            app.focus = FocusZone::ShopJokers;
            let joker_count = app.game.shop.jokers.len();
            let consumable_count = app.game.shop.consumables.len();
            app.cursor = joker_count
                + consumable_count
                + app
                    .game
                    .shop
                    .cards
                    .iter()
                    .position(|c| c == &card)
                    .unwrap_or(0);
        }
        Action::UseConsumable(c) => {
            app.focus = FocusZone::ConsumableStrip;
            app.cursor = app
                .game
                .consumables
                .iter()
                .position(|x| *x == c)
                .unwrap_or(0);
        }
        Action::ApplyTarot() => {
            app.focus = FocusZone::TarotButtons;
            app.cursor = 0;
        }
        Action::SellJoker(slot) => {
            app.focus = FocusZone::JokerStrip;
            app.cursor = slot;
        }
        Action::SellConsumable(slot) => {
            app.focus = FocusZone::ConsumableStrip;
            app.cursor = slot;
        }
        Action::BuyPack(pack) => {
            app.focus = FocusZone::ShopPacks;
            app.cursor = app
                .game
                .shop
                .packs
                .iter()
                .position(|p| p == &pack)
                .unwrap_or(0);
        }
        Action::PickPackCard(content) => {
            app.focus = FocusZone::PackContents;
            app.cursor = app
                .game
                .open_pack
                .as_ref()
                .and_then(|p| p.contents.iter().position(|c| c == &content))
                .unwrap_or(0);
        }
        Action::SkipPack() => {
            app.focus = FocusZone::PackSkip;
            app.cursor = 0;
        }
        Action::SortHand(sort) => {
            app.focus = FocusZone::ActionButtons;
            app.cursor = match sort {
                SortBy::Rank => 1,
                SortBy::Suit => 2,
            };
        }
        Action::Reroll() => {
            app.focus = FocusZone::ShopReroll;
            app.cursor = 0;
        }
        Action::ApplySpectral() => {
            app.focus = FocusZone::SpectralButtons;
            app.cursor = 0;
        }
        _ => {}
    }
}

fn blind_index(blind: Blind) -> usize {
    match blind {
        Blind::Small => 0,
        Blind::Big => 1,
        Blind::Boss => 2,
    }
}

pub fn apply_gym_action(game: &mut balatro_rs::game::Game, index: usize) -> Result<()> {
    game.handle_action_index(index)
        .map_err(|e| anyhow!("handle_action_index({index}) failed: {e:?}"))
}

pub fn begin_highlight_action(app: &mut AppState, action: usize, highlight_ms: u64) {
    focus_for_action(app, action);
    app.ai_action = Some(action);
    app.pending_ai_action = Some(action);
    app.highlight_until = Some(std::time::Instant::now() + Duration::from_millis(highlight_ms));
}

pub fn autoplay_step(
    app: &mut AppState,
    agent: &mut AgentSidecar,
    highlight_ms: u64,
) -> Result<bool> {
    if app.game.is_over() {
        app.autoplay_paused = true;
        return Ok(false);
    }

    let json = app
        .game
        .to_json()
        .map_err(|e| anyhow!("serialize game: {e}"))?;
    let Some(action) = agent.request_action(&json)? else {
        app.autoplay_paused = true;
        return Ok(false);
    };

    begin_highlight_action(app, action, highlight_ms);
    Ok(true)
}

pub fn replay_step(
    app: &mut AppState,
    driver: &mut crate::replay::ReplayDriver,
    highlight_ms: u64,
) -> Result<bool> {
    if app.game.is_over() {
        app.autoplay_paused = true;
        return Ok(false);
    }
    let Some(action) = driver.next_action() else {
        app.autoplay_paused = true;
        return Ok(false);
    };
    begin_highlight_action(app, action, highlight_ms);
    Ok(true)
}

pub fn finish_pending_action(app: &mut AppState) -> Result<()> {
    let Some(action) = app.pending_ai_action.take() else {
        return Ok(());
    };
    apply_gym_action(&mut app.game, action)?;
    app.ai_action = None;
    app.highlight_until = None;
    app.sync_focus_to_stage();
    Ok(())
}
