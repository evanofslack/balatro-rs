use crate::app::{AppState, DeckTab, FocusZone, Overlay, RunInfoTab};
use balatro_rs::action::Action;
use balatro_rs::stage::Stage;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

pub fn handle_key(app: &mut AppState, key: KeyEvent) {
    if key.code == KeyCode::Char('c')
        && key
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
    {
        app.should_quit = true;
        return;
    }

    if let Some(overlay) = app.overlay.clone() {
        handle_key_overlay(app, key, overlay);
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('s') => app.open_save(),
        KeyCode::Char('r') => app.overlay = Some(Overlay::RunInfo),
        KeyCode::Char('?') => app.overlay = Some(Overlay::Controls),
        KeyCode::Char('i') => open_inspect(app),
        KeyCode::Tab | KeyCode::Char('j') | KeyCode::Down => app.tab_next(),
        KeyCode::BackTab | KeyCode::Char('k') | KeyCode::Up => app.tab_prev(),
        _ => handle_key_stage(app, key),
    }
}

fn handle_key_overlay(app: &mut AppState, key: KeyEvent, overlay: Overlay) {
    match overlay {
        Overlay::Save => handle_key_save(app, key),
        Overlay::RunInfo => handle_key_run_info(app, key),
        Overlay::Controls => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter) {
                app.close_overlay();
            }
        }
        Overlay::Inspect(_) => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('i') | KeyCode::Enter) {
                app.close_overlay();
            }
        }
        Overlay::Consumable(idx) => match key.code {
            KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => {
                if app.overlay_cursor > 0 {
                    app.overlay_cursor -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                if app.overlay_cursor < 1 {
                    app.overlay_cursor += 1;
                }
            }
            KeyCode::Char('u') => {
                if let Some(c) = app.game.consumables.get(idx).cloned() {
                    let prev = app.game.stage;
                    if app.game.handle_action(Action::UseConsumable(c)).is_ok() {
                        app.close_overlay();
                        if app.game.stage != prev {
                            app.sync_focus_to_stage();
                        }
                    }
                }
            }
            KeyCode::Enter => match app.overlay_cursor {
                0 => {
                    if let Some(c) = app.game.consumables.get(idx).cloned() {
                        let prev = app.game.stage;
                        if app.game.handle_action(Action::UseConsumable(c)).is_ok() {
                            app.close_overlay();
                            if app.game.stage != prev {
                                app.sync_focus_to_stage();
                            }
                        }
                    }
                }
                1 => {
                    if app.game.handle_action(Action::SellConsumable(idx)).is_ok() {
                        app.close_overlay();
                    }
                }
                _ => {}
            },
            KeyCode::Esc => app.close_overlay(),
            _ => {}
        },
        Overlay::Joker(idx) => match key.code {
            KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => {
                if app.overlay_cursor > 0 {
                    app.overlay_cursor -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                if app.overlay_cursor < 1 {
                    app.overlay_cursor += 1;
                }
            }
            KeyCode::Enter => match app.overlay_cursor {
                0 => {
                    if app.game.handle_action(Action::SellJoker(idx)).is_ok() {
                        app.close_overlay();
                        if app.cursor >= app.game.jokers.len() && app.cursor > 0 {
                            app.cursor -= 1;
                        }
                    }
                }
                _ => app.close_overlay(),
            },
            KeyCode::Esc => app.close_overlay(),
            _ => {}
        },
    }
}

fn handle_key_save(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_overlay(),
        KeyCode::Enter => do_save(app),
        KeyCode::Backspace => {
            app.save_input.pop();
        }
        KeyCode::Char(c) => {
            app.save_input.push(c);
        }
        _ => {}
    }
}

