use crate::action::{Action, MoveDirection};
use crate::card::Edition;
use crate::consumable::Consumable;
use crate::game::Game;
use crate::joker::Joker;
use crate::pack::PackContent;
use crate::space::ActionSpace;
use crate::stage::{Blind, Stage};

impl Game {
    // Get all legal SelectCard actions that can be executed given current state
    fn gen_actions_select_card(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only select card during blinds
        if !self.stage.is_blind() {
            return None;
        }
        // Cannot select more than max
        if self.available.selected().len() >= self.config.selected_max {
            return None;
        }
        let combos = self
            .available
            .not_selected()
            .clone()
            .into_iter()
            .map(Action::SelectCard);
        Some(combos)
    }

    // Get all legal DeselectCard actions that can be executed given current state
    fn gen_actions_deselect_card(&self) -> Option<impl Iterator<Item = Action>> {
        if !self.stage.is_blind() {
            return None;
        }
        let selected = self.available.selected();
        if selected.is_empty() {
            return None;
        }
        Some(selected.into_iter().map(Action::DeselectCard))
    }

    // Get all legal Play actions that can be executed given current state
    fn gen_actions_play(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only play hand during blinds
        if !self.stage.is_blind() {
            return None;
        }
        // If no plays remaining, return None
        if self.plays == 0 {
            return None;
        }
        // If no cards selected, return None
        if self.available.selected().is_empty() {
            return None;
        }
        let combos = vec![Action::Play()].into_iter();
        Some(combos)
    }

    // Get all legal Play actions that can be executed given current state
    fn gen_actions_discard(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only discard during blinds
        if !self.stage.is_blind() {
            return None;
        }
        // If no discards remaining, return None
        if self.discards == 0 {
            return None;
        }
        // If no cards selected, return None
        if self.available.selected().is_empty() {
            return None;
        }
        let combos = vec![Action::Discard()].into_iter();
        Some(combos)
    }

    // Get all legal move card actions
    fn gen_actions_move_card(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only move cards during blinds
        if !self.stage.is_blind() {
            return None;
        }
        let left = self
            .available
            .cards()
            .clone()
            .into_iter()
            .skip(1)
            .map(|c| Action::MoveCard(MoveDirection::Left, c));
        let right = self
            .available
            .cards()
            .clone()
            .into_iter()
            .rev()
            .skip(1)
            .rev()
            .map(|c| Action::MoveCard(MoveDirection::Right, c));

        let combos = left.chain(right);
        Some(combos)
    }

    // Get cash out action
    fn gen_actions_cash_out(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not post blind, cannot cash out
        if self.stage != Stage::PostBlind() {
            return None;
        }
        Some(vec![Action::CashOut(self.reward)].into_iter())
    }

    // Get next round action
    fn gen_actions_next_round(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not shop, cannot next round
        if self.stage != Stage::Shop() {
            return None;
        }
        Some(vec![Action::NextRound()].into_iter())
    }

    // Get select blind action
    fn gen_actions_select_blind(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not pre blind, cannot select blind
        if self.stage != Stage::PreBlind() {
            return None;
        }
        if let Some(blind) = self.blind {
            Some(vec![Action::SelectBlind(blind.next())].into_iter())
        } else {
            Some(vec![Action::SelectBlind(Blind::Small)].into_iter())
        }
    }

    // Get buy joker actions
    fn gen_actions_buy_joker(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not shop, cannot buy
        if self.stage != Stage::Shop() {
            return None;
        }
        // Cannot buy if all joker slots full
        if self.jokers.len() >= self.config.joker_slots {
            return None;
        }
        self.shop.gen_moves_buy_joker(self.money)
    }

    // Get buy consumable actions
    fn gen_actions_buy_consumable(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::Shop() {
            return None;
        }
        self.shop.gen_moves_buy_consumable(
            self.money,
            self.config.consumable_slots,
            self.consumables.len(),
        )
    }

    // Get use consumable actions
    fn gen_actions_use_consumable(&self) -> Option<impl Iterator<Item = Action>> {
        if matches!(
            self.stage,
            Stage::End(_) | Stage::TarotHand(_) | Stage::PackOpen()
        ) {
            return None;
        }
        if self.consumables.is_empty() {
            return None;
        }
        let selected_count = self.available.selected().len();
        let actions: Vec<Action> = self
            .consumables
            .iter()
            .filter(|c| match c {
                Consumable::Planet(_) => true,
                // eligible to play tarot if any
                // 1.) tarot doesn't require selected cards
                // 2.) in blind stage and card(s) are selected
                // 3.) not in blind stage (will draw hand to apply tarot)
                Consumable::Tarot(t) => {
                    if !t.requires_targets() {
                        true
                    } else if self.stage.is_blind() {
                        selected_count >= t.min_targets() && selected_count <= t.max_targets()
                    } else {
                        true
                    }
                }
            })
            .cloned()
            .map(Action::UseConsumable)
            .collect();
        if actions.is_empty() {
            None
        } else {
            Some(actions.into_iter())
        }
    }

