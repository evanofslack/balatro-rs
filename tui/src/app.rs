use crate::metrics::{action_kind, LastAction, LastHand, Metrics};
use balatro_rs::{
    action::{Action, SortBy},
    card::Card,
    consumable::Consumable,
    error::GameError,
    game::Game,
    hand::SelectHand,
    joker::Jokers,
    pack::Pack,
    tag::Tag,
    voucher::Voucher,
};
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::time::Instant;

/// Saturating so a pathological clock can't panic the UI.
pub(crate) fn elapsed_ns(t: Instant) -> u64 {
    t.elapsed().as_nanos().min(u64::MAX as u128) as u64
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WidgetId {
    Card(usize),
    ActionButton(usize),
    JokerSlot(usize),
    ConsumableSlot(usize),
    ShopJoker(usize),
    ShopConsumable(usize),
    ShopCard(usize),
    ShopPack(usize),
    ShopVoucher,
    PackContent(usize),
    SkipPackButton,
    BlindOption(usize),
    BlindSkipOption(usize),
    CashOutButton,
    NextRoundButton,
    RerollButton,
    TarotButton(usize),
    OverlayButton(usize),
    DeckTab(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusZone {
    Cards,
    ActionButtons,
    JokerStrip,
    ConsumableStrip,
    ShopJokers,
    ShopPacks,
    ShopReroll,
    ShopNextRound,
    BlindSelect,
    BlindSkip,
    PackContents,
    PackSkip,
    CashOutButton,
    TarotCards,
    TarotButtons,
}

#[derive(Debug, Clone)]
pub enum InspectTarget {
    Card(Card),
    Joker(Jokers),
    Consumable(Consumable),
    Pack(Pack),
    Tag(Tag),
    Voucher(Voucher),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeckTab {
    Remaining,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunInfoTab {
    Deck,
    PokerHands,
    Vouchers,
}

#[derive(Debug, Clone)]
pub enum Overlay {
    Inspect(InspectTarget),
    RunInfo,
    Controls,
    Save,
    Consumable(usize),
    Joker(usize),
    Metrics,
}

pub struct AppState {
    pub game: Game,
    pub focus: FocusZone,
    pub cursor: usize,
    pub overlay: Option<Overlay>,
    pub overlay_cursor: usize,
    pub deck_tab: DeckTab,
    pub run_info_tab: RunInfoTab,
    pub save_input: String,
    pub should_quit: bool,
    pub widget_rects: HashMap<WidgetId, Rect>,
    pub sort_mode: SortBy,
    pub metrics: Metrics,
}

impl AppState {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            focus: FocusZone::BlindSelect,
            cursor: 0,
            overlay: None,
            overlay_cursor: 0,
            deck_tab: DeckTab::Remaining,
            run_info_tab: RunInfoTab::Deck,
            save_input: String::new(),
            should_quit: false,
            widget_rects: HashMap::new(),
            sort_mode: SortBy::Rank,
            metrics: Metrics::default(),
        }
    }

    /// The single path every input handler takes to mutate the game, so
    /// timings and per-step stats are captured for the metrics overlay
    /// without each call site having to remember. Behaves exactly like
    /// `game.handle_action` otherwise.
    pub fn act(&mut self, action: Action) -> Result<(), GameError> {
        // Branching factor and mask density as the agent would have seen
        // them, i.e. before the action is applied.
        let t = Instant::now();
        let legal_actions = self.game.gen_actions().count();
        self.metrics.gen_actions.record(elapsed_ns(t));

        let t = Instant::now();
        let space = self.game.gen_action_space().to_vec();
        self.metrics.gen_action_space.record(elapsed_ns(t));
        let unmasked = space.iter().filter(|v| **v == 1).count();

        if matches!(action, Action::Play()) {
            self.capture_score_trace();
        }

        let kind = action_kind(&action);
        let label = action.to_string();
        let stage = format!("{:?}", self.game.stage);
        let score_before = self.game.score as i64;
        let money_before = self.game.money as i64;

        let t = Instant::now();
        let result = self.game.handle_action(action);
        let ns = elapsed_ns(t);

        self.metrics.record_action(kind, ns, result.is_ok());
        self.metrics.last_action = Some(LastAction {
            label,
            kind,
            ns,
            ok: result.is_ok(),
            error: result.as_ref().err().map(|e| e.to_string()),
            legal_actions,
            unmasked,
            mask_size: space.len(),
            score_delta: self.game.score as i64 - score_before,
            money_delta: self.game.money as i64 - money_before,
            stage,
        });
        result
    }

    /// Scores the current selection on a throwaway clone to capture a
    /// `ScoreTrace` — `play_selected` uses the untraced path, and we don't
    /// want the overlay changing what the real game does.
    fn capture_score_trace(&mut self) {
        let selected = SelectHand::new(self.game.available.selected());
        let Ok(made) = selected.best_hand() else {
            return;
        };
        let rank = format!("{:?}", made.rank);
        let mut probe = self.game.clone();
        let t = Instant::now();
        let (score, trace) = probe.calc_score_traced(made);
        let score_ns = elapsed_ns(t);
        self.metrics.score.record(score_ns);
        self.metrics.last_hand = Some(LastHand {
            rank,
            score,
            score_ns,
            trace,
        });
    }

    pub fn close_overlay(&mut self) {
        self.overlay = None;
    }

    pub fn open_save(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.save_input = format!("game_{}.json", ts);
        self.overlay = Some(Overlay::Save);
    }

    pub fn default_focus_for_stage(&self) -> FocusZone {
        use balatro_rs::stage::Stage;
        match &self.game.stage {
            Stage::PreBlind() => FocusZone::BlindSelect,
            Stage::Blind(_) => FocusZone::Cards,
            Stage::PostBlind() => FocusZone::CashOutButton,
            Stage::Shop() => FocusZone::ShopJokers,
            Stage::TarotHand(_) => FocusZone::TarotCards,
            Stage::PackOpen() => FocusZone::PackContents,
            Stage::End(_) => FocusZone::CashOutButton,
        }
    }

    pub fn sync_focus_to_stage(&mut self) {
        self.focus = self.default_focus_for_stage();
        self.cursor = self.default_cursor_for_focus();
    }

    fn default_cursor_for_focus(&self) -> usize {
        use balatro_rs::stage::{Blind, Stage};
        if matches!(self.game.stage, Stage::PreBlind()) {
            // Start on the first selectable blind so the cursor isn't on a cleared one
            let blinds = [Blind::Small, Blind::Big, Blind::Boss];
            return blinds
                .iter()
                .enumerate()
                .find(|(_, b)| {
                    self.game.gen_actions().any(
                        |a| matches!(a, balatro_rs::action::Action::SelectBlind(x) if &x == *b),
                    )
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
        0
    }

    pub fn tab_next(&mut self) {
        use balatro_rs::stage::Stage;
        let has_jokers = !self.game.jokers.is_empty();
        let has_consumables = !self.game.consumables.is_empty();
        // BlindSelect/BlindSkip share cursor semantics (which blind column) —
        // toggling between them must never disturb it, unlike other zone pairs
        // where cursor 0 is a sensible fresh start.
        let preserve_cursor = matches!(
            (&self.game.stage, &self.focus),
            (Stage::PreBlind(), FocusZone::BlindSelect) | (Stage::PreBlind(), FocusZone::BlindSkip)
        );
        self.focus = match (&self.game.stage, &self.focus) {
            (Stage::PreBlind(), FocusZone::BlindSelect) => FocusZone::BlindSkip,
            (Stage::Blind(_), FocusZone::Cards) => FocusZone::ActionButtons,
            (Stage::Blind(_), FocusZone::ActionButtons) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::Cards
                }
            }
            (Stage::Blind(_), FocusZone::JokerStrip) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::Cards
                }
            }
            (Stage::Blind(_), FocusZone::ConsumableStrip) => FocusZone::Cards,
            (Stage::PostBlind(), FocusZone::CashOutButton) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::CashOutButton
                }
            }
            (Stage::PostBlind(), FocusZone::JokerStrip) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::CashOutButton
                }
            }
            (Stage::PostBlind(), FocusZone::ConsumableStrip) => FocusZone::CashOutButton,
            (Stage::Shop(), FocusZone::ShopJokers) => FocusZone::ShopPacks,
            (Stage::Shop(), FocusZone::ShopPacks) => FocusZone::ShopReroll,
            (Stage::Shop(), FocusZone::ShopReroll) => FocusZone::ShopNextRound,
            (Stage::Shop(), FocusZone::ShopNextRound) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::ShopJokers
                }
            }
            (Stage::Shop(), FocusZone::JokerStrip) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::ShopJokers
                }
            }
            (Stage::Shop(), FocusZone::ConsumableStrip) => FocusZone::ShopJokers,
            (Stage::PackOpen(), FocusZone::PackContents) => FocusZone::PackSkip,
            (Stage::PackOpen(), FocusZone::PackSkip) => FocusZone::PackContents,
            (Stage::TarotHand(_), FocusZone::TarotCards) => FocusZone::TarotButtons,
            (Stage::TarotHand(_), FocusZone::TarotButtons) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::TarotCards
                }
            }
            (Stage::TarotHand(_), FocusZone::JokerStrip) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::TarotCards
                }
            }
            (Stage::TarotHand(_), FocusZone::ConsumableStrip) => FocusZone::TarotCards,
            _ => self.focus.clone(),
        };
        if !preserve_cursor {
            self.cursor = 0;
        }
    }

    pub fn tab_prev(&mut self) {
        use balatro_rs::stage::Stage;
        let has_jokers = !self.game.jokers.is_empty();
        let has_consumables = !self.game.consumables.is_empty();
        let preserve_cursor = matches!(
            (&self.game.stage, &self.focus),
            (Stage::PreBlind(), FocusZone::BlindSelect) | (Stage::PreBlind(), FocusZone::BlindSkip)
        );
        self.focus = match (&self.game.stage, &self.focus) {
            (Stage::PreBlind(), FocusZone::BlindSkip) => FocusZone::BlindSelect,
            (Stage::Blind(_), FocusZone::Cards) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::ActionButtons
                }
            }
            (Stage::Blind(_), FocusZone::ActionButtons) => FocusZone::Cards,
            (Stage::Blind(_), FocusZone::JokerStrip) => FocusZone::ActionButtons,
            (Stage::Blind(_), FocusZone::ConsumableStrip) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::ActionButtons
                }
            }
            (Stage::PostBlind(), FocusZone::CashOutButton) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::CashOutButton
                }
            }
            (Stage::PostBlind(), FocusZone::ConsumableStrip) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::CashOutButton
                }
            }
            (Stage::PostBlind(), FocusZone::JokerStrip) => FocusZone::CashOutButton,
            (Stage::Shop(), FocusZone::ShopJokers) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::ShopNextRound
                }
            }
            (Stage::Shop(), FocusZone::ShopPacks) => FocusZone::ShopJokers,
            (Stage::Shop(), FocusZone::ShopReroll) => FocusZone::ShopPacks,
            (Stage::Shop(), FocusZone::ShopNextRound) => FocusZone::ShopReroll,
            (Stage::Shop(), FocusZone::JokerStrip) => FocusZone::ShopNextRound,
            (Stage::Shop(), FocusZone::ConsumableStrip) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::ShopNextRound
                }
            }
            (Stage::PackOpen(), FocusZone::PackContents) => FocusZone::PackSkip,
            (Stage::PackOpen(), FocusZone::PackSkip) => FocusZone::PackContents,
            (Stage::TarotHand(_), FocusZone::TarotCards) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::TarotButtons
                }
            }
            (Stage::TarotHand(_), FocusZone::TarotButtons) => FocusZone::TarotCards,
            (Stage::TarotHand(_), FocusZone::JokerStrip) => FocusZone::TarotButtons,
            (Stage::TarotHand(_), FocusZone::ConsumableStrip) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::TarotButtons
                }
            }
            _ => self.focus.clone(),
        };
        if !preserve_cursor {
            self.cursor = 0;
        }
    }
}