fn handle_key_run_info(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('r') => app.close_overlay(),
        KeyCode::Tab | KeyCode::Char('j') | KeyCode::Down => {
            app.run_info_tab = match app.run_info_tab {
                RunInfoTab::Deck => RunInfoTab::PokerHands,
                RunInfoTab::PokerHands => RunInfoTab::Vouchers,
                RunInfoTab::Vouchers => RunInfoTab::Deck,
            };
        }
        KeyCode::BackTab | KeyCode::Char('k') | KeyCode::Up => {
            app.run_info_tab = match app.run_info_tab {
                RunInfoTab::Deck => RunInfoTab::Vouchers,
                RunInfoTab::PokerHands => RunInfoTab::Deck,
                RunInfoTab::Vouchers => RunInfoTab::PokerHands,
            };
        }
        KeyCode::Left | KeyCode::Char('h') if matches!(app.run_info_tab, RunInfoTab::Deck) => {
            app.deck_tab = match app.deck_tab {
                DeckTab::InDeck => DeckTab::Discarded,
                DeckTab::InHand => DeckTab::InDeck,
                DeckTab::Discarded => DeckTab::InHand,
            };
        }
        KeyCode::Right | KeyCode::Char('l') if matches!(app.run_info_tab, RunInfoTab::Deck) => {
            app.deck_tab = match app.deck_tab {
                DeckTab::InDeck => DeckTab::InHand,
                DeckTab::InHand => DeckTab::Discarded,
                DeckTab::Discarded => DeckTab::InDeck,
            };
        }
        _ => {}
    }
}

fn handle_key_stage(app: &mut AppState, key: KeyEvent) {
    let prev_stage = app.game.stage;

    // Normalize vim motion keys to arrow keys for all zone handlers
    let key = match key.code {
        KeyCode::Char('h') => KeyEvent {
            code: KeyCode::Left,
            ..key
        },
        KeyCode::Char('l') => KeyEvent {
            code: KeyCode::Right,
            ..key
        },
        _ => key,
    };

    match &app.game.stage {
        Stage::PreBlind() => handle_key_preblind(app, key),
        Stage::Blind(_) => handle_key_blind(app, key),
        Stage::PostBlind() => handle_key_postblind(app, key),
        Stage::Shop() => handle_key_shop(app, key),
        Stage::TarotHand(_) => handle_key_tarot(app, key),
        Stage::PackOpen() => handle_key_pack(app, key),
        Stage::End(_) => {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char('q')) {
                app.should_quit = true;
            }
        }
    }

    if app.game.stage != prev_stage {
        app.sync_focus_to_stage();
    }
}

fn handle_key_preblind(app: &mut AppState, key: KeyEvent) {
    use balatro_rs::stage::Blind;

    let blinds = [Blind::Small, Blind::Big, Blind::Boss];

    match key.code {
        KeyCode::Left => {
            let mut c = app.cursor;
            while c > 0 {
                c -= 1;
                let b = blinds[c];
                if app
                    .game
                    .gen_actions()
                    .any(|a| matches!(a, Action::SelectBlind(x) if x == b))
                {
                    app.cursor = c;
                    break;
                }
            }
        }
        KeyCode::Right => {
            let mut c = app.cursor;
            while c + 1 < blinds.len() {
                c += 1;
                let b = blinds[c];
                if app
                    .game
                    .gen_actions()
                    .any(|a| matches!(a, Action::SelectBlind(x) if x == b))
                {
                    app.cursor = c;
                    break;
                }
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            let blind = blinds[app.cursor.min(2)];
            let _ = app.game.handle_action(Action::SelectBlind(blind));
        }
        _ => {}
    }
}

fn handle_key_blind(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('p') => {
            let _ = app.game.handle_action(Action::Play());
            return;
        }
        KeyCode::Char('d') => {
            let _ = app.game.handle_action(Action::Discard());
            return;
        }
        _ => {}
    }

    match &app.focus {
        FocusZone::Cards => handle_key_blind_cards(app, key),
        FocusZone::ActionButtons => handle_key_blind_buttons(app, key),
        FocusZone::JokerStrip => handle_key_joker_strip(app, key),
        FocusZone::ConsumableStrip => handle_key_consumable_strip(app, key),
        _ => {}
    }
}

