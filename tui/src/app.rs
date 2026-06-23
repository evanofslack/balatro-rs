use balatro_rs::{action::SortBy, card::Card, consumable::Consumable, game::Game, joker::Jokers, pack::Pack};
use ratatui::layout::Rect;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WidgetId {
    Card(usize),
    ActionButton(usize),
    JokerSlot(usize),
    ConsumableSlot(usize),
    ShopJoker(usize),
    ShopPack(usize),
    PackContent(usize),
    SkipPackButton,
    BlindOption(usize),
    CashOutButton,
    NextRoundButton,
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
    ShopNextRound,
    BlindSelect,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeckTab {
    InDeck,
    InHand,
    Discarded,
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
}

impl AppState {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            focus: FocusZone::BlindSelect,
            cursor: 0,
            overlay: None,
            overlay_cursor: 0,
            deck_tab: DeckTab::InDeck,
            run_info_tab: RunInfoTab::Deck,
            save_input: String::new(),
            should_quit: false,
            widget_rects: HashMap::new(),
            sort_mode: SortBy::Rank,
        }
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
        self.focus = match (&self.game.stage, &self.focus) {
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
            (Stage::Shop(), FocusZone::ShopPacks) => FocusZone::ShopNextRound,
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
        self.cursor = 0;
    }

    pub fn tab_prev(&mut self) {
        use balatro_rs::stage::Stage;
        let has_jokers = !self.game.jokers.is_empty();
        let has_consumables = !self.game.consumables.is_empty();
        self.focus = match (&self.game.stage, &self.focus) {
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
            (Stage::Shop(), FocusZone::ShopNextRound) => FocusZone::ShopPacks,
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
        self.cursor = 0;
    }
}
