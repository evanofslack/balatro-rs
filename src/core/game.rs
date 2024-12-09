use crate::core::action::{Action, MoveDirection};
use crate::core::ante::Ante;
use crate::core::card::Card;
use crate::core::deck::Deck;
use crate::core::effect::EffectRegistry;
use crate::core::error::GameError;
use crate::core::hand::{MadeHand, SelectHand};
use crate::core::joker::Jokers;
use crate::core::shop::Shop;
use crate::core::stage::{Blind, End, Stage};
use std::collections::HashSet;
use std::fmt;

use itertools::Itertools;

use super::effect::Effects;

const DEFAULT_ROUND_START: usize = 0;
const DEFAULT_PLAYS: usize = 4;
const DEFAULT_DISCARDS: usize = 4;
const DEFAULT_MONEY: usize = 0;
const DEFAULT_REWARD: usize = 0;
const DEFAULT_MONEY_PER_HAND: usize = 1;
const DEFAULT_INTEREST_RATE: f32 = 0.2;
const DEFAULT_INTEREST_MAX: usize = 5;
const HAND_SIZE: usize = 8;
const BASE_MULT: usize = 0;
const BASE_CHIPS: usize = 0;
const BASE_SCORE: usize = 0;

#[derive(Debug, Clone)]
pub struct Game {
    pub shop: Shop,
    pub deck: Deck,
    pub available: Vec<Card>,
    pub discarded: Vec<Card>,
    pub blind: Option<Blind>,
    pub stage: Stage,
    pub ante: Ante,
    pub action_history: Vec<Action>,
    pub round: usize,

    // jokers and their effects
    pub jokers: Vec<Jokers>,
    pub effect_registry: EffectRegistry,

    // playing
    pub plays: usize,
    pub discards: usize,
    pub reward: usize,
    pub money: usize,

    // for scoring
    pub chips: usize,
    pub mult: usize,
    pub score: usize,
}

impl Game {
    pub fn new() -> Self {
        Self {
            shop: Shop::new(),
            deck: Deck::default(),
            available: Vec::new(),
            discarded: Vec::new(),
            blind: None,
            stage: Stage::PreBlind,
            ante: Ante::One,
            action_history: Vec::new(),
            round: DEFAULT_ROUND_START,
            jokers: Vec::new(),
            effect_registry: EffectRegistry::new(),
            plays: DEFAULT_PLAYS,
            discards: DEFAULT_DISCARDS,
            reward: DEFAULT_REWARD,
            money: DEFAULT_MONEY,
            chips: BASE_CHIPS,
            mult: BASE_MULT,
            score: BASE_SCORE,
        }
    }

    pub fn start(&mut self) {
        // for now just move state to small blind
        self.stage = Stage::PreBlind;
        self.deal();
    }

    pub fn result(&self) -> Option<End> {
        match self.stage {
            Stage::End(end) => {
                return Some(end);
            }
            _ => return None,
        }
    }

    pub fn is_over(&self) -> bool {
        return self.result().is_some();
    }

    pub fn clear_blind(&mut self) {
        self.score = BASE_SCORE;
        self.plays = DEFAULT_PLAYS;
        self.discards = DEFAULT_DISCARDS;
        self.deal();
    }

    // draw from deck to available
    pub fn draw(&mut self, count: usize) {
        if let Some(drawn) = self.deck.draw(count) {
            self.available.extend(drawn);
        }
    }

    // shuffle and deal new cards to available
    pub fn deal(&mut self) {
        // add discarded and available back to deck, emptying in process
        self.deck.append(&mut self.discarded);
        self.deck.append(&mut self.available);
        self.deck.shuffle();
        self.draw(HAND_SIZE);
    }

    // remove specific cards from available, send to discarded, and draw equal number back to available
    fn _discard(&mut self, select: SelectHand, check: bool) -> Result<(), GameError> {
        if check {
            if self.discards <= 0 {
                return Err(GameError::NoRemainingDiscards);
            }
            self.discards -= 1;
        }
        // retain cards that we are not discarding
        let remove: HashSet<Card> = HashSet::from_iter(select.cards());
        // self.available.retain(|c| !remove.contains(c));

        let available = std::mem::take(&mut self.available);
        let (discarded, new_avail): (Vec<Card>, Vec<Card>) =
            available.into_iter().partition(|c| remove.contains(c));
        self.available = new_avail;
        self.discarded.extend(discarded);
        self.draw(select.cards().len());
        return Ok(());
    }