fn handle_key_blind_cards(app: &mut AppState, key: KeyEvent) {
    let card_count = app.game.available.cards().len();
    match key.code {
        KeyCode::Left => {
            if app.cursor > 0 {
                app.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if app.cursor + 1 < card_count {
                app.cursor += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => toggle_card(app),
        _ => {}
    }
}

fn handle_key_blind_buttons(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Left => {
            if app.cursor > 0 {
                app.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if app.cursor < 2 {
                app.cursor += 1;
            }
        }
        KeyCode::Enter => match app.cursor {
            0 => {
                let _ = app.game.handle_action(Action::Play());
            }
            1 => {}
            2 => {
                let _ = app.game.handle_action(Action::Discard());
            }
            _ => {}
        },
        _ => {}
    }
}

fn handle_key_joker_strip(app: &mut AppState, key: KeyEvent) {
    let joker_count = app.game.jokers.len();
    match key.code {
        KeyCode::Left => {
            if app.cursor > 0 {
                app.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if joker_count > 0 && app.cursor + 1 < joker_count {
                app.cursor += 1;
            }
        }
        KeyCode::Enter => {
            if app.cursor < joker_count {
                app.overlay = Some(Overlay::Joker(app.cursor));
                app.overlay_cursor = 1;
            }
        }
        _ => {}
    }
}

fn handle_key_consumable_strip(app: &mut AppState, key: KeyEvent) {
    let count = app.game.consumables.len();
    match key.code {
        KeyCode::Left => {
            if app.cursor > 0 {
                app.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if count > 0 && app.cursor + 1 < count {
                app.cursor += 1;
            }
        }
        KeyCode::Enter => {
            if app.cursor < count {
                app.overlay = Some(Overlay::Consumable(app.cursor));
                app.overlay_cursor = 0;
            }
        }
        _ => {}
    }
}

fn handle_key_postblind(app: &mut AppState, key: KeyEvent) {
    match &app.focus {
        FocusZone::CashOutButton => {
            if key.code == KeyCode::Enter {
                let reward = compute_cashout(app);
                let _ = app.game.handle_action(Action::CashOut(reward));
            }
        }
        FocusZone::JokerStrip => handle_key_joker_strip(app, key),
        FocusZone::ConsumableStrip => handle_key_consumable_strip(app, key),
        _ => {}
    }
}

pub fn compute_cashout(app: &AppState) -> usize {
    app.game.reward
}

fn handle_key_shop(app: &mut AppState, key: KeyEvent) {
    match &app.focus {
        FocusZone::ShopJokers => handle_key_shop_jokers(app, key),
        FocusZone::ShopPacks => handle_key_shop_packs(app, key),
        FocusZone::ShopNextRound => {
            if key.code == KeyCode::Enter {
                let _ = app.game.handle_action(Action::NextRound());
            }
        }
        FocusZone::JokerStrip => handle_key_joker_strip(app, key),
        FocusZone::ConsumableStrip => handle_key_consumable_strip(app, key),
        _ => {}
    }

    if key.code == KeyCode::Char('n') {
        let _ = app.game.handle_action(Action::NextRound());
    }
}

fn handle_key_shop_packs(app: &mut AppState, key: KeyEvent) {
    let count = app.game.shop.packs.len();
    match key.code {
        KeyCode::Left => {
            if app.cursor > 0 {
                app.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if count > 0 && app.cursor + 1 < count {
                app.cursor += 1;
            }
        }
        KeyCode::Enter => {
            if app.cursor < count {
                let pack = app.game.shop.packs[app.cursor].clone();
                let _ = app.game.handle_action(Action::BuyPack(pack));
            }
        }
        _ => {}
    }
}

fn handle_key_pack(app: &mut AppState, key: KeyEvent) {
    match &app.focus {
        FocusZone::PackContents => handle_key_pack_contents(app, key),
        FocusZone::PackSkip => {
            if key.code == KeyCode::Enter {
                let _ = app.game.handle_action(Action::SkipPack());
            }
        }
        _ => {
            app.focus = FocusZone::PackContents;
            app.cursor = 0;
        }
    }
}

fn handle_key_pack_contents(app: &mut AppState, key: KeyEvent) {
    let count = app
        .game
        .open_pack
        .as_ref()
        .map(|s| s.contents.len())
        .unwrap_or(0);
    match key.code {
        KeyCode::Left => {
            if app.cursor > 0 {
                app.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if count > 0 && app.cursor + 1 < count {
                app.cursor += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(content) = app
                .game
                .open_pack
                .as_ref()
                .and_then(|s| s.contents.get(app.cursor))
                .cloned()
            {
                let _ = app.game.handle_action(Action::PickPackCard(content));
            }
        }
        _ => {}
    }
}

fn handle_key_shop_jokers(app: &mut AppState, key: KeyEvent) {
    let count = app.game.shop.jokers.len();
    match key.code {
        KeyCode::Left => {
            if app.cursor > 0 {
                app.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if count > 0 && app.cursor + 1 < count {
                app.cursor += 1;
            }
        }
        KeyCode::Enter => {
            if app.cursor < count {
                let joker = app.game.shop.jokers[app.cursor].clone();
                let _ = app.game.handle_action(Action::BuyJoker(joker));
            }
        }
        _ => {}
    }
}

fn handle_key_tarot(app: &mut AppState, key: KeyEvent) {
    match &app.focus {
        FocusZone::TarotCards => handle_key_tarot_cards(app, key),
        FocusZone::TarotButtons => handle_key_tarot_buttons(app, key),
        FocusZone::JokerStrip => handle_key_joker_strip(app, key),
        FocusZone::ConsumableStrip => handle_key_consumable_strip(app, key),
        _ => {
            app.focus = FocusZone::TarotCards;
            app.cursor = 0;
        }
    }
}

fn handle_key_tarot_cards(app: &mut AppState, key: KeyEvent) {
    let card_count = app.game.available.cards().len();
    match key.code {
        KeyCode::Left => {
            if app.cursor > 0 {
                app.cursor -= 1;
            }
        }
        KeyCode::Right => {
            if app.cursor + 1 < card_count {
                app.cursor += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => toggle_card(app),
        _ => {}
    }
}

fn handle_key_tarot_buttons(app: &mut AppState, key: KeyEvent) {
    if key.code == KeyCode::Enter {
        let _ = app.game.handle_action(Action::ApplyTarot());
    }
}

fn toggle_card(app: &mut AppState) {
    let cards = app.game.available.cards();
    let selected_ids: std::collections::HashSet<usize> =
        app.game.available.selected().iter().map(|c| c.id).collect();
    if let Some(&card) = cards.get(app.cursor) {
        if selected_ids.contains(&card.id) {
            let _ = app.game.handle_action(Action::DeselectCard(card));
        } else {
            let _ = app.game.handle_action(Action::SelectCard(card));
        }
    }
}

fn open_inspect(app: &mut AppState) {
    use crate::app::InspectTarget;
    match &app.focus {
        FocusZone::Cards | FocusZone::TarotCards => {
            let cards = app.game.available.cards();
            if let Some(card) = cards.get(app.cursor) {
                app.overlay = Some(Overlay::Inspect(InspectTarget::Card(*card)));
            }
        }
        FocusZone::JokerStrip => {
            if let Some(joker) = app.game.jokers.get(app.cursor) {
                app.overlay = Some(Overlay::Inspect(InspectTarget::Joker(joker.clone())));
            }
        }
        FocusZone::ConsumableStrip => {
            if let Some(c) = app.game.consumables.get(app.cursor) {
                app.overlay = Some(Overlay::Inspect(InspectTarget::Consumable(c.clone())));
            }
        }
        FocusZone::ShopJokers => {
            if let Some(joker) = app.game.shop.jokers.get(app.cursor) {
                app.overlay = Some(Overlay::Inspect(InspectTarget::Joker(joker.clone())));
            }
        }
        FocusZone::ShopPacks => {
            if let Some(pack) = app.game.shop.packs.get(app.cursor) {
                app.overlay = Some(Overlay::Inspect(InspectTarget::Pack(pack.clone())));
            }
        }
        FocusZone::PackContents => {
            if let Some(target) = crate::ui::pack::inspect_target_for_cursor(app) {
                app.overlay = Some(Overlay::Inspect(target));
            }
        }
        _ => {}
    }
}

fn do_save(app: &mut AppState) {
    match app.game.to_json() {
        Ok(json) => match std::fs::write(&app.save_input, json) {
            Ok(()) => app.close_overlay(),
            Err(e) => app.save_input = format!("ERROR: {}", e),
        },
        Err(e) => app.save_input = format!("ERROR: {}", e),
    }
}

pub fn handle_mouse(app: &mut AppState, event: MouseEvent) {
    if event.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left) {
        return;
    }
    let col = event.column;
    let row = event.row;

    let hit = app
        .widget_rects
        .iter()
        .find(|(_, rect)| {
            col >= rect.x
                && col < rect.x + rect.width
                && row >= rect.y
                && row < rect.y + rect.height
        })
        .map(|(id, _)| id.clone());

    let Some(widget_id) = hit else { return };
    dispatch_mouse_click(app, widget_id);
}

fn dispatch_mouse_click(app: &mut AppState, id: crate::app::WidgetId) {
    use crate::app::WidgetId::*;
    let prev_stage = app.game.stage;

    match id {
        Card(idx) => {
            app.focus = if matches!(app.game.stage, Stage::TarotHand(_)) {
                FocusZone::TarotCards
            } else {
                FocusZone::Cards
            };
            app.cursor = idx;
            toggle_card(app);
        }
        ActionButton(0) => {
            let _ = app.game.handle_action(Action::Play());
        }
        ActionButton(2) => {
            let _ = app.game.handle_action(Action::Discard());
        }
        ActionButton(_) => {}
        JokerSlot(idx) => {
            app.focus = FocusZone::JokerStrip;
            app.cursor = idx;
            if idx < app.game.jokers.len() {
                app.overlay = Some(Overlay::Joker(idx));
                app.overlay_cursor = 1;
            }
        }
        ConsumableSlot(idx) => {
            app.focus = FocusZone::ConsumableStrip;
            app.cursor = idx;
            app.overlay = Some(Overlay::Consumable(idx));
            app.overlay_cursor = 0;
        }
        ShopJoker(idx) => {
            app.focus = FocusZone::ShopJokers;
            app.cursor = idx;
            if let Some(joker) = app.game.shop.jokers.get(idx) {
                let _ = app.game.handle_action(Action::BuyJoker(joker.clone()));
            }
        }
        ShopPack(idx) => {
            app.focus = FocusZone::ShopPacks;
            app.cursor = idx;
            if let Some(pack) = app.game.shop.packs.get(idx) {
                let _ = app.game.handle_action(Action::BuyPack(pack.clone()));
            }
        }
        PackContent(idx) => {
            app.focus = FocusZone::PackContents;
            app.cursor = idx;
            if let Some(content) = app
                .game
                .open_pack
                .as_ref()
                .and_then(|s| s.contents.get(idx))
                .cloned()
            {
                let _ = app.game.handle_action(Action::PickPackCard(content));
            }
        }
        SkipPackButton => {
            let _ = app.game.handle_action(Action::SkipPack());
        }
        BlindOption(idx) => {
            use balatro_rs::stage::Blind;
            app.cursor = idx;
            let blind = match idx {
                0 => Blind::Small,
                1 => Blind::Big,
                _ => Blind::Boss,
            };
            let _ = app.game.handle_action(Action::SelectBlind(blind));
        }
        CashOutButton => {
            if matches!(app.game.stage, Stage::End(_)) {
                app.should_quit = true;
            } else {
                let reward = compute_cashout(app);
                let _ = app.game.handle_action(Action::CashOut(reward));
            }
        }
        NextRoundButton => {
            let _ = app.game.handle_action(Action::NextRound());
        }
        TarotButton(0) => {
            let _ = app.game.handle_action(Action::ApplyTarot());
        }
        TarotButton(_) => {}
        OverlayButton(0) => match app.overlay.clone() {
            Some(crate::app::Overlay::Consumable(idx)) => {
                if let Some(c) = app.game.consumables.get(idx).cloned() {
                    let prev = app.game.stage;
                    if app.game.handle_action(Action::UseConsumable(c)).is_ok() {
                        app.close_overlay();
                        if app.game.stage != prev {
                            app.sync_focus_to_stage();
                        }
                    }
                }
            }
            Some(crate::app::Overlay::Joker(idx)) => {
                if app.game.handle_action(Action::SellJoker(idx)).is_ok() {
                    app.close_overlay();
                    if app.cursor >= app.game.jokers.len() && app.cursor > 0 {
                        app.cursor -= 1;
                    }
                }
            }
            Some(crate::app::Overlay::Save) => do_save(app),
            _ => app.close_overlay(),
        },
        OverlayButton(1) => match app.overlay.clone() {
            Some(crate::app::Overlay::Consumable(idx)) => {
                if app.game.handle_action(Action::SellConsumable(idx)).is_ok() {
                    app.close_overlay();
                }
            }
            _ => app.close_overlay(),
        },
        OverlayButton(_) => {}
        DeckTab(idx) => {
            use crate::app::DeckTab as DT;
            app.deck_tab = match idx {
                0 => DT::InDeck,
                1 => DT::InHand,
                _ => DT::Discarded,
            };
        }
    }

    if app.game.stage != prev_stage {
        app.sync_focus_to_stage();
    }
}
