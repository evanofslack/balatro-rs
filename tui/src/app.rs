use balatro_rs::{card::Card, consumable::Consumable, game::Game, joker::Jokers};
use ratatui::layout::Rect;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WidgetId {
    Card(usize),
    ActionButton(usize),
    JokerSlot(usize),
    ConsumableSlot(usize),
    ShopJoker(usize),
    ShopConsumable(usize),
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
    ShopConsumables,
    ShopNextRound,
    BlindSelect,
    CashOutButton,
    TarotCards,
    TarotButtons,
}

#[derive(Debug, Clone)]
pub enum InspectTarget {
    Card(Card),
    Joker(Jokers),
    Consumable(Consumable),
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
}

pub struct AppState {
    pub game: Game,
    pub focus: FocusZone,
    pub cursor: usize,
    pub overlay: Option<Overlay>,
    pub deck_tab: DeckTab,
    pub run_info_tab: RunInfoTab,
    pub save_input: String,
    pub should_quit: bool,
    pub widget_rects: HashMap<WidgetId, Rect>,
}

impl AppState {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            focus: FocusZone::BlindSelect,
            cursor: 0,
            overlay: None,
            deck_tab: DeckTab::InDeck,
            run_info_tab: RunInfoTab::Deck,
            save_input: String::new(),
            should_quit: false,
            widget_rects: HashMap::new(),
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
            Stage::End(_) => FocusZone::CashOutButton,
        }
    }

    pub fn sync_focus_to_stage(&mut self) {
        self.focus = self.default_focus_for_stage();
        self.cursor = 0;
    }

    pub fn tab_next(&mut self) {
        use balatro_rs::stage::Stage;
        self.focus = match (&self.game.stage, &self.focus) {
            (Stage::Blind(_), FocusZone::Cards) => FocusZone::ActionButtons,
            (Stage::Blind(_), FocusZone::ActionButtons) => FocusZone::JokerStrip,
            (Stage::Blind(_), FocusZone::JokerStrip) => FocusZone::ConsumableStrip,
            (Stage::Blind(_), FocusZone::ConsumableStrip) => FocusZone::Cards,
            (Stage::Shop(), FocusZone::ShopJokers) => FocusZone::ShopConsumables,
            (Stage::Shop(), FocusZone::ShopConsumables) => FocusZone::ShopNextRound,
            (Stage::Shop(), FocusZone::ShopNextRound) => FocusZone::JokerStrip,
            (Stage::Shop(), FocusZone::JokerStrip) => FocusZone::ConsumableStrip,
            (Stage::Shop(), FocusZone::ConsumableStrip) => FocusZone::ShopJokers,
            (Stage::TarotHand(_), FocusZone::TarotCards) => FocusZone::TarotButtons,
            (Stage::TarotHand(_), FocusZone::TarotButtons) => FocusZone::TarotCards,
            _ => self.focus.clone(),
        };
        self.cursor = 0;
    }

    pub fn tab_prev(&mut self) {
        use balatro_rs::stage::Stage;
        self.focus = match (&self.game.stage, &self.focus) {
            (Stage::Blind(_), FocusZone::Cards) => FocusZone::ConsumableStrip,
            (Stage::Blind(_), FocusZone::ActionButtons) => FocusZone::Cards,
            (Stage::Blind(_), FocusZone::JokerStrip) => FocusZone::ActionButtons,
            (Stage::Blind(_), FocusZone::ConsumableStrip) => FocusZone::JokerStrip,
            (Stage::Shop(), FocusZone::ShopJokers) => FocusZone::ConsumableStrip,
            (Stage::Shop(), FocusZone::ConsumableStrip) => FocusZone::JokerStrip,
            (Stage::Shop(), FocusZone::JokerStrip) => FocusZone::ShopNextRound,
            (Stage::Shop(), FocusZone::ShopNextRound) => FocusZone::ShopConsumables,
            (Stage::Shop(), FocusZone::ShopConsumables) => FocusZone::ShopJokers,
            (Stage::TarotHand(_), FocusZone::TarotCards) => FocusZone::TarotButtons,
            (Stage::TarotHand(_), FocusZone::TarotButtons) => FocusZone::TarotCards,
            _ => self.focus.clone(),
        };
        self.cursor = 0;
    }
}