    // discard specific cards from available and draw equal number back to available
    pub fn discard(&mut self, select: SelectHand) -> Result<(), GameError> {
        return self._discard(select, true);
    }

    pub fn move_card(&mut self, direction: MoveDirection, card: Card) -> Result<(), GameError> {
        if let Some((i, _)) = self.available.iter().find_position(|c| c.id == card.id) {
            match direction {
                MoveDirection::Left => {
                    if i == 0 {
                        return Err(GameError::InvalidMoveDirection);
                    }
                    self.available.swap(i, i - 1);
                    return Ok(());
                }
                MoveDirection::Right => {
                    if i == self.available.len() {
                        return Err(GameError::InvalidMoveDirection);
                    }
                    self.available.swap(i, i + 1);
                    return Ok(());
                }
            }
        } else {
            return Err(GameError::NoCardMatch);
        }
    }

    pub fn calc_score(&mut self, hand: MadeHand) -> usize {
        // compute chips and mult from hand level
        self.chips += hand.rank.level().chips;
        self.mult += hand.rank.level().mult;

        // add chips for each played card
        let card_chips: usize = hand.hand.cards().iter().map(|c| c.chips()).sum();
        self.chips += card_chips;

        // Apply effects that modify game.chips and game.mult
        for e in self.effect_registry.on_score.clone() {
            match e {
                Effects::OnScore(f) => f(self),
                _ => (),
            }
        }

        // compute score
        let score = self.chips * self.mult;

        // reset chips and mult
        self.mult = BASE_MULT;
        self.chips = BASE_CHIPS;
        return score;
    }

    pub fn required_score(&self) -> Result<usize, GameError> {
        let base = self.ante.base();
        let required = match self.stage {
            Stage::Blind(Blind::Small) => base,
            Stage::Blind(Blind::Big) => (base as f32 * 1.5) as usize,
            Stage::Blind(Blind::Boss) => base * 2,
            // can only check score if in blind stage
            _ => return Err(GameError::InvalidStage),
        };
        return Ok(required);
    }

    pub fn calc_reward(&mut self, blind: Blind) -> Result<usize, GameError> {
        let mut interest = (self.money as f32 * DEFAULT_INTEREST_RATE).floor() as usize;
        if interest > DEFAULT_INTEREST_MAX {
            interest = DEFAULT_INTEREST_MAX
        }
        let base = blind.reward();
        let hand_bonus = self.plays * DEFAULT_MONEY_PER_HAND;
        let reward = base + interest + hand_bonus;
        return Ok(reward);
    }

    pub fn cashout(&mut self) -> Result<(), GameError> {
        self.money += self.reward;
        self.reward = 0;
        self.stage = Stage::Shop;
        return Ok(());
    }

    pub fn buy_joker(&mut self, joker: Jokers) -> Result<(), GameError> {
        if self.stage != Stage::Shop {
            return Err(GameError::InvalidStage);
        }
        self.jokers.push(joker);
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
        return Ok(());
    }

    pub fn select_blind(&mut self, blind: Blind) -> Result<(), GameError> {
        // can only set blind if stage is pre blind
        if self.stage != Stage::PreBlind {
            return Err(GameError::InvalidStage);
        }
        // provided blind must be expected next blind
        if let Some(current) = self.blind {
            if blind != current.next() {
                return Err(GameError::InvalidBlind);
            }
        } else {
            // if game just started, blind will be None, in which case
            // we can only set it to small.
            if blind != Blind::Small {
                return Err(GameError::InvalidBlind);
            }
        }
        self.blind = Some(blind);
        self.stage = Stage::Blind(blind);
        return Ok(());
    }

    pub fn next_round(&mut self) -> Result<(), GameError> {
        self.stage = Stage::PreBlind;
        self.round += 1;
        return Ok(());
    }

    // Returns true if blind passed, false if not.
    pub fn handle_score(&mut self, score: usize) -> Result<bool, GameError> {
        // can only handle score if stage is blind
        if !self.stage.is_blind() {
            return Err(GameError::InvalidStage);
        }

        self.score += score;
        let required = self.required_score()?;

        // blind not passed
        if self.score < required {
            // no more hands to play -> lose
            if self.plays == 0 {
                self.stage = Stage::End(End::Lose);
                return Ok(false);
            } else {
                // more hands to play, carry on
                return Ok(false);
            }
        }

        let blind = self.blind.expect("stage is blind");
        // score exceeds blind (blind passed).
        // handle reward then progress to next stage.
        let reward = self.calc_reward(blind)?;
        self.reward = reward;

        // passed boss blind, either win or progress ante
        if blind == Blind::Boss {
            if let Some(next_ante) = self.ante.next() {
                self.ante = next_ante;
            } else {
                self.stage = Stage::End(End::Win);
                return Ok(true);
            }
        };

        // finish blind, proceed to post blind
        self.stage = Stage::PostBlind;
        return Ok(true);
    }

