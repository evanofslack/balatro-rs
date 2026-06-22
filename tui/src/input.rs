use crate::app::{AppState, DeckTab, FocusZone, Overlay, RunInfoTab};
use balatro_rs::action::Action;
use balatro_rs::stage::Stage;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

pub fn handle_key(app: &mut AppState, key: KeyEvent) {
    if let Some(ref overlay) = app.overlay.clone() {
        handle_key_overlay(app, key, overlay.clone());
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
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
            KeyCode::Enter | KeyCode::Char('u') => {
                if let Some(c) = app.game.consumables.get(idx).cloned() {
                    let prev = app.game.stage.clone();
                    app.close_overlay();
                    let _ = app.game.handle_action(Action::UseConsumable(c));
                    if app.game.stage != prev {
                        app.sync_focus_to_stage();
                    }
                }
            }
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
    let prev_stage = app.game.stage.clone();

    // Normalize vim motion keys to arrow keys for all zone handlers
    let key = match key.code {
        KeyCode::Char('h') => KeyEvent { code: KeyCode::Left, ..key },
        KeyCode::Char('l') => KeyEvent { code: KeyCode::Right, ..key },
        _ => key,
    };

    match &app.game.stage.clone() {
        Stage::PreBlind() => handle_key_preblind(app, key),
        Stage::Blind(_) => handle_key_blind(app, key),
        Stage::PostBlind() => handle_key_postblind(app, key),
        Stage::Shop() => handle_key_shop(app, key),
        Stage::TarotHand(_) => handle_key_tarot(app, key),
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
                if app.game.gen_actions().any(|a| matches!(a, Action::SelectBlind(x) if x == b)) {
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
                if app.game.gen_actions().any(|a| matches!(a, Action::SelectBlind(x) if x == b)) {
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
            }
        }
        _ => {}
    }
}

fn handle_key_postblind(app: &mut AppState, key: KeyEvent) {
    if key.code == KeyCode::Enter {
        let reward = compute_cashout(app);
        let _ = app.game.handle_action(Action::CashOut(reward));
    }
}

pub fn compute_cashout(app: &AppState) -> usize {
    app.game.reward
}

fn handle_key_shop(app: &mut AppState, key: KeyEvent) {
    match &app.focus.clone() {
        FocusZone::ShopJokers => handle_key_shop_jokers(app, key),
        FocusZone::ShopConsumables => handle_key_shop_consumables(app, key),
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

fn handle_key_shop_consumables(app: &mut AppState, key: KeyEvent) {
    let count = app.game.shop.consumables.len();
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
                let c = app.game.shop.consumables[app.cursor].clone();
                let _ = app.game.handle_action(Action::BuyConsumable(c));
            }
        }
        _ => {}
    }
}

fn handle_key_tarot(app: &mut AppState, key: KeyEvent) {
    match &app.focus.clone() {
        FocusZone::TarotCards => handle_key_tarot_cards(app, key),
        FocusZone::TarotButtons => handle_key_tarot_buttons(app, key),
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
    match key.code {
        KeyCode::Enter => {
            let _ = app.game.handle_action(Action::ApplyTarot());
        }
        _ => {}
    }
}

fn toggle_card(app: &mut AppState) {
    let cards = app.game.available.cards();
    let selected_ids: std::collections::HashSet<usize> = app
        .game
        .available
        .selected()
        .iter()
        .map(|c| c.id)
        .collect();
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
        FocusZone::ShopConsumables => {
            if let Some(c) = app.game.shop.consumables.get(app.cursor) {
                app.overlay = Some(Overlay::Inspect(InspectTarget::Consumable(c.clone())));
            }
        }
        _ => {}
    }
}

fn do_save(app: &mut AppState) {
    if let Ok(json) = app.game.to_json() {
        let _ = std::fs::write(&app.save_input, json);
    }
    app.close_overlay();
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
    let prev_stage = app.game.stage.clone();

    match id {
        Card(idx) => {
            app.focus = FocusZone::Cards;
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
        }
        ConsumableSlot(idx) => {
            app.focus = FocusZone::ConsumableStrip;
            app.cursor = idx;
            app.overlay = Some(Overlay::Consumable(idx));
        }
        ShopJoker(idx) => {
            app.focus = FocusZone::ShopJokers;
            app.cursor = idx;
            if let Some(joker) = app.game.shop.jokers.get(idx) {
                let _ = app.game.handle_action(Action::BuyJoker(joker.clone()));
            }
        }
        ShopConsumable(idx) => {
            app.focus = FocusZone::ShopConsumables;
            app.cursor = idx;
            if let Some(c) = app.game.shop.consumables.get(idx) {
                let _ = app.game.handle_action(Action::BuyConsumable(c.clone()));
            }
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
            let reward = compute_cashout(app);
            let _ = app.game.handle_action(Action::CashOut(reward));
        }
        NextRoundButton => {
            let _ = app.game.handle_action(Action::NextRound());
        }
        TarotButton(0) => {
            let _ = app.game.handle_action(Action::ApplyTarot());
        }
        TarotButton(_) => {}
        OverlayButton(0) => {
            if let Some(crate::app::Overlay::Consumable(idx)) = app.overlay.clone() {
                if let Some(c) = app.game.consumables.get(idx).cloned() {
                    let prev = app.game.stage.clone();
                    app.close_overlay();
                    let _ = app.game.handle_action(Action::UseConsumable(c));
                    if app.game.stage != prev {
                        app.sync_focus_to_stage();
                    }
                }
            } else {
                app.close_overlay();
            }
        }
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