    fn gen_actions_sell_joker(&self) -> Option<impl Iterator<Item = Action>> {
        if matches!(self.stage, Stage::End(_)) {
            return None;
        }
        if self.jokers.is_empty() {
            return None;
        }
        let actions: Vec<Action> = (0..self.jokers.len()).map(Action::SellJoker).collect();
        Some(actions.into_iter())
    }

    fn gen_actions_sell_consumable(&self) -> Option<impl Iterator<Item = Action>> {
        if matches!(self.stage, Stage::End(_)) {
            return None;
        }
        if self.consumables.is_empty() {
            return None;
        }
        let actions: Vec<Action> = (0..self.consumables.len())
            .map(Action::SellConsumable)
            .collect();
        Some(actions.into_iter())
    }

    // Get buy pack actions
    fn gen_actions_buy_pack(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::Shop() {
            return None;
        }
        self.shop.gen_moves_buy_pack(self.money)
    }

    // Get pick pack card actions (active while a pack is open)
    fn gen_actions_pick_pack_card(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::PackOpen() {
            return None;
        }
        let state = self.open_pack.as_ref()?;
        let joker_slots = self.config.joker_slots;
        let joker_count = self.jokers.len();
        let picks: Vec<Action> = state
            .contents
            .iter()
            .filter(|c| match c {
                PackContent::Joker(j) => {
                    joker_count < joker_slots || j.edition() == Edition::Negative
                }
                _ => true,
            })
            .cloned()
            .map(Action::PickPackCard)
            .collect();
        if picks.is_empty() {
            None
        } else {
            Some(picks.into_iter())
        }
    }

    // Skip is always available while a pack is open
    fn gen_actions_skip_pack(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::PackOpen() {
            return None;
        }
        Some(vec![Action::SkipPack()].into_iter())
    }

    // are we in the temp tarot hand stage?
    // if so, we draw a temp hand and give options for applying the tarot.
    fn gen_actions_tarot_hand(&self) -> Option<impl Iterator<Item = Action>> {
        let Stage::TarotHand(t) = self.stage else {
            return None;
        };
        let selected_count = self.available.selected().len();

        let select_cards: Vec<Action> = if selected_count < t.max_targets() {
            self.available
                .not_selected()
                .into_iter()
                .map(Action::SelectCard)
                .collect()
        } else {
            vec![]
        };

        let deselect_cards: Vec<Action> = self
            .available
            .selected()
            .into_iter()
            .map(Action::DeselectCard)
            .collect();

        let move_left: Vec<Action> = self
            .available
            .cards()
            .into_iter()
            .skip(1)
            .map(|c| Action::MoveCard(MoveDirection::Left, c))
            .collect();

        let move_right: Vec<Action> = {
            let cards = self.available.cards();
            let len = cards.len();
            if len > 1 {
                cards
                    .into_iter()
                    .take(len - 1)
                    .map(|c| Action::MoveCard(MoveDirection::Right, c))
                    .collect()
            } else {
                vec![]
            }
        };

        let apply: Vec<Action> =
            if selected_count >= t.min_targets() && selected_count <= t.max_targets() {
                vec![Action::ApplyTarot()]
            } else {
                vec![]
            };

        let all: Vec<Action> = select_cards
            .into_iter()
            .chain(deselect_cards)
            .chain(move_left)
            .chain(move_right)
            .chain(apply)
            .collect();

        Some(all.into_iter())
    }