    pub fn play(&mut self, select: SelectHand) -> Result<(), GameError> {
        if self.plays <= 0 {
            return Err(GameError::NoRemainingPlays);
        }
        self.plays -= 1;
        let best = select.best_hand()?;
        let score = self.calc_score(best);
        let pass_blind = self.handle_score(score)?;
        self._discard(select, false)?;
        if pass_blind {
            self.clear_blind();
        }
        return Ok(());
    }

    // get all legal Play moves that can be executed given current state
    pub fn gen_moves_play(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only play hand during blinds
        if !self.stage.is_blind() {
            return None;
        }
        // If no plays remaining, return None
        if self.plays <= 0 {
            return None;
        }
        // For all available cards, we can both play every combination
        // of 1, 2, 3, 4 or 5 cards.
        let combos = self
            .available
            .clone()
            .into_iter()
            .combinations(5)
            .chain(self.available.clone().into_iter().combinations(4))
            .chain(self.available.clone().into_iter().combinations(3))
            .chain(self.available.clone().into_iter().combinations(2))
            .chain(self.available.clone().into_iter().combinations(1))
            .map(|cards| Action::Play(SelectHand::new(cards)));
        return Some(combos);
    }

    // get all legal Play moves that can be executed given current state
    pub fn gen_moves_discard(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only discard during blinds
        if !self.stage.is_blind() {
            return None;
        }
        // If no discards remaining, return None
        if self.discards <= 0 {
            return None;
        }
        // For all available cards, we can both discard every combination
        // of 1, 2, 3, 4 or 5 cards.
        let combos = self
            .available
            .clone()
            .into_iter()
            .combinations(5)
            .chain(self.available.clone().into_iter().combinations(4))
            .chain(self.available.clone().into_iter().combinations(3))
            .chain(self.available.clone().into_iter().combinations(2))
            .chain(self.available.clone().into_iter().combinations(1))
            .map(|cards| Action::Discard(SelectHand::new(cards)));
        return Some(combos);
    }

    // get all legal move card moves
    pub fn gen_moves_move_card(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only move cards during blinds
        if !self.stage.is_blind() {
            return None;
        }
        // if 0 or 1 available cards, there are no possible moves
        if self.available.len() == 0 || self.available.len() == 1 {
            return None;
        }
        // We can move all cards left and right, except 1st card can only move right
        // and last card can only move left.
        let combos = self
            .available
            .clone()
            .into_iter()
            .skip(1)
            .rev()
            .skip(1)
            .flat_map(|c| {
                vec![
                    Action::MoveCard(MoveDirection::Left, c),
                    Action::MoveCard(MoveDirection::Right, c),
                ]
                .into_iter()
            })
            .chain(
                self.available
                    .clone()
                    .first()
                    .map(|c| Action::MoveCard(MoveDirection::Right, *c)),
            )
            .chain(
                self.available
                    .clone()
                    .last()
                    .map(|c| Action::MoveCard(MoveDirection::Left, *c)),
            );
        return Some(combos);
    }

    // get cash out move
    pub fn gen_moves_cash_out(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not post blind, cannot cash out
        if self.stage != Stage::PostBlind {
            return None;
        }
        return Some(vec![Action::CashOut(self.reward)].into_iter());
    }

    // get next round move
    pub fn gen_moves_next_round(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not shop, cannot next round
        if self.stage != Stage::Shop {
            return None;
        }
        return Some(vec![Action::NextRound()].into_iter());
    }

    // get select blind move
    pub fn gen_moves_select_blind(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not pre blind, cannot select blind
        if self.stage != Stage::PreBlind {
            return None;
        }
        if let Some(blind) = self.blind {
            return Some(vec![Action::SelectBlind(blind.next())].into_iter());
        } else {
            return Some(vec![Action::SelectBlind(Blind::Small)].into_iter());
        }
    }

    // get buy joker moves
    pub fn gen_moves_buy_joker(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not shop, cannot buy
        if self.stage != Stage::Shop {
            return None;
        }
        return self.shop.gen_moves_buy_joker();
    }

