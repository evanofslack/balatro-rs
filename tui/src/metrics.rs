//! Engine telemetry for the TUI's metrics overlay (`m`).
//!
//! Everything here is measured in the TUI layer, not `core` — the engine
//! stays untouched so these numbers reflect exactly what an RL env would
//! pay per step. Timings come from `Instant`, so they include allocation
//! and cloning, which is the honest cost of a `handle_action` call.

use balatro_rs::score::ScoreTrace;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// How many recent samples percentiles are computed over. Small enough to
/// sort on every frame without caring, large enough to be meaningful.
const WINDOW: usize = 256;

/// Rolling stats for one repeatedly-measured operation.
#[derive(Debug, Clone, Default)]
pub struct Timing {
    pub count: u64,
    pub total_ns: u128,
    pub min_ns: u64,
    pub max_ns: u64,
    pub last_ns: u64,
    /// Most recent `WINDOW` samples, oldest first.
    recent: Vec<u64>,
}

impl Timing {
    pub fn record(&mut self, ns: u64) {
        self.count += 1;
        self.total_ns += ns as u128;
        self.last_ns = ns;
        self.max_ns = self.max_ns.max(ns);
        self.min_ns = if self.count == 1 {
            ns
        } else {
            self.min_ns.min(ns)
        };
        if self.recent.len() == WINDOW {
            self.recent.remove(0);
        }
        self.recent.push(ns);
    }

    pub fn mean_ns(&self) -> u64 {
        if self.count == 0 {
            return 0;
        }
        (self.total_ns / self.count as u128) as u64
    }

    /// `pct` in 0..=100, over the recent window.
    pub fn percentile_ns(&self, pct: usize) -> u64 {
        if self.recent.is_empty() {
            return 0;
        }
        let mut sorted = self.recent.clone();
        sorted.sort_unstable();
        let idx = (sorted.len() - 1) * pct.min(100) / 100;
        sorted[idx]
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Everything captured about the most recent action.
#[derive(Debug, Clone)]
pub struct LastAction {
    pub label: String,
    pub kind: &'static str,
    pub ns: u64,
    pub ok: bool,
    pub error: Option<String>,
    /// Legal-move counts *before* the action ran — the branching factor an
    /// agent actually faced at that step.
    pub legal_actions: usize,
    pub unmasked: usize,
    pub mask_size: usize,
    pub score_delta: i64,
    pub money_delta: i64,
    pub stage: String,
}

impl LastAction {
    /// Fraction of the fixed action space that was legal — the number a
    /// masked-policy agent cares about.
    pub fn mask_density(&self) -> f64 {
        if self.mask_size == 0 {
            return 0.0;
        }
        self.unmasked as f64 / self.mask_size as f64
    }
}

/// A scored hand, kept so the overlay can show where the number came from.
#[derive(Debug, Clone)]
pub struct LastHand {
    pub rank: String,
    pub score: usize,
    /// Time to evaluate scoring alone, separate from the enclosing action.
    pub score_ns: u64,
    pub trace: ScoreTrace,
}

#[derive(Debug)]
pub struct Metrics {
    /// `handle_action` across every action kind.
    pub handle_action: Timing,
    /// ...and broken out per action kind, since a `Play` costs far more
    /// than a `SelectCard` and a single average hides that.
    pub by_kind: BTreeMap<&'static str, Timing>,
    pub gen_actions: Timing,
    pub gen_action_space: Timing,
    pub score: Timing,
    pub render: Timing,

    pub last_action: Option<LastAction>,
    pub last_hand: Option<LastHand>,

    pub actions_ok: u64,
    pub actions_err: u64,
    started: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            handle_action: Timing::default(),
            by_kind: BTreeMap::new(),
            gen_actions: Timing::default(),
            gen_action_space: Timing::default(),
            score: Timing::default(),
            render: Timing::default(),
            last_action: None,
            last_hand: None,
            actions_ok: 0,
            actions_err: 0,
            started: Instant::now(),
        }
    }
}