    // Get all legal actions that can be executed given current state
    pub fn gen_actions(&self) -> impl Iterator<Item = Action> {
        let select_cards = self.gen_actions_select_card();
        let deselect_cards = self.gen_actions_deselect_card();
        let plays = self.gen_actions_play();
        let discards = self.gen_actions_discard();
        let move_cards = self.gen_actions_move_card();
        let cash_outs = self.gen_actions_cash_out();
        let next_rounds = self.gen_actions_next_round();
        let select_blinds = self.gen_actions_select_blind();
        let buy_jokers = self.gen_actions_buy_joker();
        let buy_consumables = self.gen_actions_buy_consumable();
        let use_consumables = self.gen_actions_use_consumable();
        let tarot_hand = self.gen_actions_tarot_hand();
        let sell_jokers = self.gen_actions_sell_joker();
        let sell_consumables = self.gen_actions_sell_consumable();
        let buy_packs = self.gen_actions_buy_pack();
        let pick_pack_cards = self.gen_actions_pick_pack_card();
        let skip_packs = self.gen_actions_skip_pack();

        select_cards
            .into_iter()
            .flatten()
            .chain(deselect_cards.into_iter().flatten())
            .chain(plays.into_iter().flatten())
            .chain(discards.into_iter().flatten())
            .chain(move_cards.into_iter().flatten())
            .chain(cash_outs.into_iter().flatten())
            .chain(next_rounds.into_iter().flatten())
            .chain(select_blinds.into_iter().flatten())
            .chain(buy_jokers.into_iter().flatten())
            .chain(buy_consumables.into_iter().flatten())
            .chain(use_consumables.into_iter().flatten())
            .chain(tarot_hand.into_iter().flatten())
            .chain(sell_jokers.into_iter().flatten())
            .chain(sell_consumables.into_iter().flatten())
            .chain(buy_packs.into_iter().flatten())
            .chain(pick_pack_cards.into_iter().flatten())
            .chain(skip_packs.into_iter().flatten())
    }

    fn unmask_action_space_select_cards(&self, space: &mut ActionSpace) {
        if !self.stage.is_blind() {
            return;
        }
        // Cannot select more if max already selected
        if self.available.selected().len() >= self.config.selected_max {
            return;
        }
        self.available
            .cards_and_selected()
            .iter()
            .enumerate()
            .filter(|(_, (_, a))| !*a)
            .for_each(|(i, _)| {
                space
                    .unmask_select_card(i)
                    .expect("valid index for selecting");
            });
    }

    fn unmask_action_space_play_and_discard(&self, space: &mut ActionSpace) {
        if !self.stage.is_blind() {
            return;
        }
        // Cannot play/discard if no cards selected
        if self.available.selected().is_empty() {
            return;
        }
        // Can only play/discard is have remaining
        if self.plays != 0 {
            space.unmask_play();
        }
        if self.discards != 0 {
            space.unmask_discard();
        }
    }