    // get all legal moves that can be executed given current state
    pub fn gen_moves(&self) -> impl Iterator<Item = Action> {
        let plays = self.gen_moves_play();
        let discards = self.gen_moves_discard();
        let move_cards = self.gen_moves_move_card();
        let cashouts = self.gen_moves_cash_out();
        let nextrounds = self.gen_moves_next_round();
        let selectblinds = self.gen_moves_select_blind();
        let buy_jokers = self.gen_moves_buy_joker();

        return plays
            .into_iter()
            .flatten()
            .chain(discards.into_iter().flatten())
            .chain(move_cards.into_iter().flatten())
            .chain(cashouts.into_iter().flatten())
            .chain(nextrounds.into_iter().flatten())
            .chain(selectblinds.into_iter().flatten())
            .chain(buy_jokers.into_iter().flatten());
    }

    pub fn handle_action(&mut self, action: Action) -> Result<(), GameError> {
        self.action_history.push(action.clone());
        return match action {
            Action::Play(hand) => match self.stage.is_blind() {
                true => self.play(hand),
                false => Err(GameError::InvalidAction),
            },
            Action::Discard(hand) => match self.stage.is_blind() {
                true => self.discard(hand),
                false => Err(GameError::InvalidAction),
            },
            Action::MoveCard(dir, card) => match self.stage.is_blind() {
                true => self.move_card(dir, card),
                false => Err(GameError::InvalidAction),
            },
            Action::CashOut(_reward) => match self.stage {
                Stage::PostBlind => self.cashout(),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyJoker(joker) => match self.stage {
                Stage::Shop => self.buy_joker(joker),
                _ => Err(GameError::InvalidAction),
            },
            Action::NextRound() => match self.stage {
                Stage::Shop => self.next_round(),
                _ => Err(GameError::InvalidAction),
            },
            Action::SelectBlind(blind) => match self.stage {
                Stage::PreBlind => self.select_blind(blind),
                _ => Err(GameError::InvalidAction),
            },
        };
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "deck length: {}", self.deck.len())?;
        writeln!(f, "available length: {}", self.available.len())?;
        writeln!(f, "discard length: {}", self.discarded.len())?;
        writeln!(f, "action history length: {}", self.action_history.len())?;
        writeln!(f, "blind: {:?}", self.blind)?;
        writeln!(f, "stage: {:?}", self.stage)?;
        writeln!(f, "ante: {:?}", self.ante)?;
        writeln!(f, "round: {}", self.round)?;
        writeln!(f, "hands remaining: {}", self.plays)?;
        writeln!(f, "discards remaining: {}", self.discards)?;
        writeln!(f, "money: {}", self.money)?;
        writeln!(f, "score: {}", self.score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::card::{Suit, Value};

    #[test]
    fn test_constructor() {
        let g = Game::new();
        assert_eq!(g.available.len(), 0);
        assert_eq!(g.deck.len(), 52);
        assert_eq!(g.mult, 0);
    }

    #[test]
    fn test_deal() {
        let mut g = Game::new();
        g.deal();
        // deck should be 7 cards smaller than we started with
        assert_eq!(g.deck.len(), 52 - HAND_SIZE);
        // should be 7 cards now available
        assert_eq!(g.available.len(), HAND_SIZE);
    }

    #[test]
    fn test_draw() {
        let mut g = Game::new();
        g.draw(1);
        assert_eq!(g.available.len(), 1);
        assert_eq!(g.deck.len(), 52 - 1);
        g.draw(3);
        assert_eq!(g.available.len(), 4);
        assert_eq!(g.deck.len(), 52 - 4);
    }
    #[test]
    fn test_discard() {
        let mut g = Game::new();
        g.deal();
        assert_eq!(g.available.len(), HAND_SIZE);
        assert_eq!(g.deck.len(), 52 - HAND_SIZE);
        // select first 4 cards
        let select = SelectHand::new(g.available[0..4].to_vec());
        let discard_res = g.discard(select.clone());
        assert!(discard_res.is_ok());
        // available should still be 7, we discarded then redrew to match
        assert_eq!(g.available.len(), HAND_SIZE);
        // deck is now smaller since we drew from it
        assert_eq!(g.deck.len(), 52 - HAND_SIZE - select.len());
    }

    #[test]
    fn test_score() {
        let g = &mut Game::new();
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let jack = Card::new(Value::Jack, Suit::Club);

        // Score [Ah, Kd, Jc]
        // High card (level 1) -> chips=5, mult=1
        // Played cards (1 ace) -> 11 chips
        // (5 + 11) * 1 = 16
        let cards = vec![ace, king, jack];
        let hand = SelectHand::new(cards).best_hand().unwrap();
        let score = g.calc_score(hand);
        assert_eq!(score, 16);

        // Score [Kd, Kd, Ah]
        // Pair (level 1) -> chips=10, mult=2
        // Played cards (2 kings) -> 10 + 10 == 20 chips
        // (10 + 20) * 2 = 60
        let cards = vec![king, king, ace];
        let hand = SelectHand::new(cards).best_hand().unwrap();
        let score = g.calc_score(hand);
        assert_eq!(score, 60);

        // Score [Ah, Ah, Ah, Kd]
        // Three of kind (level 1) -> chips=30, mult=3
        // Played cards (3 aces) -> 11 + 11 + 11 == 33 chips
        // (30 + 33) * 3 = 189
        let cards = vec![ace, ace, ace, king];
        let hand = SelectHand::new(cards).best_hand().unwrap();
        let score = g.calc_score(hand);
        assert_eq!(score, 189);

        // Score [Kd, Kd, Kd, Kd, Ah]
        // Four of kind (level 1) -> chips=60, mult=7
        // Played cards (4 kings) -> 10 + 10 + 10 + 10 == 40 chips
        // (60 + 40) * 7 = 700
        let cards = vec![king, king, king, king, ace];
        let hand = SelectHand::new(cards).best_hand().unwrap();
        let score = g.calc_score(hand);
        assert_eq!(score, 700);

        // Score [Jc, Jc, Jc, Jc, Jc]
        // Flush five (level 1) -> chips=160, mult=16
        // Played cards (5 jacks) -> 10 + 10 + 10 + 10 + 10 == 50 chips
        // (160 + 50) * 16 = 3360
        let cards = vec![jack, jack, jack, jack, jack];
        let hand = SelectHand::new(cards).best_hand().unwrap();
        let score = g.calc_score(hand);
        assert_eq!(score, 3360);
    }

    #[test]
    fn test_gen_moves_play() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let jack = Card::new(Value::Jack, Suit::Club);

        let mut g = Game::new();
        g.stage = Stage::Blind(Blind::Small);
        // Only 1 card available [(Ah)]
        // Playable moves: [Ah]
        g.available = vec![ace];
        let moves: Vec<Action> = g.gen_moves_play().expect("are plays").collect();
        assert_eq!(moves.len(), 1);

        // 2 cards available [Ah, Kd]
        // Playable moves: [(Ah, Kd), (Ah), (Kd)]
        g.available = vec![ace, king];
        let moves: Vec<Action> = g.gen_moves_play().expect("are plays").collect();
        assert_eq!(moves.len(), 3);

        // 3 cards available [Ah, Kd, Jc]
        // Playable moves: [(Ah, Kd, Jc), (Ah, Kd), (Ah, Jc), (Kd, Jc), (Ah), (Kd), (Jc)]
        g.available = vec![ace, king, jack];
        let moves: Vec<Action> = g.gen_moves_play().expect("are plays").collect();
        assert_eq!(moves.len(), 7);
    }

    #[test]
    fn test_gen_moves_discard() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let jack = Card::new(Value::Jack, Suit::Club);

        let mut g = Game::new();
        g.stage = Stage::Blind(Blind::Small);
        // Only 1 card available [(Ah)]
        // Playable moves: [Ah]
        g.available = vec![ace];
        let moves: Vec<Action> = g.gen_moves_discard().expect("are discards").collect();
        assert_eq!(moves.len(), 1);
        // let m = &moves[0];
        // // Test that we can apply that discard move to the game
        // m.apply(&mut g);
        // // available should still be 1, we discarded then redrew to match
        // assert_eq!(g.available.len(), 1);
        // // deck is now smaller since we drew from it
        // assert_eq!(g.deck.len(), 52 - 1);

        // 2 cards available [Ah, Kd]
        // Playable moves: [(Ah, Kd), (Ah), (Kd)]
        g.available = vec![ace, king];
        let moves: Vec<Action> = g.gen_moves_discard().expect("are discards").collect();
        assert_eq!(moves.len(), 3);

        // 3 cards available [Ah, Kd, Jc]
        // Playable moves: [(Ah, Kd, Jc), (Ah, Kd), (Ah, Jc), (Kd, Jc), (Ah), (Kd), (Jc)]
        g.available = vec![ace, king, jack];
        let moves: Vec<Action> = g.gen_moves_discard().expect("are discards").collect();
        assert_eq!(moves.len(), 7);
    }
}
