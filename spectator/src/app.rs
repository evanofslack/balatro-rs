use balatro_rs::{
    action::SortBy, card::Card, consumable::Consumable, game::Game, joker::Jokers, pack::Pack,
    tag::Tag, voucher::Voucher,
};
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::replay::{ReplayDriver, ReplayEntry, ReplayFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportButton {
    RewindFast,
    RewindStep,
    Pause,
    ForwardStep,
    ForwardFast,
    Exit,
}

#[derive(Clone)]
pub struct ReplaySession {
    pub file: ReplayFile,
    pub source_path: PathBuf,
}

pub struct ReplayChrome {
    pub forward_mult: u8,
    pub rewind_mult: u8,
    pub rewinding: bool,
    pub button_rects: HashMap<TransportButton, Rect>,
}

impl ReplayChrome {
    pub fn new() -> Self {
        Self {
            forward_mult: 1,
            rewind_mult: 1,
            rewinding: false,
            button_rects: HashMap::new(),
        }
    }

    pub fn cycle_forward_mult(&mut self) {
        self.forward_mult = cycle_mult(self.forward_mult);
        self.rewinding = false;
    }

    pub fn cycle_rewind_mult(&mut self) {
        self.rewind_mult = cycle_mult(self.rewind_mult);
        self.rewinding = true;
    }
}

fn cycle_mult(current: u8) -> u8 {
    match current {
        1 => 2,
        2 => 4,
        _ => 1,
    }
}

pub const REPLAY_COMPLETE_DELAY_MS: u128 = 1500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Library,
    Spectating,
    Manual,
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
    SpectralButton(usize),
    OverlayButton(usize),
    DeckTab(usize),
    RunInfoTab(usize),
    SidebarButton(usize),
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
    SpectralCards,
    SpectralButtons,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopSlot {
    Joker(usize),
    Consumable(usize),
    Pack(usize),
    Voucher,
    PlayingCard(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeckTab {
    Remaining,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunInfoTab {
    PokerHands,
    Blinds,
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
    ShopBuy(ShopSlot),
    Deck,
    Options,
    PackPick(usize),
}

pub struct LibraryState {
    pub replays_dir: PathBuf,
    pub entries: Vec<ReplayEntry>,
    pub selected: usize,
    pub confirm_delete: bool,
    pub rename_active: bool,
    pub rename_input: String,
    pub rename_error: Option<String>,
    pub row_rects: Vec<Rect>,
}

pub const LIBRARY_PAGE_SIZE: usize = 25;

impl LibraryState {
    pub fn page(&self) -> usize {
        self.selected / LIBRARY_PAGE_SIZE
    }

    pub fn total_pages(&self) -> usize {
        let n = self.entries.len();
        if n == 0 {
            1
        } else {
            n.div_ceil(LIBRARY_PAGE_SIZE)
        }
    }

    pub fn page_start(&self) -> usize {
        self.page() * LIBRARY_PAGE_SIZE
    }

    pub fn page_end(&self) -> usize {
        (self.page_start() + LIBRARY_PAGE_SIZE).min(self.entries.len())
    }

    pub fn prev_page(&mut self) {
        if self.page() == 0 {
            return;
        }
        self.selected = self.page_start().saturating_sub(LIBRARY_PAGE_SIZE);
    }

    pub fn next_page(&mut self) {
        let next = self.page() + 1;
        if next >= self.total_pages() {
            return;
        }
        self.selected = next * LIBRARY_PAGE_SIZE;
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
    }
}

pub struct AppState {
    pub mode: AppMode,
    pub library: Option<LibraryState>,
    pub replay_driver: Option<ReplayDriver>,
    pub replay_session: Option<ReplaySession>,
    pub replay_chrome: Option<ReplayChrome>,
    pub replay_complete_at: Option<std::time::Instant>,
    pub last_rewind_at: Option<std::time::Instant>,
    pub replays_dir: PathBuf,
    pub speed_ms: u64,
    pub game: Game,
    pub focus: FocusZone,
    pub cursor: usize,
    pub overlay: Option<Overlay>,
    pub overlay_cursor: usize,
    pub deck_tab: DeckTab,
    pub run_info_tab: RunInfoTab,
    pub save_input: String,
    pub should_quit: bool,
    pub clear_screen: bool,
    pub widget_rects: HashMap<WidgetId, Rect>,
    pub sort_mode: SortBy,
    pub agent_mode: bool,
    pub autoplay_paused: bool,
    pub step_once: bool,
    pub ai_action: Option<usize>,
    pub pending_ai_action: Option<usize>,
    pub highlight_until: Option<std::time::Instant>,
}

impl AppState {
    pub fn new(game: Game) -> Self {
        Self {
            mode: AppMode::Manual,
            library: None,
            replay_driver: None,
            replay_session: None,
            replay_chrome: None,
            replay_complete_at: None,
            last_rewind_at: None,
            replays_dir: crate::replay::default_replays_dir(),
            speed_ms: 400,
            game,
            focus: FocusZone::BlindSelect,
            cursor: 0,
            overlay: None,
            overlay_cursor: 0,
            deck_tab: DeckTab::Remaining,
            run_info_tab: RunInfoTab::PokerHands,
            save_input: String::new(),
            should_quit: false,
            clear_screen: false,
            widget_rects: HashMap::new(),
            sort_mode: SortBy::Rank,
            agent_mode: false,
            autoplay_paused: false,
            step_once: false,
            ai_action: None,
            pending_ai_action: None,
            highlight_until: None,
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

    pub fn is_json_replay(&self) -> bool {
        self.replay_chrome.is_some()
    }

    pub fn default_focus_for_stage(&self) -> FocusZone {
        use balatro_rs::stage::Stage;
        match &self.game.stage {
            Stage::PreBlind() => FocusZone::BlindSelect,
            Stage::Blind(_) => FocusZone::Cards,
            Stage::PostBlind() => FocusZone::CashOutButton,
            Stage::Shop() => FocusZone::ShopJokers,
            Stage::TarotHand(_) => FocusZone::TarotCards,
            Stage::SpectralHand(_) => FocusZone::SpectralCards,
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
            (Stage::SpectralHand(_), FocusZone::SpectralCards) => FocusZone::SpectralButtons,
            (Stage::SpectralHand(_), FocusZone::SpectralButtons) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::SpectralCards
                }
            }
            (Stage::SpectralHand(_), FocusZone::JokerStrip) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else {
                    FocusZone::SpectralCards
                }
            }
            (Stage::SpectralHand(_), FocusZone::ConsumableStrip) => FocusZone::SpectralCards,
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
            (Stage::SpectralHand(_), FocusZone::SpectralCards) => {
                if has_consumables {
                    FocusZone::ConsumableStrip
                } else if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::SpectralButtons
                }
            }
            (Stage::SpectralHand(_), FocusZone::SpectralButtons) => FocusZone::SpectralCards,
            (Stage::SpectralHand(_), FocusZone::JokerStrip) => FocusZone::SpectralButtons,
            (Stage::SpectralHand(_), FocusZone::ConsumableStrip) => {
                if has_jokers {
                    FocusZone::JokerStrip
                } else {
                    FocusZone::SpectralButtons
                }
            }
            _ => self.focus.clone(),
        };
        if !preserve_cursor {
            self.cursor = 0;
        }
    }
}