impl Metrics {
    pub fn record_action(&mut self, kind: &'static str, ns: u64, ok: bool) {
        self.handle_action.record(ns);
        self.by_kind.entry(kind).or_default().record(ns);
        if ok {
            self.actions_ok += 1;
        } else {
            self.actions_err += 1;
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// Actions per second of wall-clock, including think time — this is
    /// the human-play rate, not the engine's ceiling.
    pub fn actions_per_sec(&self) -> f64 {
        let secs = self.elapsed().as_secs_f64();
        if secs <= 0.0 {
            return 0.0;
        }
        (self.actions_ok + self.actions_err) as f64 / secs
    }

    /// What the engine alone could sustain, from mean `handle_action` plus
    /// one `gen_actions` per step — the number to extrapolate training
    /// throughput from.
    pub fn engine_steps_per_sec(&self) -> f64 {
        let per_step = self.handle_action.mean_ns() + self.gen_actions.mean_ns();
        if per_step == 0 {
            return 0.0;
        }
        1e9 / per_step as f64
    }

    /// Share of wall-clock actually spent inside the engine.
    pub fn engine_time_fraction(&self) -> f64 {
        let total = self.elapsed().as_nanos();
        if total == 0 {
            return 0.0;
        }
        let engine = self.handle_action.total_ns
            + self.gen_actions.total_ns
            + self.gen_action_space.total_ns;
        engine as f64 / total as f64
    }
}

/// Stable short name for an action, used as the per-kind timing key.
/// Deliberately the variant name only — the payload goes in the label.
pub fn action_kind(action: &balatro_rs::action::Action) -> &'static str {
    use balatro_rs::action::Action::*;
    match action {
        SelectCard(_) => "SelectCard",
        DeselectCard(_) => "DeselectCard",
        MoveCard(_, _) => "MoveCard",
        Play() => "Play",
        Discard() => "Discard",
        CashOut(_) => "CashOut",
        BuyJoker(_) => "BuyJoker",
        BuyConsumable(_) => "BuyConsumable",
        BuyVoucher(_) => "BuyVoucher",
        BuyPlayingCard(_) => "BuyPlayingCard",
        UseConsumable(_) => "UseConsumable",
        NextRound() => "NextRound",
        SelectBlind(_) => "SelectBlind",
        SkipBlind(_) => "SkipBlind",
        ApplyTarot() => "ApplyTarot",
        SellJoker(_) => "SellJoker",
        SellConsumable(_) => "SellConsumable",
        BuyPack(_) => "BuyPack",
        PickPackCard(_) => "PickPackCard",
        SkipPack() => "SkipPack",
        SortHand(_) => "SortHand",
        Reroll() => "Reroll",
    }
}

/// `1_234_567` -> `1.23ms`. Keeps three significant figures so small
/// differences stay visible.
pub fn fmt_ns(ns: u64) -> String {
    if ns < 1_000 {
        format!("{ns}ns")
    } else if ns < 1_000_000 {
        format!("{:.2}µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.2}ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", ns as f64 / 1_000_000_000.0)
    }
}

pub fn fmt_count(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.2}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}k", n / 1_000.0)
    } else {
        format!("{n:.0}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_tracks_min_max_mean() {
        let mut t = Timing::default();
        for ns in [100, 300, 200] {
            t.record(ns);
        }
        assert_eq!(t.count, 3);
        assert_eq!(t.min_ns, 100);
        assert_eq!(t.max_ns, 300);
        assert_eq!(t.mean_ns(), 200);
        assert_eq!(t.last_ns, 200);
    }

    #[test]
    fn test_percentiles_use_the_recent_window() {
        let mut t = Timing::default();
        for ns in 1..=100 {
            t.record(ns);
        }
        assert_eq!(t.percentile_ns(0), 1);
        assert_eq!(t.percentile_ns(100), 100);
        // p50 of 1..=100 lands mid-window; exact index is off-by-one
        // tolerant by design, so just bracket it.
        let p50 = t.percentile_ns(50);
        assert!((45..=55).contains(&p50), "p50 was {p50}");
    }

    #[test]
    fn test_window_is_bounded() {
        let mut t = Timing::default();
        for ns in 0..(WINDOW as u64 * 3) {
            t.record(ns);
        }
        assert_eq!(t.recent.len(), WINDOW);
        // Oldest samples evicted, so the window's min has moved up.
        assert_eq!(t.percentile_ns(0), WINDOW as u64 * 2);
        // ...while the all-time min is still the first sample recorded.
        assert_eq!(t.min_ns, 0);
    }

    #[test]
    fn test_fmt_ns_scales() {
        assert_eq!(fmt_ns(999), "999ns");
        assert_eq!(fmt_ns(1_500), "1.50µs");
        assert_eq!(fmt_ns(2_500_000), "2.50ms");
        assert_eq!(fmt_ns(3_000_000_000), "3.00s");
    }

    #[test]
    fn test_mask_density() {
        let la = LastAction {
            label: "Play".into(),
            kind: "Play",
            ns: 1,
            ok: true,
            error: None,
            legal_actions: 4,
            unmasked: 27,
            mask_size: 108,
            score_delta: 0,
            money_delta: 0,
            stage: "Blind".into(),
        };
        assert!((la.mask_density() - 0.25).abs() < f64::EPSILON);
    }
}
