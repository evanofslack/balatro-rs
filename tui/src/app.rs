use crate::library::LibraryState;
use crate::paths::{
    default_record_path, ensure_replays_dir, recording_enabled, set_recording_enabled,
};
use crate::record::ReplayWriter;
use crate::replay::{ReplayReader, ReplaySession, ReplayStep};
use balatro_rs::action::Action;
use balatro_rs::config::Config;
use balatro_rs::replay_log::{loggable_move, ReplayResult, ReplaySource};
use balatro_rs::{
    action::SortBy, card::Card, consumable::Consumable, game::Game, joker::Jokers, pack::Pack,
    tag::Tag, voucher::Voucher,
};
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

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
    TitleButton(usize),
    LibraryRow(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleButton {
    Play = 0,
    Library = 1,
}

impl TitleButton {
    pub fn from_index(i: usize) -> Self {
        match i {
            1 => Self::Library,
            _ => Self::Play,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Play => "PLAY",
            Self::Library => "LIBRARY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    Library,
    Playing,
}

/// Where Esc/q returns when leaving the current screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnTarget {
    Quit,
    Title,
    Library,
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
    pub recorder: Option<ReplayWriter>,
    pub replay: Option<ReplaySession>,
    pub screen: Screen,
    pub title_focus: TitleButton,
    pub record_runs: bool,
    pub title_status: String,
    pub return_target: ReturnTarget,
    pub library: Option<LibraryState>,
    pub replays_dir: PathBuf,
    pub clear_screen: bool,
    /// True when the library was entered via `--library` (Esc/q on the list exits).
    pub library_is_root: bool,
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
            run_info_tab: RunInfoTab::PokerHands,
            save_input: String::new(),
            should_quit: false,
            widget_rects: HashMap::new(),
            sort_mode: SortBy::Rank,
            recorder: None,
            replay: None,
            screen: Screen::Playing,
            title_focus: TitleButton::Play,
            record_runs: true,
            title_status: String::new(),
            return_target: ReturnTarget::Quit,
            library: None,
            replays_dir: PathBuf::from("replays"),
            clear_screen: false,
            library_is_root: false,
        }
    }

    pub fn new_title(replays_dir: PathBuf) -> Self {
        let mut app = Self::new(Game::new(Config::default()));
        app.screen = Screen::Title;
        app.return_target = ReturnTarget::Quit;
        match recording_enabled(&replays_dir) {
            Ok(enabled) => app.record_runs = enabled,
            Err(err) => app.title_status = format!("read recording setting: {err}"),
        }
        app.replays_dir = replays_dir;
        app
    }

    pub fn open_title(&mut self) {
        if let Err(err) = self.finish_recording() {
            self.title_status = format!("finish recording: {err:#}");
        }
        self.screen = Screen::Title;
        self.title_focus = TitleButton::Play;
        self.return_target = ReturnTarget::Quit;
        self.overlay = None;
        self.replay = None;
        self.library = None;
        self.library_is_root = false;
        self.clear_screen = true;
    }

    pub fn start_play_from_title(&mut self) -> anyhow::Result<()> {
        let mut game = Game::new(Config::default());
        game.start();
        let recorder = if self.record_runs {
            ensure_replays_dir(&self.replays_dir)?;
            let path = default_record_path(&self.replays_dir, game.seed);
            Some(ReplayWriter::create(&path, &game)?)
        } else {
            None
        };
        self.game = game;
        self.screen = Screen::Playing;
        self.return_target = ReturnTarget::Title;
        self.overlay = None;
        self.recorder = recorder;
        self.replay = None;
        self.library = None;
        self.clear_screen = true;
        self.sync_focus_to_stage();
        Ok(())
    }

    pub fn toggle_recording(&mut self) {
        let enabled = !self.record_runs;
        match set_recording_enabled(&self.replays_dir, enabled) {
            Ok(()) => {
                self.record_runs = enabled;
                self.title_status.clear();
            }
            Err(err) => self.title_status = format!("save recording setting: {err}"),
        }
    }

    pub fn finish_recording(&mut self) -> anyhow::Result<()> {
        if self.game.is_over() {
            if let Some(recorder) = self.recorder.as_mut() {
                recorder.record_outcome(&self.game)?;
            }
            self.recorder = None;
        } else if let Some(recorder) = self.recorder.take() {
            recorder.discard()?;
        }
        Ok(())
    }

    pub fn open_library(&mut self, as_root: bool) {
        self.screen = Screen::Library;
        self.library_is_root = as_root;
        self.return_target = if as_root {
            ReturnTarget::Quit
        } else {
            ReturnTarget::Title
        };
        self.overlay = None;
        self.replay = None;
        self.library = Some(LibraryState::open(self.replays_dir.clone()));
        self.clear_screen = true;
    }

    pub fn attach_replay(&mut self, path: PathBuf, from_library: bool) -> anyhow::Result<()> {
        let (reader, game) = ReplayReader::open(&path)?;
        self.game = game;
        self.screen = Screen::Playing;
        self.overlay = None;
        self.recorder = None;
        if from_library {
            self.return_target = ReturnTarget::Library;
            self.replay = Some(ReplaySession::from_library(reader));
        } else {
            self.return_target = ReturnTarget::Quit;
            self.replay = Some(ReplaySession::new(reader));
        }
        self.clear_screen = true;
        self.sync_focus_to_stage();
        Ok(())
    }

    pub fn return_from_replay(&mut self) {
        let from_library = self.replay.as_ref().is_some_and(|s| s.from_library);
        self.replay = None;
        if from_library {
            let as_root = self.library_is_root;
            self.open_library(as_root);
        } else {
            self.should_quit = true;
        }
    }

    pub fn activate_title_button(&mut self) {
        match self.title_focus {
            TitleButton::Play => {
                if let Err(err) = self.start_play_from_title() {
                    self.title_status = format!("start run: {err:#}");
                }
            }
            TitleButton::Library => self.open_library(false),
        }
    }

    /// Apply `action`. When recording, the loggable move is captured first and
    /// written only after the action succeeds. Deselects are logged as hand
    /// slots because they are not in the gym action space. An outcome line
    /// follows the action that ends the episode.
    pub fn apply_action(&mut self, action: Action) -> bool {
        let logged = if self.recorder.is_some() {
            loggable_move(&self.game, &action)
        } else {
            None
        };
        if self.game.handle_action(action).is_err() {
            return false;
        }
        if let Some(step) = logged {
            if let Some(recorder) = self.recorder.as_mut() {
                if let Err(err) = recorder.record_move(step) {
                    eprintln!("replay log: {err:#}");
                } else if self.game.is_over() {
                    if let Err(err) = recorder.record_outcome(&self.game) {
                        eprintln!("replay log: {err:#}");
                    }
                }
            }
        } else if self.recorder.is_some() {
            eprintln!("replay log: action succeeded but has no log encoding");
        }
        true
    }

    pub fn replay_autoplay_due(&self) -> bool {
        self.replay
            .as_ref()
            .is_some_and(|session| session.autoplay_due())
    }

    pub fn replay_forward(&mut self) {
        let Some(mut session) = self.replay.take() else {
            return;
        };
        if session.reader.is_finished() {
            if session.finished_at.is_none() {
                session.finished_at = Some(Instant::now());
            }
            self.replay = Some(session);
            return;
        }
        match session.reader.forward(&mut self.game) {
            Ok(ReplayStep::Waiting) => session.waiting = true,
            Ok(ReplayStep::Applied) => {
                session.waiting = false;
                session.last_step = Instant::now();
                self.sync_focus_to_stage();
            }
            Ok(ReplayStep::Finished) => {
                session.waiting = false;
                session.last_step = Instant::now();
                if session.finished_at.is_none() {
                    session.finished_at = Some(Instant::now());
                }
                self.sync_focus_to_stage();
            }
            Err(err) => {
                eprintln!("replay: {err:#}");
                session.paused = true;
            }
        }
        self.replay = Some(session);
    }

    pub fn tick_replay_library_return(&mut self) {
        if self.replay.as_ref().is_some_and(|s| s.library_return_due()) {
            self.return_from_replay();
        }
    }

    pub fn replay_back(&mut self) {
        let Some(mut session) = self.replay.take() else {
            return;
        };
        if session.reader.at_start() {
            self.replay = Some(session);
            return;
        }
        match session.reader.back() {
            Ok(game) => {
                self.game = game;
                session.waiting = false;
                session.last_step = Instant::now();
                self.sync_focus_to_stage();
            }
            Err(err) => {
                eprintln!("replay: {err:#}");
                session.paused = true;
            }
        }
        self.replay = Some(session);
    }

    pub fn toggle_replay_pause(&mut self) {
        if let Some(session) = self.replay.as_mut() {
            session.paused = !session.paused;
            session.last_step = Instant::now();
        }
    }

    pub fn replay_status_text(&self) -> Option<String> {
        let session = self.replay.as_ref()?;
        let step = session.reader.applied_len();
        let source = match session.reader.source() {
            ReplaySource::Player => "player",
            ReplaySource::Model => "model",
        };
        let state = if session.reader.is_finished() {
            match session.reader.result() {
                Some(ReplayResult::Won) => "win",
                Some(ReplayResult::Lost) => "loss",
                Some(ReplayResult::Abandoned) => "abandoned",
                Some(ReplayResult::Error) => "error",
                None => "finished",
            }
        } else if session.paused {
            "paused"
        } else if session.waiting {
            "waiting"
        } else {
            "playing"
        };
        let back = if session.from_library {
            "q/esc library"
        } else {
            "q quit"
        };
        Some(format!(
            "replay  {source}  step {step}  {state}    space pause    h/left back    l/right forward    {back}"
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::scan_replays;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn title_play_discards_early_exit_and_remembers_toggle() {
        let dir = std::env::temp_dir().join(format!(
            "balatro-title-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut app = AppState::new_title(dir.clone());
        assert_eq!(app.screen, Screen::Title);

        app.title_focus = TitleButton::Play;
        app.activate_title_button();
        assert_eq!(app.screen, Screen::Playing);
        assert_eq!(app.return_target, ReturnTarget::Title);
        assert!(app.recorder.is_some());
        let recorded_path = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().is_some_and(|extension| extension == "log"))
            .expect("Play created a replay log");
        app.open_title();
        assert_eq!(app.screen, Screen::Title);
        assert!(!recorded_path.exists());
        let entries = scan_replays(&dir).unwrap();
        assert!(entries.is_empty());

        app.toggle_recording();
        assert!(!app.record_runs);
        let mut app = AppState::new_title(dir.clone());
        assert!(!app.record_runs);
        app.title_focus = TitleButton::Play;
        app.activate_title_button();
        assert!(app.recorder.is_none());
        app.open_title();
        assert!(scan_replays(&dir).unwrap().is_empty());

        app.title_focus = TitleButton::Library;
        app.activate_title_button();
        assert_eq!(app.screen, Screen::Library);
        assert!(!app.library_is_root);
        assert!(app.library.is_some());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