    fn unmask_action_space_move_cards(&self, space: &mut ActionSpace) {
        if !self.stage.is_blind() {
            return;
        }
        // move left
        // every available card except the first can move left
        self.available
            .cards()
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(i, _)| {
                space
                    .unmask_move_card_left(i)
                    .expect("valid index for move left")
            });
        // move right
        // every available card except the last can move right
        self.available
            .cards()
            .iter()
            .rev()
            .skip(1)
            .rev()
            .enumerate()
            .for_each(|(i, _)| {
                space
                    .unmask_move_card_right(i)
                    .expect("valid index for move right")
            });
    }

    fn unmask_action_space_cash_out(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PostBlind() {
            return;
        }
        space.unmask_cash_out();
    }

    fn unmask_action_space_next_round(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        space.unmask_next_round();
    }

    fn unmask_action_space_select_blind(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PreBlind() {
            return;
        }
        space.unmask_select_blind();
    }

    fn unmask_action_space_buy_joker(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        self.shop
            .jokers
            .iter()
            .enumerate()
            .filter(|(_i, j)| j.cost() <= self.money)
            .for_each(|(i, _j)| {
                space
                    .unmask_buy_joker(i)
                    .expect("valid index for buy joker")
            });
    }

    fn unmask_action_space_buy_consumable(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        if self.consumables.len() >= self.config.consumable_slots {
            return;
        }
        self.shop
            .consumables
            .iter()
            .enumerate()
            .filter(|(_i, c)| c.cost() <= self.money)
            .for_each(|(i, _c)| {
                space
                    .unmask_buy_consumable(i)
                    .expect("valid index for buy consumable")
            });
    }

    fn unmask_action_space_use_consumable(&self, space: &mut ActionSpace) {
        if matches!(self.stage, Stage::End(_) | Stage::TarotHand(_)) {
            return;
        }
        let selected_count = self.available.selected().len();
        self.consumables.iter().enumerate().for_each(|(i, c)| {
            let valid = match c {
                Consumable::Planet(_) => true,
                Consumable::Tarot(t) => {
                    if !t.requires_targets() {
                        true
                    } else if self.stage.is_blind() {
                        selected_count >= t.min_targets() && selected_count <= t.max_targets()
                    } else {
                        true
                    }
                }
            };
            if valid {
                space
                    .unmask_use_consumable(i)
                    .expect("valid index for use consumable");
            }
        });
    }

    fn unmask_action_space_sell_joker(&self, space: &mut ActionSpace) {
        if matches!(self.stage, Stage::End(_)) {
            return;
        }
        self.jokers.iter().enumerate().for_each(|(i, _)| {
            space
                .unmask_sell_joker(i)
                .expect("valid index for sell joker");
        });
    }

    fn unmask_action_space_sell_consumable(&self, space: &mut ActionSpace) {
        if matches!(self.stage, Stage::End(_)) {
            return;
        }
        self.consumables.iter().enumerate().for_each(|(i, _)| {
            space
                .unmask_sell_consumable(i)
                .expect("valid index for sell consumable");
        });
    }

    fn unmask_action_space_buy_pack(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        self.shop
            .packs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.cost() <= self.money)
            .for_each(|(i, _)| {
                space.unmask_buy_pack(i).expect("valid index for buy pack");
            });
    }

    fn unmask_action_space_pick_pack_card(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PackOpen() {
            return;
        }
        let Some(state) = self.open_pack.as_ref() else {
            return;
        };
        let joker_count = self.jokers.len();
        let joker_slots = self.config.joker_slots;
        state
            .contents
            .iter()
            .enumerate()
            .filter(|(_, c)| match c {
                PackContent::Joker(j) => {
                    joker_count < joker_slots || j.edition() == Edition::Negative
                }
                _ => true,
            })
            .for_each(|(i, _)| {
                space
                    .unmask_pick_pack_card(i)
                    .expect("valid index for pick pack card");
            });
    }

    fn unmask_action_space_skip_pack(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PackOpen() {
            return;
        }
        space.unmask_skip_pack();
    }

    fn unmask_action_space_tarot_hand(&self, space: &mut ActionSpace) {
        let Stage::TarotHand(t) = self.stage else {
            return;
        };
        // select cards
        let selected_count = self.available.selected().len();
        if selected_count < t.max_targets() {
            self.available
                .cards_and_selected()
                .iter()
                .enumerate()
                .filter(|(_, (_, a))| !*a)
                .for_each(|(i, _)| {
                    space
                        .unmask_select_card(i)
                        .expect("valid index for selecting");
                });
        }
        // move cards
        self.available
            .cards()
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(i, _)| {
                space
                    .unmask_move_card_left(i)
                    .expect("valid index for move left");
            });
        self.available
            .cards()
            .iter()
            .rev()
            .skip(1)
            .rev()
            .enumerate()
            .for_each(|(i, _)| {
                space
                    .unmask_move_card_right(i)
                    .expect("valid index for move right");
            });
        if selected_count >= t.min_targets() && selected_count <= t.max_targets() {
            space.unmask_apply_tarot();
        }
    }

    // Get an action space, masked for legal actions only
    pub fn gen_action_space(&self) -> ActionSpace {
        let mut space = ActionSpace::from(self.config.clone());
        self.unmask_action_space_select_cards(&mut space);
        self.unmask_action_space_play_and_discard(&mut space);
        self.unmask_action_space_move_cards(&mut space);
        self.unmask_action_space_cash_out(&mut space);
        self.unmask_action_space_next_round(&mut space);
        self.unmask_action_space_select_blind(&mut space);
        self.unmask_action_space_buy_joker(&mut space);
        self.unmask_action_space_buy_consumable(&mut space);
        self.unmask_action_space_use_consumable(&mut space);
        self.unmask_action_space_tarot_hand(&mut space);
        self.unmask_action_space_sell_joker(&mut space);
        self.unmask_action_space_sell_consumable(&mut space);
        self.unmask_action_space_buy_pack(&mut space);
        self.unmask_action_space_pick_pack_card(&mut space);
        self.unmask_action_space_skip_pack(&mut space);
        space
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, Suit, Value};

    #[test]
    fn test_gen_moves_play() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        // nothing selected, nothing to play
        assert!(g.gen_actions_discard().is_none());

        g.available.extend(vec![ace]);
        g.select_card(ace).unwrap();
        let moves: Vec<Action> = g.gen_actions_play().expect("are plays").collect();
        assert_eq!(moves.len(), 1);

        g.available.extend(vec![ace, king]);
        g.select_card(ace).unwrap();
        g.select_card(king).unwrap();
        let moves: Vec<Action> = g.gen_actions_play().expect("are plays").collect();
        assert_eq!(moves.len(), 1);
    }

    #[test]
    fn test_gen_moves_discard() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        // nothing selected, nothing to discard
        assert!(g.gen_actions_discard().is_none());

        g.available.extend(vec![ace, king]);
        g.select_card(ace).unwrap();
        g.select_card(king).unwrap();
        let moves: Vec<Action> = g.gen_actions_discard().expect("are discards").collect();
        assert_eq!(moves.len(), 1);
    }

    #[test]
    fn test_unmask_action_space_select_cards() {
        let mut g = Game::default();
        g.deal();
        g.stage = Stage::Blind(Blind::Small);
        let mut space = ActionSpace::from(g.config.clone());

        // Default action space everything should be masked
        assert!(space.select_card[0] == 0);

        // Unmask card selects, we have all selects available
        g.unmask_action_space_select_cards(&mut space);
        assert!(space.select_card[0] == 1);

        // Make a fresh space
        space = ActionSpace::from(g.config.clone());
        // Select 2 cards, regenerate action space
        for _ in 0..2 {
            g.select_card(*g.available.not_selected().first().expect("is first card"))
                .expect("can select");
        }
        g.unmask_action_space_select_cards(&mut space);
        // Cannot select first and second, can select third
        assert!(space.select_card[0] == 0);
        assert!(space.select_card[1] == 0);
        assert!(space.select_card[2] == 1);
    }

    #[test]
    fn test_unmask_action_space_select_cards_max() {
        let mut g = Game::default();
        g.deal();
        g.stage = Stage::Blind(Blind::Small);
        let mut space = ActionSpace::from(g.config.clone());

        // Default action space everything should be masked
        assert!(space.select_card[0] == 0);

        // Unmask card selects, we have all selects available
        g.unmask_action_space_select_cards(&mut space);
        assert!(space.select_card[0] == 1);

        // Make a fresh space
        space = ActionSpace::from(g.config.clone());
        // Now select 5 cards, no more selects available, regenerate action space
        for _ in 0..g.config.selected_max {
            g.select_card(*g.available.not_selected().first().expect("is first card"))
                .expect("can select");
        }
        g.unmask_action_space_select_cards(&mut space);
        for i in 0..space.select_card.len() - 1 {
            assert!(space.select_card[i] == 0);
        }

        // If stage is not blind, don't alter space
        g.stage = Stage::Shop();
        space = ActionSpace::from(g.config.clone());
        space.select_card[0] = 1;
        assert!(space.select_card[0] == 1);
        g.unmask_action_space_select_cards(&mut space);
        assert!(space.select_card[0] == 1);
    }

    #[test]
    fn test_unmask_action_space_play_and_discard() {
        let mut g = Game::default();
        g.deal();
        g.stage = Stage::Blind(Blind::Small);
        let mut space = ActionSpace::from(g.config.clone());

        // Default action space everything should be masked
        assert!(space.play[0] == 0);
        assert!(space.discard[0] == 0);

        for c in g.available.cards()[0..5].to_vec() {
            g.select_card(c).unwrap();
        }
        // Unmask play/discard
        g.unmask_action_space_play_and_discard(&mut space);
        assert!(space.play[0] == 1);
        assert!(space.discard[0] == 1);
    }

    #[test]
    fn test_unmask_action_space_move_cards() {
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let mut space = ActionSpace::from(g.config.clone());

        // Default action space everything should be masked, since no cards available yet
        assert_eq!(g.available.cards().len(), 0);
        for i in 0..space.move_card_left.len() {
            assert!(space.move_card_left[i] == 0);
        }
        for i in 0..space.move_card_right.len() {
            assert!(space.move_card_right[i] == 0);
        }

        // deal and make available
        g.deal();
        // Unmask play/discard
        g.unmask_action_space_move_cards(&mut space);

        // Should be able to move left every available card except leftmost
        let available = g.available.cards().len();
        for i in 0..available - 1 {
            assert!(space.move_card_left[i] == 1);
        }
        for i in 0..available - 1 {
            assert!(space.move_card_right[i] == 1);
        }

        // Even when selected, we can still move cards
        let not_selected = g.available.not_selected();
        for c in &not_selected[0..5] {
            g.select_card(*c).unwrap();
        }

        // Get fresh action space and mask
        space = ActionSpace::from(g.config.clone());
        g.unmask_action_space_move_cards(&mut space);
        for i in 0..available - 1 {
            assert!(space.move_card_left[i] == 1);
        }
        for i in 0..available - 1 {
            assert!(space.move_card_right[i] == 1);
        }
    }
}
