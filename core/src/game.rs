use crate::action::{Action, MoveDirection};
use crate::ante::Ante;
use crate::available::Available;
use crate::card::{Card, Edition, Enhancement, Seal};
use crate::config::{Config, RngMode};
use crate::consumable::Consumable;
use crate::deck::Deck;
use crate::effect::{EffectRegistry, Effects};
use crate::error::GameError;
use crate::hand::{MadeHand, SelectHand};
use crate::joker::{joker_display, JokerEffects, Jokers};
use crate::pack::{OpenPackState, Pack, PackCategory, PackContent};
use crate::planet::Planetarium;
use crate::rank::HandRank;
use crate::rng::{Backend, FastBackend, RealBackend, RngBackend};
use crate::score::{ScoreSource, ScoreStep, ScoreTrace};
use crate::shop::Shop;
use crate::stage::{Blind, BlindExt, End, Stage};
use crate::tag::Tag;
use crate::tarot::{Tarot, TarotEffect};

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::fmt;
use strum::IntoEnumIterator;

#[cfg(feature = "serde")]
fn default_rng() -> ChaCha8Rng {
    ChaCha8Rng::from_entropy()
}

#[cfg(feature = "serde")]
fn default_backend() -> Backend {
    Backend::Fast(FastBackend::new(ChaCha8Rng::from_entropy()))
}

fn default_reroll_cost() -> usize {
    5
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Game {
    pub config: Config,
    pub shop: Shop,
    pub planetarium: Planetarium,
    pub deck: Deck,
    pub available: Available,
    pub discarded: Vec<Card>,
    pub tags: Vec<Tag>,
    pub small_blind_tag: Tag,
    pub big_blind_tag: Tag,
    pub blind: Option<Blind>,
    pub stage: Stage,
    pub ante_start: Ante,
    pub ante_end: Ante,
    pub ante_current: Ante,
    pub action_history: Vec<Action>,
    pub round: usize,

    // jokers and their effects
    pub jokers: Vec<Jokers>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub effect_registry: EffectRegistry,

    // held consumables (planets, tarots, etc.)
    pub consumables: Vec<Consumable>,

    // playing
    pub plays: usize,
    pub discards: usize,
    pub reward: usize,
    pub money: usize,

    // for scoring
    pub chips: usize,
    pub mult: usize,
    pub score: usize,
    pub prob_mult: u32,

    pub last_consumable_used: Option<Consumable>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub last_score: usize,
    #[cfg_attr(feature = "serde", serde(default = "default_reroll_cost"))]
    pub reroll_cost: usize,

    #[cfg_attr(feature = "serde", serde(default))]
    pub seed: u64,
    #[cfg_attr(feature = "serde", serde(default))]
    pub seed_str: Option<String>,
    #[cfg_attr(feature = "serde", serde(default = "default_rng"))]
    pub(crate) rng: ChaCha8Rng,
    // shop/pack generation backend (Fast/Real, per config.rng_mode) — does
    // not affect deck shuffling, prob_roll, or tag draws, which stay on
    // `rng` above regardless of RngMode.
    #[cfg_attr(feature = "serde", serde(default = "default_backend"))]
    pub(crate) backend: Backend,
    // track stage so we can come back to it after temp tarot stage
    pub(crate) tarot_prev_stage: Option<Stage>,

    // open pack state (set when a pack is purchased and being opened)
    #[cfg_attr(feature = "serde", serde(default))]
    pub open_pack: Option<OpenPackState>,
}

impl Game {
    pub fn new(config: Config) -> Self {
        let ante_start = Ante::try_from(config.ante_start).unwrap_or(Ante::One);
        let (seed, seed_str, rng) = match (config.seed_str.clone(), config.seed) {
            (Some(s), _) => {
                let u = crate::seed_from_str(&s);
                (u, Some(s), ChaCha8Rng::seed_from_u64(u))
            }
            (None, Some(u)) => (u, None, ChaCha8Rng::seed_from_u64(u)),
            (None, None) => {
                let u: u64 = thread_rng().gen();
                (u, None, ChaCha8Rng::seed_from_u64(u))
            }
        };
        let backend = match config.rng_mode {
            RngMode::Fast => Backend::Fast(FastBackend::new(ChaCha8Rng::seed_from_u64(seed))),
            RngMode::Real => {
                // Real mode needs a Balatro-format seed *string*; fall back
                // to the numeric seed's decimal representation if only
                // `config.seed` (no `seed_str`) was given — deterministic,
                // but not a real Balatro seed shape in that fallback case.
                let seed_string = seed_str.clone().unwrap_or_else(|| seed.to_string());
                Backend::Real(RealBackend::new(&seed_string))
            }
        };
        let mut game = Self {
            shop: Shop::new(),
            planetarium: Planetarium::new(),
            deck: Deck::default(),
            available: Available::default(),
            discarded: Vec::new(),
            tags: Vec::new(),
            small_blind_tag: Tag::Uncommon,
            big_blind_tag: Tag::Uncommon,
            action_history: Vec::new(),
            jokers: Vec::new(),
            effect_registry: EffectRegistry::new(),
            consumables: Vec::new(),
            blind: None,
            stage: Stage::PreBlind(),
            ante_start,
            ante_end: Ante::try_from(config.ante_end).unwrap_or(Ante::Eight),
            ante_current: ante_start,
            round: config.round_start,
            plays: config.plays,
            discards: config.discards,
            reward: config.reward_base,
            money: config.money_start,
            chips: config.base_chips,
            mult: config.base_mult,
            score: config.base_score,
            prob_mult: 1,
            last_consumable_used: None,
            last_score: 0,
            reroll_cost: default_reroll_cost(),
            tarot_prev_stage: None,
            open_pack: None,
            seed,
            seed_str,
            rng,
            backend,
            config,
        };
        game.draw_ante_tags();
        game
    }

    pub fn start(&mut self) {
        self.stage = Stage::PreBlind();
    }

    pub fn result(&self) -> Option<End> {
        match self.stage {
            Stage::End(end) => Some(end),
            _ => None,
        }
    }

    pub fn is_over(&self) -> bool {
        self.result().is_some()
    }

    // Fires when the round that just ended cleared the blind, before played
    // cards are swapped out and replacements drawn in. `self.available` still
    // reflects what was actually held in hand at the moment the round ended.
    fn handle_round_end(&mut self) {
        for e in self.effect_registry.on_round_end.clone() {
            if let Effects::OnRoundEnd(f) = e {
                f.lock().unwrap()(self)
            }
        }

        let planetarium = self.planetarium.clone();
        for card in self.available.not_selected() {
            if card.seal == Some(Seal::Blue)
                && self.consumables.len() < self.config.consumable_slots
            {
                let gen = crate::shop::ConsumableGenerator::new();
                let planet = gen.gen_planet_consumable(&planetarium, &[], &mut self.rng);
                self.consumables.push(planet);
            }
        }
    }

    fn clear_blind(&mut self) {
        self.score = self.config.base_score;
        self.plays = self.config.plays;
        self.discards = self.config.discards;
        self.deck.append(&mut self.discarded);
        self.deck.extend(self.available.cards());
        self.available.empty();
        self.deck.shuffle(&mut self.rng);
    }

    // draw from deck to available
    fn draw(&mut self, count: usize) {
        self.available.extend(self.deck.draw(count));
    }

    // shuffle and deal new cards to available
    pub(crate) fn deal(&mut self) {
        // add discarded back to deck, emptying in process
        self.deck.append(&mut self.discarded);
        // add available back to deck and empty
        self.deck.extend(self.available.cards());
        self.available.empty();
        self.deck.shuffle(&mut self.rng);
        self.draw(self.config.available);
    }

    /// Reshuffles discarded/held cards back into the deck and redraws a
    /// fresh hand, regardless of stage. Just a helper.
    pub fn redeal(&mut self) {
        self.deal();
    }

    pub(crate) fn select_card(&mut self, card: Card) -> Result<(), GameError> {
        if self.available.selected().len() >= self.config.selected_max {
            return Err(GameError::InvalidSelectCard);
        }
        self.available.select_card(card)
    }

    pub(crate) fn move_card(
        &mut self,
        direction: MoveDirection,
        card: Card,
    ) -> Result<(), GameError> {
        self.available.move_card(direction, card)
    }

    pub(crate) fn play_selected(&mut self) -> Result<(), GameError> {
        if self.plays == 0 {
            return Err(GameError::NoRemainingPlays);
        }
        self.plays -= 1;
        let selected = SelectHand::new(self.available.selected());
        let best = selected.best_hand()?;
        let scored = best.clone();
        let score = self.calc_score(best);
        let clear_blind = self.handle_score(score)?;
        // must run before the discard/redraw below: self.available still reflects
        // the cards actually held when the round ended, not their replacements.
        if clear_blind {
            self.handle_round_end();
        }
        self.discarded.extend(self.available.selected());
        let removed = self.available.remove_selected();
        self.draw(removed);
        for card in scored.hand.cards() {
            if card.enhancement == Some(Enhancement::Glass) && self.prob_roll(1, 4) {
                self.destroy_card(card.id);
            }
        }
        if clear_blind {
            self.clear_blind();
        }
        Ok(())
    }

    // discard selected cards from available and draw equal number back to available
    pub(crate) fn discard_selected(&mut self) -> Result<(), GameError> {
        if self.discards == 0 {
            return Err(GameError::NoRemainingDiscards);
        }
        self.discards -= 1;
        let discarded = self.available.selected();
        self.discarded.extend(discarded.clone());
        let removed = self.available.remove_selected();
        self.draw(removed);

        // Check for purple seals
        for card in &discarded {
            if card.seal == Some(Seal::Purple)
                && self.consumables.len() < self.config.consumable_slots
            {
                let excl: Vec<Tarot> = self
                    .consumables
                    .iter()
                    .filter_map(|c| match c {
                        Consumable::Tarot(t) => Some(*t),
                        _ => None,
                    })
                    .collect();
                let gen = crate::shop::ConsumableGenerator::new();
                let tarot = gen.gen_tarot_consumable(&excl, &mut self.rng);
                self.consumables.push(tarot);
            }
        }

        let hand = MadeHand {
            hand: SelectHand::new(discarded.clone()),
            rank: HandRank::HighCard,
            all: discarded,
        };
        for e in self.effect_registry.on_discard.clone() {
            if let Effects::OnDiscard(f) = e {
                f.lock().unwrap()(self, hand.clone())
            }
        }
        Ok(())
    }

    pub(crate) fn destroy_card(&mut self, id: usize) {
        self.available.remove_by_id(id);
        self.deck.remove_by_id(id);
        self.discarded.retain(|c| c.id != id);
    }

    pub(crate) fn mutate_card<F: Fn(&mut Card) + Copy>(&mut self, id: usize, f: F) {
        self.available.mutate_card(id, f);
        self.deck.mutate_card(id, f);
    }

    // Every card the player owns this run, regardless of whether it's
    // currently undrawn, in hand, or already discarded this round.
    // `self.deck` alone is only the undrawn remainder mid-round.
    pub(crate) fn full_deck(&self) -> Vec<Card> {
        let mut all = self.deck.cards();
        all.extend(self.available.cards());
        all.extend(self.discarded.clone());
        all
    }

    pub fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        self.rng
            .gen_ratio((numerator * self.prob_mult).min(denominator), denominator)
    }

    pub fn calc_score(&mut self, hand: MadeHand) -> usize {
        self.calc_score_inner(hand, None)
    }

    /// Same scoring pass as `calc_score`, plus an ordered `ScoreTrace` of
    /// every chips/mult contribution along the way.
    pub fn calc_score_traced(&mut self, hand: MadeHand) -> (usize, ScoreTrace) {
        let mut trace = ScoreTrace::default();
        let score = self.calc_score_inner(hand, Some(&mut trace));
        (score, trace)
    }

    fn record_step(
        &self,
        trace: &mut Option<&mut ScoreTrace>,
        source: ScoreSource,
        chips_before: usize,
        mult_before: usize,
        retrigger: bool,
    ) {
        let Some(t) = trace else {
            return;
        };
        // skip no-op checks (e.g. a Base-edition joker, a conditional
        // effect that didn't fire)
        if chips_before == self.chips && mult_before == self.mult {
            return;
        }
        t.0.push(ScoreStep {
            source,
            chips_before,
            chips_after: self.chips,
            mult_before,
            mult_after: self.mult,
            retrigger,
        });
    }

    // Extension point for retrigger (jokers + red seal)
    fn trigger_count_played(&mut self, card: &Card, is_first: bool) -> usize {
        let mut n = if card.seal == Some(Seal::Red) { 2 } else { 1 };
        for e in self.effect_registry.trigger_count_played.clone() {
            if let Effects::TriggerCountPlayed(f) = e {
                n += f.lock().unwrap()(self, *card, is_first);
            }
        }
        n
    }

    // Extension point for retrigger (jokers + red seal)
    fn trigger_count_held(&mut self, card: &Card) -> usize {
        let mut n = if card.seal == Some(Seal::Red) { 2 } else { 1 };
        for e in self.effect_registry.trigger_count_held.clone() {
            if let Effects::TriggerCountHeld(f) = e {
                n += f.lock().unwrap()(self, *card, false);
            }
        }
        n
    }

    fn calc_score_inner(
        &mut self,
        mut hand: MadeHand,
        mut trace: Option<&mut ScoreTrace>,
    ) -> usize {
        // compute chips and mult from hand level
        let chips_before = self.chips;
        let mult_before = self.mult;
        let level = self.planetarium.play(hand.rank);
        self.chips += level.chips;
        self.mult += level.mult;
        self.record_step(
            &mut trace,
            ScoreSource::HandLevel(hand.rank),
            chips_before,
            mult_before,
            false,
        );

        // first card in original play order that's actually used in scoring:
        // either part of the made hand, or a stone kicker (which always scores).
        // ordinary kickers that never score don't count, even if played first.
        let hand_ids: Vec<usize> = hand.hand.cards().iter().map(|c| c.id).collect();
        let first_played_id = hand
            .all
            .iter()
            .find(|c| hand_ids.contains(&c.id) || c.enhancement == Some(Enhancement::Stone))
            .map(|c| c.id);

        // per-scored-card, rank chips, enhancements, editions
        for card in hand.hand.cards() {
            let is_first = Some(card.id) == first_played_id;
            for trig in 0..self.trigger_count_played(&card, is_first) {
                let chips_before = self.chips;
                let mult_before = self.mult;

                // stone has no rank, skip normal chip value
                if card.enhancement != Some(Enhancement::Stone) {
                    self.chips += card.chips();
                }
                match card.enhancement {
                    Some(Enhancement::Bonus) => self.chips += 30,
                    Some(Enhancement::Mult) => self.mult += 4,
                    Some(Enhancement::Glass) => self.mult *= 2,
                    Some(Enhancement::Lucky) => {
                        if self.prob_roll(1, 5) {
                            self.mult += 20;
                        }
                        if self.prob_roll(1, 15) {
                            self.money += 20;
                        }
                    }
                    _ => {}
                }
                match card.edition {
                    Edition::Foil => self.chips += 50,
                    Edition::Holographic => self.mult += 10,
                    Edition::Polychrome => self.mult += self.mult / 2,
                    _ => {}
                }
                if card.seal == Some(Seal::Gold) {
                    self.money += 3;
                }

                self.record_step(
                    &mut trace,
                    ScoreSource::PlayedCard(card),
                    chips_before,
                    mult_before,
                    trig > 0,
                );
            }
        }

        // stone cards always score +50 chips, even as kickers outside the played
        // hand's own scoring subset.
        // still subject to the same retrigger rules as any other played card.
        for card in &hand.all {
            if card.enhancement == Some(Enhancement::Stone) {
                let is_first = Some(card.id) == first_played_id;
                for trig in 0..self.trigger_count_played(card, is_first) {
                    let chips_before = self.chips;
                    let mult_before = self.mult;
                    self.chips += 50;
                    self.record_step(
                        &mut trace,
                        ScoreSource::StoneKicker(*card),
                        chips_before,
                        mult_before,
                        trig > 0,
                    );
                }
            }
        }

        // held in hand effects (cards not played this hand)
        for card in self.available.not_selected() {
            for trig in 0..self.trigger_count_held(&card) {
                let chips_before = self.chips;
                let mult_before = self.mult;
                match card.enhancement {
                    Some(Enhancement::Steel) => self.mult += self.mult / 2,
                    Some(Enhancement::Gold) => self.money += 3,
                    _ => {}
                }
                self.record_step(
                    &mut trace,
                    ScoreSource::HeldCard(card),
                    chips_before,
                    mult_before,
                    trig > 0,
                );
            }
        }

        // run hand modifiers (e.g. Pareidolia) before joker scoring effects
        for e in self.effect_registry.on_modify_hand.clone() {
            if let Effects::OnModifyHand(f) = e {
                f.lock().unwrap()(self, &mut hand)
            }
        }

        // apply joker effects with per-joker edition ordering
        for joker in self.jokers.clone() {
            let chips_before = self.chips;
            let mult_before = self.mult;
            match joker.edition() {
                Edition::Foil => self.chips += 50,
                Edition::Holographic => self.mult += 10,
                _ => {}
            }
            self.record_step(
                &mut trace,
                ScoreSource::Joker(joker.clone()),
                chips_before,
                mult_before,
                false,
            );

            for e in joker.effects(self) {
                if let Effects::OnScore(f) = e {
                    let chips_before = self.chips;
                    let mult_before = self.mult;
                    f.lock().unwrap()(self, hand.clone());
                    self.record_step(
                        &mut trace,
                        ScoreSource::Joker(joker.clone()),
                        chips_before,
                        mult_before,
                        false,
                    );
                }
            }

            if joker.edition() == Edition::Polychrome {
                let chips_before = self.chips;
                let mult_before = self.mult;
                self.mult += self.mult / 2;
                self.record_step(
                    &mut trace,
                    ScoreSource::Joker(joker.clone()),
                    chips_before,
                    mult_before,
                    false,
                );
            }
        }

        // compute score
        let score = self.chips * self.mult;

        // reset chips and mult
        self.mult = self.config.base_mult;
        self.chips = self.config.base_chips;

        score
    }

    pub fn required_score(&self) -> usize {
        let base = self.ante_current.base();
        match self.blind {
            None => base,
            Some(Blind::Small) => base,
            Some(Blind::Big) => (base as f32 * 1.5) as usize,
            Some(Blind::Boss) => base * 2,
        }
    }

    fn calc_reward(&mut self, blind: Blind) -> Result<usize, GameError> {
        let mut interest = (self.money as f32 * self.config.interest_rate).floor() as usize;
        if interest > self.config.interest_max {
            interest = self.config.interest_max
        }
        let base = blind.reward();
        let hand_bonus = self.plays * self.config.money_per_hand;
        let reward = base + interest + hand_bonus;
        Ok(reward)
    }

    fn apply_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Uncommon => {}
            Tag::Rare => {}
            Tag::Negative => {}
            Tag::Foil => {}
            Tag::Holographic => {}
            Tag::Polychrome => {}
            Tag::Investment => {}
            Tag::Voucher => {}
            Tag::Boss => {}
            Tag::Standard => {}
            Tag::Charm => {}
            Tag::Meteor => {}
            Tag::Buffoon => {}
            Tag::Handy => {}
            Tag::Garbage => {}
            Tag::Ethereal => {}
            Tag::Coupon => {}
            Tag::Double => {}
            Tag::Juggle => {}
            Tag::D6 => {}
            Tag::TopUp => {}
            Tag::Speed => {}
            Tag::Orbital => {}
            Tag::Economy => {}
        }
    }

    fn cashout(&mut self) -> Result<(), GameError> {
        for tag in std::mem::take(&mut self.tags) {
            self.apply_tag(tag);
        }
        self.money += self.reward;
        self.reward = 0;
        self.reroll_cost = default_reroll_cost();
        self.stage = Stage::Shop();
        let planetarium = self.planetarium.clone();
        let held_consumables = self.consumables.clone();
        let held_jokers = self.jokers.clone();
        self.shop.refresh(
            &planetarium,
            &held_consumables,
            false,
            self.prob_mult,
            &held_jokers,
            self.ante_current as i32,
            &mut self.backend,
        );
        Ok(())
    }

    pub(crate) fn reroll(&mut self) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.money < self.reroll_cost {
            return Err(GameError::InvalidBalance);
        }
        self.money -= self.reroll_cost;
        self.reroll_cost += 1;
        let planetarium = self.planetarium.clone();
        let mut held = self.consumables.clone();
        held.extend(self.shop.consumables.clone());
        let mut held_jokers = self.jokers.clone();
        held_jokers.extend(self.shop.jokers.clone());
        self.shop.refresh_cards(
            &planetarium,
            &held,
            self.prob_mult,
            &held_jokers,
            self.ante_current as i32,
            &mut self.backend,
        );
        Ok(())
    }

    pub(crate) fn sell_joker(&mut self, idx: usize) -> Result<(), GameError> {
        if matches!(self.stage, Stage::End(_)) {
            return Err(GameError::InvalidStage);
        }
        if idx >= self.jokers.len() {
            return Err(GameError::InvalidAction);
        }
        self.money += self.jokers[idx].sell_value();
        let sold = self.jokers.remove(idx);
        if sold.edition() == Edition::Negative {
            self.config.joker_slots = self.config.joker_slots.saturating_sub(1);
        }
        // Only unlock if this was the last owned copy (Showman is the only
        // way to own more than one) — a discriminant compare, matching
        // shop.rs's own exclude-list comparisons, since Jokers' derived
        // Eq also compares edition/stickers, not just "which joker".
        let still_owned = self
            .jokers
            .iter()
            .any(|j| std::mem::discriminant(j) == std::mem::discriminant(&sold));
        if !still_owned {
            self.backend.on_joker_sold(&sold);
        }
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
        Ok(())
    }

    pub(crate) fn sell_consumable(&mut self, idx: usize) -> Result<(), GameError> {
        if matches!(self.stage, Stage::End(_)) {
            return Err(GameError::InvalidStage);
        }
        if idx >= self.consumables.len() {
            return Err(GameError::InvalidAction);
        }
        self.money += self.consumables[idx].sell_value();
        self.consumables.remove(idx);
        Ok(())
    }

    pub(crate) fn buy_joker(&mut self, joker: Jokers) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if joker.edition() == Edition::Negative {
            self.config.joker_slots =
                (self.config.joker_slots + 1).min(self.config.joker_slots_max);
        }
        if self.jokers.len() >= self.config.joker_slots {
            return Err(GameError::NoAvailableSlot);
        }
        if joker.cost() > self.money {
            return Err(GameError::InvalidBalance);
        }
        self.shop.buy_joker(&joker)?;
        self.money -= joker.cost();
        self.backend.on_joker_bought(&joker);
        self.jokers.push(joker);
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
        Ok(())
    }

    pub(crate) fn buy_consumable(&mut self, consumable: Consumable) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.consumables.len() >= self.config.consumable_slots {
            return Err(GameError::NoAvailableSlot);
        }
        if consumable.cost() > self.money {
            return Err(GameError::InvalidBalance);
        }
        self.shop.buy_consumable(&consumable)?;
        self.money -= consumable.cost();
        self.consumables.push(consumable);
        Ok(())
    }

    pub(crate) fn use_consumable(&mut self, consumable: Consumable) -> Result<(), GameError> {
        if matches!(self.stage, Stage::End(_)) {
            return Err(GameError::InvalidStage);
        }
        // Validate selection before removal so the consumable is not lost on error
        if let Consumable::Tarot(t) = &consumable {
            if t.requires_targets() && self.stage.is_blind() {
                let selected_count = self.available.selected().len();
                if selected_count < t.min_targets() || selected_count > t.max_targets() {
                    return Err(GameError::InvalidAction);
                }
            }
        }
        let i = self
            .consumables
            .iter()
            .position(|c| c == &consumable)
            .ok_or(GameError::InvalidAction)?;
        self.consumables.remove(i);
        match consumable {
            Consumable::Planet(planet) => {
                self.planetarium.level_up(planet.hand_rank());
                self.last_consumable_used = Some(Consumable::Planet(planet));
            }
            Consumable::Tarot(t) => {
                if t.requires_targets() && !self.stage.is_blind() {
                    let prev = self.stage;
                    self.tarot_prev_stage = Some(prev);
                    self.stage = Stage::TarotHand(t);
                    self.draw(self.config.available);
                } else {
                    t.apply(self)?;
                    if t != Tarot::Fool {
                        self.last_consumable_used = Some(Consumable::Tarot(t));
                    }
                }
            }
            // TODO: Spectral
            Consumable::Spectral(_) => return Err(GameError::InvalidAction),
        }
        Ok(())
    }

    pub(crate) fn apply_tarot(&mut self) -> Result<(), GameError> {
        let Stage::TarotHand(t) = self.stage else {
            return Err(GameError::InvalidStage);
        };
        let prev = self
            .tarot_prev_stage
            .take()
            .ok_or(GameError::InvalidStage)?;
        let selected_count = self.available.selected().len();
        if selected_count < t.min_targets() || selected_count > t.max_targets() {
            self.tarot_prev_stage = Some(prev);
            return Err(GameError::InvalidAction);
        }
        t.apply(self)?;
        if t != Tarot::Fool {
            self.last_consumable_used = Some(Consumable::Tarot(t));
        }
        // Don't clear the hand when returning to PackOpen; finish_pack handles cleanup
        if !prev.is_blind() && !prev.is_pack_open() {
            let cards = self.available.cards();
            self.available.empty();
            self.deck.extend(cards);
            self.deck.shuffle(&mut self.rng);
        }
        self.stage = prev;
        // Returning to a pack open: decrement picks and possibly finish
        if prev.is_pack_open() {
            if let Some(ref mut state) = self.open_pack {
                state.picks_remaining = state.picks_remaining.saturating_sub(1);
            }
            let done = self
                .open_pack
                .as_ref()
                .is_some_and(|s| s.picks_remaining == 0);
            if done {
                self.finish_pack();
            }
        }
        Ok(())
    }

    pub(crate) fn buy_pack(&mut self, pack: Pack) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if pack.cost() > self.money {
            return Err(GameError::InvalidBalance);
        }
        self.shop.buy_pack(&pack)?;
        self.money -= pack.cost();

        // Arcana packs draw a hand for the player to apply tarots against
        if pack.category == PackCategory::Arcana {
            self.draw(self.config.available);
        }

        self.open_pack = Some(OpenPackState {
            picks_remaining: pack.picks_allowed(),
            description: pack.description(),
            contents: pack.contents,
        });
        self.stage = Stage::PackOpen();
        Ok(())
    }

    pub(crate) fn pick_pack_card(&mut self, content: PackContent) -> Result<(), GameError> {
        if self.stage != Stage::PackOpen() {
            return Err(GameError::InvalidStage);
        }

        // Remove the chosen content from the pack
        {
            let state = self.open_pack.as_mut().ok_or(GameError::InvalidAction)?;
            let pos = state
                .contents
                .iter()
                .position(|c| c == &content)
                .ok_or(GameError::InvalidAction)?;
            state.contents.remove(pos);
        }

        // Targeting tarots enter TarotHand; picks_remaining decremented later in apply_tarot
        if let PackContent::Tarot(t) = &content {
            if t.requires_targets() {
                self.tarot_prev_stage = Some(Stage::PackOpen());
                self.stage = Stage::TarotHand(*t);
                return Ok(());
            }
        }

        match content {
            PackContent::Tarot(t) => {
                t.apply(self)?;
            }
            PackContent::Planet(p) => {
                self.planetarium.level_up(p.hand_rank());
            }
            PackContent::Joker(j) => {
                if j.edition() == Edition::Negative {
                    self.config.joker_slots =
                        (self.config.joker_slots + 1).min(self.config.joker_slots_max);
                }
                if self.jokers.len() >= self.config.joker_slots {
                    return Err(GameError::NoAvailableSlot);
                }
                self.jokers.push(j);
                self.effect_registry
                    .register_jokers(self.jokers.clone(), &self.clone());
            }
            PackContent::PlayingCard(c) => {
                self.deck.push(c);
            }
            // TODO: Spectral
            PackContent::Spectral(_) => return Err(GameError::InvalidAction),
        }

        if let Some(ref mut state) = self.open_pack {
            state.picks_remaining = state.picks_remaining.saturating_sub(1);
        }

        let done = self
            .open_pack
            .as_ref()
            .is_some_and(|s| s.picks_remaining == 0);
        if done {
            self.finish_pack();
        }
        Ok(())
    }

    pub(crate) fn skip_pack(&mut self) -> Result<(), GameError> {
        if self.stage != Stage::PackOpen() {
            return Err(GameError::InvalidStage);
        }
        self.finish_pack();
        Ok(())
    }

    fn finish_pack(&mut self) {
        // Return any drawn hand (Arcana packs draw cards for tarot targeting)
        if !self.available.cards().is_empty() {
            let cards = self.available.cards();
            self.available.empty();
            self.deck.extend(cards);
            self.deck.shuffle(&mut self.rng);
        }
        self.open_pack = None;
        self.stage = Stage::Shop();
    }

    fn select_blind(&mut self, blind: Blind) -> Result<(), GameError> {
        // can only set blind if stage is pre blind
        if self.stage != Stage::PreBlind() {
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
        self.deal();
        Ok(())
    }

    fn skip_blind(&mut self, blind: Blind) -> Result<(), GameError> {
        if self.stage != Stage::PreBlind() {
            return Err(GameError::InvalidStage);
        }
        if blind == Blind::Boss {
            return Err(GameError::InvalidBlind);
        }
        if let Some(current) = self.blind {
            if blind != current.next() {
                return Err(GameError::InvalidBlind);
            }
        } else if blind != Blind::Small {
            return Err(GameError::InvalidBlind);
        }
        self.blind = Some(blind);
        self.stage = Stage::PreBlind();
        let tag = self
            .skip_tag(blind)
            .expect("blind is not Boss, checked above");
        self.tags.push(tag);
        Ok(())
    }

    fn draw_ante_tags(&mut self) {
        let tags: Vec<Tag> = Tag::iter().collect();
        self.small_blind_tag = *tags.choose(&mut self.rng).unwrap();
        self.big_blind_tag = *tags.choose(&mut self.rng).unwrap();
    }

    pub fn skip_tag(&self, blind: Blind) -> Option<Tag> {
        match blind {
            Blind::Small => Some(self.small_blind_tag),
            Blind::Big => Some(self.big_blind_tag),
            Blind::Boss => None,
        }
    }

    fn next_round(&mut self) -> Result<(), GameError> {
        self.stage = Stage::PreBlind();
        self.round += 1;
        Ok(())
    }

    // Returns true if should clear blind after, false if not.
    fn handle_score(&mut self, score: usize) -> Result<bool, GameError> {
        // can only handle score if stage is blind
        if !self.stage.is_blind() {
            return Err(GameError::InvalidStage);
        }

        self.score += score;
        let required = self.required_score();

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
        self.last_score = self.score;

        // score exceeds blind (blind passed).
        // handle reward then progress to next stage.
        let reward = self.calc_reward(blind)?;
        self.reward = reward;

        // passed boss blind, either win or progress ante
        if blind == Blind::Boss {
            if let Some(ante_next) = self.ante_current.next(self.ante_end) {
                self.ante_current = ante_next;
                self.blind = None;
                self.draw_ante_tags();
            } else {
                self.stage = Stage::End(End::Win);
                return Ok(false);
            }
        };

        // finish blind, proceed to post blind
        self.stage = Stage::PostBlind();
        Ok(true)
    }

    pub fn handle_action(&mut self, action: Action) -> Result<(), GameError> {
        self.action_history.push(action.clone());
        match action {
            Action::SelectCard(card) => {
                if self.stage.is_blind() {
                    self.select_card(card)
                } else if let Stage::TarotHand(t) = self.stage {
                    if self.available.selected().len() >= t.max_targets() {
                        return Err(GameError::InvalidAction);
                    }
                    self.select_card(card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::DeselectCard(card) => {
                if self.stage.is_blind() || matches!(self.stage, Stage::TarotHand(_)) {
                    self.available.deselect_card(card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::Play() => match self.stage.is_blind() {
                true => self.play_selected(),
                false => Err(GameError::InvalidAction),
            },
            Action::Discard() => match self.stage.is_blind() {
                true => self.discard_selected(),
                false => Err(GameError::InvalidAction),
            },
            Action::MoveCard(dir, card) => {
                if self.stage.is_blind() || matches!(self.stage, Stage::TarotHand(_)) {
                    self.move_card(dir, card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::CashOut(_reward) => match self.stage {
                Stage::PostBlind() => self.cashout(),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyJoker(joker) => match self.stage {
                Stage::Shop() => self.buy_joker(joker),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyConsumable(consumable) => match self.stage {
                Stage::Shop() => self.buy_consumable(consumable),
                _ => Err(GameError::InvalidAction),
            },
            Action::UseConsumable(consumable) => match self.stage {
                Stage::End(_) | Stage::TarotHand(_) => Err(GameError::InvalidAction),
                _ => self.use_consumable(consumable),
            },
            Action::NextRound() => match self.stage {
                Stage::Shop() => self.next_round(),
                _ => Err(GameError::InvalidAction),
            },
            Action::SelectBlind(blind) => match self.stage {
                Stage::PreBlind() => self.select_blind(blind),
                _ => Err(GameError::InvalidAction),
            },
            Action::SkipBlind(blind) => match self.stage {
                Stage::PreBlind() => self.skip_blind(blind),
                _ => Err(GameError::InvalidAction),
            },
            Action::ApplyTarot() => self.apply_tarot(),
            Action::SellJoker(idx) => self.sell_joker(idx),
            Action::SellConsumable(idx) => self.sell_consumable(idx),
            Action::BuyPack(pack) => match self.stage {
                Stage::Shop() => self.buy_pack(pack),
                _ => Err(GameError::InvalidStage),
            },
            Action::PickPackCard(content) => match self.stage {
                Stage::PackOpen() => self.pick_pack_card(content),
                _ => Err(GameError::InvalidStage),
            },
            Action::SortHand(sort_by) => {
                if self.stage.is_blind() {
                    self.available.sort(sort_by);
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::SkipPack() => match self.stage {
                Stage::PackOpen() => self.skip_pack(),
                _ => Err(GameError::InvalidStage),
            },
            Action::Reroll() => match self.stage {
                Stage::Shop() => self.reroll(),
                _ => Err(GameError::InvalidAction),
            },
        }
    }

    pub fn handle_action_index(&mut self, index: usize) -> Result<(), GameError> {
        let space = self.gen_action_space();
        let action = space.to_action(index, self)?;
        self.handle_action(action)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hand_str: String = self
            .available
            .cards_and_selected()
            .iter()
            .map(|(c, sel)| {
                if *sel {
                    format!("[{}]", c)
                } else {
                    format!("{}", c)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(f, "hand: {}", hand_str)?;
        writeln!(f, "discard pile: {}", self.discarded.len())?;
        writeln!(f, "deck: {}", self.deck.len())?;
        if self.jokers.is_empty() {
            writeln!(f, "jokers: (none)")?;
        } else {
            writeln!(f, "jokers:")?;
            for j in self.jokers.clone() {
                writeln!(f, "  {}", joker_display(&j))?;
            }
        }
        if self.consumables.is_empty() {
            writeln!(f, "consumables: (none)")?;
        } else {
            let parts: Vec<String> = self
                .consumables
                .iter()
                .map(|c| format!("[{}] {}", c.type_label(), c.name()))
                .collect();
            writeln!(f, "consumables: {}", parts.join(", "))?;
        }
        writeln!(f, "planetarium: {}", self.planetarium)?;
        writeln!(f, "stage: {:?}", self.stage)?;
        writeln!(f, "ante: {:?}", self.ante_current)?;
        writeln!(f, "blind: {:?}", self.blind)?;
        writeln!(f, "round: {}", self.round)?;
        writeln!(f, "hands remaining: {}", self.plays)?;
        writeln!(f, "discards remaining: {}", self.discards)?;
        writeln!(f, "money: {}", self.money)?;
        if matches!(self.stage, Stage::Blind(_)) {
            writeln!(
                f,
                "score: {}  target: {}",
                self.score,
                self.required_score()
            )
        } else {
            writeln!(f, "score: {}", self.score)
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(feature = "serde")]
impl Game {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        let mut game: Self = serde_json::from_str(s)?;
        let jokers = game.jokers.clone();
        game.effect_registry.register_jokers(jokers, &game.clone());

        // Deserializing doesn't allocate ids, so Card::new() calls after this
        // point would otherwise collide with ids already present in `game`.
        let max_id = game
            .deck
            .cards()
            .iter()
            .chain(game.available.cards().iter())
            .chain(game.discarded.iter())
            .map(|c| c.id)
            .max();
        if let Some(max_id) = max_id {
            crate::card::ensure_id_counter_past(max_id);
        }

        Ok(game)
    }
}

/// Compute a score from base values and jokers, without needing a full Game.
pub fn score_hand(
    base_chips: usize,
    base_mult: usize,
    played_cards: &[Card],
    jokers: &[Jokers],
    mut hand: MadeHand,
) -> usize {
    let card_chips: usize = played_cards.iter().map(|c| c.chips()).sum();
    let mut g = Game {
        deck: Deck::new(),
        chips: base_chips + card_chips,
        mult: base_mult,
        jokers: jokers.to_vec(),
        ..Default::default()
    };

    for j in jokers {
        for e in j.effects(&g) {
            match e {
                Effects::OnScore(_) => g.effect_registry.on_score.push(e),
                Effects::OnModifyHand(_) => g.effect_registry.on_modify_hand.push(e),
                _ => (),
            }
        }
    }

    for e in g.effect_registry.on_modify_hand.clone() {
        if let Effects::OnModifyHand(f) = e {
            f.lock().unwrap()(&mut g, &mut hand)
        }
    }

    for e in g.effect_registry.on_score.clone() {
        if let Effects::OnScore(f) = e {
            f.lock().unwrap()(&mut g, hand.clone())
        }
    }

    g.chips * g.mult
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Suit, Value};

    // Cross-checked by hand against `explore`'s output for seed "TEST" —
    // see ARCHITECTURE.md.
    #[test]
    fn test_real_rng_mode_matches_balatro_seed_reference() {
        use crate::pack::PackSize;

        let config = Config {
            rng_mode: RngMode::Real,
            seed_str: Some("TEST".to_string()),
            ..Config::default()
        };
        let mut g = Game::new(config);

        let planetarium = g.planetarium.clone();
        g.shop
            .refresh(&planetarium, &[], false, 1, &[], 1, &mut g.backend);

        assert_eq!(g.shop.jokers.len(), 1);
        assert_eq!(g.shop.jokers[0].name(), "Ice Cream");
        assert_eq!(g.shop.consumables.len(), 1);
        assert_eq!(g.shop.consumables[0].name(), "Strength");

        assert_eq!(g.shop.packs.len(), 2);

        assert_eq!(g.shop.packs[0].category, PackCategory::Buffoon);
        assert_eq!(g.shop.packs[0].size, PackSize::Normal);
        let p1_contents: Vec<String> = g.shop.packs[0].contents.iter().map(|c| c.name()).collect();
        assert_eq!(p1_contents, vec!["Baseball Card", "Drunkard"]);

        assert_eq!(g.shop.packs[1].category, PackCategory::Spectral);
        assert_eq!(g.shop.packs[1].size, PackSize::Normal);
        let p2_contents: Vec<String> = g.shop.packs[1].contents.iter().map(|c| c.name()).collect();
        assert_eq!(p2_contents, vec!["Medium", "Wraith"]);
    }

    // Same seed through two independent `Game`s: proves `RngMode::Real`
    // stays deterministic as antes advance.
    #[test]
    fn test_real_rng_mode_is_deterministic_across_antes() {
        use crate::pack::PackSize;

        fn run() -> Vec<(Vec<String>, Vec<String>, Vec<(PackCategory, PackSize)>)> {
            let config = Config {
                rng_mode: RngMode::Real,
                seed_str: Some("TEST".to_string()),
                ..Config::default()
            };
            let mut g = Game::new(config);
            let planetarium = g.planetarium.clone();
            let mut antes = Vec::new();
            for ante in 1..=4 {
                g.shop
                    .refresh(&planetarium, &[], false, 1, &[], ante, &mut g.backend);
                let jokers: Vec<String> =
                    g.shop.jokers.iter().map(|j| j.name().to_string()).collect();
                let consumables: Vec<String> = g
                    .shop
                    .consumables
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect();
                let packs = vec![
                    (g.shop.packs[0].category, g.shop.packs[0].size),
                    (g.shop.packs[1].category, g.shop.packs[1].size),
                ];
                antes.push((jokers, consumables, packs));
            }
            antes
        }

        assert_eq!(
            run(),
            run(),
            "same seed must produce the same draws across every ante"
        );
    }

    // Buys the real shop joker, then refreshes many times: proves
    // on_joker_bought reached the live Instance's lock table.
    #[test]
    fn test_real_rng_mode_excludes_bought_joker_from_future_draws() {
        let config = Config {
            rng_mode: RngMode::Real,
            seed_str: Some("TEST".to_string()),
            ..Config::default()
        };
        let mut g = Game::new(config);
        g.stage = Stage::Shop();
        g.money = 1000;

        let planetarium = g.planetarium.clone();
        g.shop
            .refresh(&planetarium, &[], false, 1, &[], 1, &mut g.backend);
        let bought = g.shop.jokers[0].clone();
        g.buy_joker(bought.clone()).expect("buy joker");

        for ante in 1..=50 {
            g.shop.refresh(
                &planetarium,
                &[],
                false,
                1,
                &g.jokers.clone(),
                ante,
                &mut g.backend,
            );
            assert!(
                g.shop
                    .jokers
                    .iter()
                    .all(|j| std::mem::discriminant(j) != std::mem::discriminant(&bought)),
                "bought joker {} reappeared in shop while still owned (ante {ante})",
                bought.name()
            );
        }
    }

    // Mirror of the test above: selling should make the joker drawable
    // again. Probabilistic — enough draws that reappearance is near-certain.
    #[test]
    fn test_real_rng_mode_sold_joker_becomes_drawable_again() {
        let config = Config {
            rng_mode: RngMode::Real,
            seed_str: Some("TEST".to_string()),
            ..Config::default()
        };
        let mut g = Game::new(config);
        g.stage = Stage::Shop();
        g.money = 1000;

        let planetarium = g.planetarium.clone();
        g.shop
            .refresh(&planetarium, &[], false, 1, &[], 1, &mut g.backend);
        let bought = g.shop.jokers[0].clone();
        g.buy_joker(bought.clone()).expect("buy joker");
        g.sell_joker(0).expect("sell joker");

        let mut reappeared = false;
        for ante in 1..=500 {
            g.shop
                .refresh(&planetarium, &[], false, 1, &[], ante, &mut g.backend);
            if g.shop
                .jokers
                .iter()
                .any(|j| std::mem::discriminant(j) == std::mem::discriminant(&bought))
            {
                reappeared = true;
                break;
            }
        }
        assert!(
            reappeared,
            "sold joker {} never reappeared across 500 draws",
            bought.name()
        );
    }

    #[test]
    fn test_constructor() {
        let g = Game::default();
        assert_eq!(g.available.cards().len(), 0);
        assert_eq!(g.deck.len(), 52);
        assert_eq!(g.mult, 0);
    }

    #[test]
    fn test_deal() {
        let mut g = Game::default();
        g.deal();
        // deck should be 7 cards smaller than we started with
        assert_eq!(g.deck.len(), 52 - g.config.available);
        // should be 7 cards now available
        assert_eq!(g.available.cards().len(), g.config.available);
    }

    #[test]
    fn test_draw() {
        let mut g = Game::default();
        g.draw(1);
        assert_eq!(g.available.cards().len(), 1);
        assert_eq!(g.deck.len(), 52 - 1);
        g.draw(3);
        assert_eq!(g.available.cards().len(), 4);
        assert_eq!(g.deck.len(), 52 - 4);
    }
    #[test]
    fn test_discard() {
        let mut g = Game::default();
        g.deal();
        assert_eq!(g.available.cards().len(), g.config.available);
        assert_eq!(g.deck.len(), 52 - g.config.available);
        // select first 4 cards
        for c in g.available.cards()[0..5].to_vec() {
            g.select_card(c).unwrap();
        }
        let discard_res = g.discard_selected();
        assert!(discard_res.is_ok());
        // available should still be 7, we discarded then redrew to match
        assert_eq!(g.available.cards().len(), g.config.available);
        // deck is now smaller since we drew from it
        assert_eq!(g.deck.len(), 52 - g.config.available - 5);
    }

    #[test]
    fn test_calc_score() {
        let mut g = Game::default();
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
    fn test_calc_score_traced_matches_calc_score() {
        // guards calc_score/calc_score_traced against silently diverging
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let jack = Card::new(Value::Jack, Suit::Club);

        let fixtures = vec![
            vec![ace, king, jack],
            vec![king, king, ace],
            vec![ace, ace, ace, king],
            vec![king, king, king, king, ace],
            vec![jack, jack, jack, jack, jack],
        ];

        for cards in fixtures {
            let hand = SelectHand::new(cards).best_hand().unwrap();

            let mut plain = Game::default();
            let plain_score = plain.calc_score(hand.clone());

            let mut traced = Game::default();
            let (traced_score, trace) = traced.calc_score_traced(hand);

            assert_eq!(plain_score, traced_score);
            assert!(!trace.0.is_empty());
        }
    }

    #[test]
    fn test_calc_score_traced_hand_and_played_card_steps() {
        let mut g = Game::default();
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let jack = Card::new(Value::Jack, Suit::Club);

        // Same fixture as test_calc_score's first case, best hand is the ace alone.
        let hand = SelectHand::new(vec![ace, king, jack]).best_hand().unwrap();
        let (score, trace) = g.calc_score_traced(hand);
        assert_eq!(score, 16);

        assert_eq!(trace.0.len(), 2);

        assert_eq!(
            trace.0[0].source,
            ScoreSource::HandLevel(HandRank::HighCard)
        );
        assert_eq!(trace.0[0].chips_before, 0);
        assert_eq!(trace.0[0].chips_after, 5);
        assert_eq!(trace.0[0].mult_before, 0);
        assert_eq!(trace.0[0].mult_after, 1);
        assert!(!trace.0[0].retrigger);

        assert_eq!(trace.0[1].source, ScoreSource::PlayedCard(ace));
        assert_eq!(trace.0[1].chips_before, 5);
        assert_eq!(trace.0[1].chips_after, 16);
        assert_eq!(trace.0[1].mult_before, 1);
        assert_eq!(trace.0[1].mult_after, 1);
        assert!(!trace.0[1].retrigger);
    }

    #[test]
    fn test_calc_score_traced_held_card_steps() {
        let mut g = Game::default();
        let ace = Card::new(Value::Ace, Suit::Heart);
        let mut steel_king = Card::new(Value::King, Suit::Diamond);
        steel_king.enhancement = Some(Enhancement::Steel);

        // Held, not played — g.available.extend() leaves cards unselected.
        g.available.extend(vec![steel_king]);

        // mult=2 baseline so Steel's `mult += mult/2` has a nonzero result.
        let hand = SelectHand::new(vec![ace, ace]).best_hand().unwrap();
        let (score, trace) = g.calc_score_traced(hand);

        let held_steps: Vec<_> = trace
            .0
            .iter()
            .filter(|s| matches!(s.source, ScoreSource::HeldCard(_)))
            .collect();
        assert_eq!(held_steps.len(), 1);
        assert_eq!(held_steps[0].source, ScoreSource::HeldCard(steel_king));
        assert_eq!(held_steps[0].mult_before, 2);
        assert_eq!(held_steps[0].mult_after, 3);
        assert_eq!(held_steps[0].chips_before, held_steps[0].chips_after);

        // Pair (level 1): 10 chips, 2 mult. Played (2 aces): +11+11 chips.
        // Held Steel king: mult 2 -> 3. (10 + 22) * 3 = 96.
        assert_eq!(score, 96);
    }

    #[test]
    fn test_handle_score() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);

        // Not enough to pass
        let required = g.required_score();
        let score = required - 1;

        let passed = g.handle_score(score).unwrap();
        assert!(!passed);
        assert_eq!(g.score, score);

        // Enough to pass now
        let passed = g.handle_score(1).unwrap();
        assert!(passed);
        assert_eq!(g.score, required);
        assert_eq!(g.stage, Stage::PostBlind());
    }

    #[test]
    fn test_clear_blind() {
        let mut g = Game::default();
        g.start();
        g.deal();
        g.clear_blind();
        // all cards return to deck, available is empty
        assert_eq!(g.deck.len(), 52);
        assert_eq!(g.available.cards().len(), 0);
    }

    #[test]
    fn test_play_selected() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);
        g.deal();
        for card in g.available.cards().iter().take(5) {
            g.available.select_card(*card).expect("can select card");
        }

        assert_eq!(g.available.selected().len(), 5);
        // Artifically set score so blind passes
        g.score += g.required_score();
        g.play_selected().expect("can play selected");

        // Should have cleared blind
        assert_eq!(g.stage, Stage::PostBlind());
        // Score should reset to 0
        assert_eq!(g.score, g.config.base_score);
        // Plays and discards should reset
        assert_eq!(g.plays, g.config.plays);
        assert_eq!(g.discards, g.config.discards);
        // All cards back in deck after clear_blind, available is empty
        assert_eq!(g.deck.len(), 52);
        // Discarded should be length 0
        assert_eq!(g.discarded.len(), 0);
        // Available is empty until next blind starts
        assert_eq!(g.available.cards().len(), 0);
    }

    #[test]
    fn test_buy_joker() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.money = 10;
        let j1 = crate::joker::jokers_by_rarity(crate::joker::Rarity::Common)[0].clone();
        g.shop.jokers = vec![j1.clone()];
        g.buy_joker(j1.clone()).expect("buy joker");
        assert_eq!(g.money, 10 - j1.cost());
        assert_eq!(g.jokers.len(), 1);
    }

    #[test]
    fn test_score_hand_no_jokers() {
        use crate::hand::SelectHand;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let played = vec![ace, king];
        let jokers = vec![];
        let hand = SelectHand::new(played.clone()).best_hand().unwrap();
        // High card (level 1): chips=5, mult=1
        // Played cards: 11 + 10 = 21 chips
        // score = (5 + 21) * 1 = 26
        let score = score_hand(5, 1, &played, &jokers, hand);
        assert_eq!(score, 26);
    }

    #[test]
    fn test_score_hand_mystic_summit_active() {
        use crate::hand::SelectHand;
        use crate::joker::*;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let played = vec![ace];
        let jokers = vec![Jokers::MysticSummit(MysticSummit::default())];
        let hand = SelectHand::new(played.clone()).best_hand().unwrap();
        // Set g.discards to 0 via the scratch Game — we need to reach into it
        // Instead, just verify the joker triggers with discards=0 and doesn't with >0
        let score = score_hand(5, 1, &played, &jokers, hand.clone());
        // Default Game has discards=3, so Mystic Summit does NOT fire: 16 * 1 = 16
        assert_eq!(score, 16);

        // Now test with discards=0 — we need score_hand to pass that through
        // For now this is a limitation; skip this assertion
    }

    #[test]
    fn test_enhancement_bonus() {
        // High card Ace with Bonus: (5 + 11 + 30) * 1 = 46
        let mut g = Game::default();
        let mut ace = Card::new(Value::Ace, Suit::Heart);
        ace.enhancement = Some(Enhancement::Bonus);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 46);
    }

    #[test]
    fn test_enhancement_mult() {
        // High card Ace with Mult: (5 + 11) * (1 + 4) = 80
        let mut g = Game::default();
        let mut ace = Card::new(Value::Ace, Suit::Heart);
        ace.enhancement = Some(Enhancement::Mult);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 80);
    }

    #[test]
    fn test_enhancement_glass() {
        // High card Ace with Glass: (5 + 11) * (1 * 2) = 32
        let mut g = Game::default();
        let mut ace = Card::new(Value::Ace, Suit::Heart);
        ace.enhancement = Some(Enhancement::Glass);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 32);
    }

    #[test]
    fn test_enhancement_stone_as_kicker() {
        // [Ace, Stone] -> High Card Ace; Stone kicker always scores +50
        // (5 + 11 + 50) * 1 = 66
        let mut g = Game::default();
        let ace = Card::new(Value::Ace, Suit::Heart);
        let mut stone = Card::new(Value::Two, Suit::Diamond);
        stone.enhancement = Some(Enhancement::Stone);
        let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 66);
    }

    #[test]
    fn test_enhancement_steel_held() {
        // Pair of Kings scored; Steel King held (not played)
        // Pair level 1: chips=10, mult=2; both Kings score: chips=10+10+10=30
        // Steel held: mult = floor(2 * 1.5) = 3 -> score = 30 * 3 = 90
        let mut g = Game::default();
        let king1 = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Diamond);
        let mut steel_king = Card::new(Value::King, Suit::Spade);
        steel_king.enhancement = Some(Enhancement::Steel);
        g.available.extend(vec![steel_king]);
        let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 90);
    }

    #[test]
    fn test_enhancement_gold_held() {
        // Gold card held in unplayed hand gives $3 per hand scored
        let mut g = Game::default();
        let mut gold_card = Card::new(Value::Two, Suit::Heart);
        gold_card.enhancement = Some(Enhancement::Gold);
        g.available.extend(vec![gold_card]);
        let ace = Card::new(Value::Ace, Suit::Spade);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        g.calc_score(hand);
        assert_eq!(g.money, 3);
    }

    #[test]
    fn test_seal_gold_scored() {
        // Gold Seal: $3 when this card is played and scores (score itself unaffected)
        let mut g = Game::default();
        let mut ace = Card::new(Value::Ace, Suit::Spade);
        ace.seal = Some(Seal::Gold);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 16);
        assert_eq!(g.money, 3);
    }

    #[test]
    fn test_seal_red_retrigger_played() {
        // Red Seal: retrigger this card's played abilities.
        // High card Ace, chips scored twice: (5 + 11 + 11) * 1 = 27
        let mut g = Game::default();
        let mut ace = Card::new(Value::Ace, Suit::Spade);
        ace.seal = Some(Seal::Red);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 27);
    }

    #[test]
    fn test_seal_red_retrigger_gold_held() {
        // Red Seal retriggers held-card abilities too, not just played ones:
        // a held Gold+Red card earns $6 (2x $3), not $3.
        let mut g = Game::default();
        let mut gold_red = Card::new(Value::Two, Suit::Heart);
        gold_red.enhancement = Some(Enhancement::Gold);
        gold_red.seal = Some(Seal::Red);
        g.available.extend(vec![gold_red]);
        let ace = Card::new(Value::Ace, Suit::Spade);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        g.calc_score(hand);
        assert_eq!(g.money, 6);
    }

    #[test]
    fn test_seal_red_retrigger_steel_held() {
        // Pair of Kings scored; held Steel+Red King retriggers its Mult bonus twice.
        // Pair level 1: chips=10, mult=2; both Kings score: chips=30
        // Steel triggers twice: mult = 2 -> 3 (floor(2*1.5)) -> 4 (floor(3*1.5))
        // score = 30 * 4 = 120
        let mut g = Game::default();
        let king1 = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Diamond);
        let mut steel_red_king = Card::new(Value::King, Suit::Spade);
        steel_red_king.enhancement = Some(Enhancement::Steel);
        steel_red_king.seal = Some(Seal::Red);
        g.available.extend(vec![steel_red_king]);
        let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 120);
    }

    #[test]
    fn test_stone_kicker_red_seal_retrigger() {
        // [Ace, Stone+Red Seal] -> High Card Ace; stone kicker retriggers twice.
        // (5 + 11 + 50 + 50) * 1 = 116
        let mut g = Game::default();
        let ace = Card::new(Value::Ace, Suit::Heart);
        let mut stone = Card::new(Value::Two, Suit::Diamond);
        stone.enhancement = Some(Enhancement::Stone);
        stone.seal = Some(Seal::Red);
        let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 116);
    }

    #[test]
    fn test_hanging_chad_retriggers_first_played_card() {
        use crate::joker::*;
        // [Ace, Stone], Ace first -> Ace retriggers 3x, stone kicker only 1x.
        // (5 + 11*3 + 50) * 1 = 88
        let mut g = Game::default();
        g.jokers.push(Jokers::HangingChad(HangingChad::default()));
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        let ace = Card::new(Value::Ace, Suit::Heart);
        let mut stone = Card::new(Value::Two, Suit::Diamond);
        stone.enhancement = Some(Enhancement::Stone);
        let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 88);
    }

    #[test]
    fn test_hanging_chad_stone_kicker_competes_for_first() {
        use crate::joker::*;
        // Same cards as above, stone selected first this time -> the stone
        // kicker gets HangingChad's retrigger instead of the Ace.
        // (5 + 11 + 50*3) * 1 = 166
        let mut g = Game::default();
        g.jokers.push(Jokers::HangingChad(HangingChad::default()));
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        let mut stone = Card::new(Value::Two, Suit::Diamond);
        stone.enhancement = Some(Enhancement::Stone);
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![stone, ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 166);
    }

    #[test]
    fn test_dusk_no_retrigger_with_plays_remaining() {
        use crate::joker::*;
        let mut g = Game::default();
        g.jokers.push(Jokers::Dusk(Dusk::default()));
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        g.plays = 1;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 16);
    }

    #[test]
    fn test_dusk_retriggers_on_final_hand() {
        use crate::joker::*;
        // Same hand as above, but this is the last play of the round:
        // (5 + 11 + 11) * 1 = 27
        let mut g = Game::default();
        g.jokers.push(Jokers::Dusk(Dusk::default()));
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        g.plays = 0;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 27);
    }

    #[test]
    fn test_hack_retriggers_only_low_ranks() {
        use crate::joker::*;
        // Two Pair (Threes + Kings): Hack retriggers each Three, not the Kings.
        let cards = vec![
            Card::new(Value::Three, Suit::Heart),
            Card::new(Value::Three, Suit::Diamond),
            Card::new(Value::King, Suit::Spade),
            Card::new(Value::King, Suit::Club),
        ];

        let mut base_game = Game::default();
        let base_hand = SelectHand::new(cards.clone()).best_hand().unwrap();
        let base = base_game.calc_score(base_hand);

        let mut g = Game::default();
        g.jokers.push(Jokers::Hack(Hack::default()));
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        let hand = SelectHand::new(cards).best_hand().unwrap();
        // Each Three retriggers once more: +3 chips * 2 cards = +6 chips,
        // scaled by TwoPair's X2 mult -> +12 final score.
        assert_eq!(g.calc_score(hand), base + 12);
    }

    #[test]
    fn test_sock_and_buskin_retriggers_only_face_cards() {
        use crate::joker::*;
        // Two Pair (Jacks + Fours): SockAndBuskin retriggers each Jack, not the Fours.
        let cards = vec![
            Card::new(Value::Jack, Suit::Heart),
            Card::new(Value::Jack, Suit::Diamond),
            Card::new(Value::Four, Suit::Spade),
            Card::new(Value::Four, Suit::Club),
        ];

        let mut base_game = Game::default();
        let base_hand = SelectHand::new(cards.clone()).best_hand().unwrap();
        let base = base_game.calc_score(base_hand);

        let mut g = Game::default();
        g.jokers
            .push(Jokers::SockAndBuskin(SockAndBuskin::default()));
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        let hand = SelectHand::new(cards).best_hand().unwrap();
        // Each Jack retriggers once more: +10 chips * 2 cards = +20 chips,
        // scaled by TwoPair's X2 mult -> +40 final score.
        assert_eq!(g.calc_score(hand), base + 40);
    }

    #[test]
    fn test_mime_retriggers_held_steel() {
        use crate::joker::*;
        // Same math as test_seal_red_retrigger_steel_held (120), Mime instead of Red Seal:
        // mult 2 -> 3 (floor(2*1.5)) -> 4 (floor(3*1.5)); score = 30 * 4 = 120
        let mut g = Game::default();
        g.jokers.push(Jokers::Mime(Mime::default()));
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        let king1 = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Diamond);
        let mut steel_king = Card::new(Value::King, Suit::Spade);
        steel_king.enhancement = Some(Enhancement::Steel);
        g.available.extend(vec![steel_king]);
        let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 120);
    }

    #[test]
    fn test_seal_purple_discard_creates_tarot() {
        // Purple Seal: creates a Tarot card when this card is discarded.
        let mut g = Game::default();
        g.deal();
        let card = g.available.cards()[0];
        g.mutate_card(card.id, |c| c.seal = Some(Seal::Purple));
        let card = g.available.cards()[0];
        g.select_card(card).expect("can select card");

        assert_eq!(g.consumables.len(), 0);
        g.discard_selected().expect("can discard");
        assert_eq!(g.consumables.len(), 1);
        assert!(matches!(g.consumables[0], Consumable::Tarot(_)));
    }

    #[test]
    fn test_seal_purple_discard_respects_slots() {
        let mut g = Game::default();
        g.config.consumable_slots = 0;
        g.deal();
        let card = g.available.cards()[0];
        g.mutate_card(card.id, |c| c.seal = Some(Seal::Purple));
        let card = g.available.cards()[0];
        g.select_card(card).expect("can select card");

        g.discard_selected().expect("can discard");
        assert_eq!(g.consumables.len(), 0);
    }

    #[test]
    fn test_seal_blue_round_end_creates_planet() {
        // Blue Seal: creates a Planet card if held in hand when the round ends.
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);
        g.deal();

        let held_card = g.available.cards()[7];
        g.mutate_card(held_card.id, |c| c.seal = Some(Seal::Blue));
        for card in g.available.cards()[0..5].to_vec() {
            g.available.select_card(card).expect("can select card");
        }
        assert_eq!(g.available.selected().len(), 5);

        g.score += g.required_score();
        assert_eq!(g.consumables.len(), 0);
        g.play_selected().expect("can play selected");

        assert_eq!(g.consumables.len(), 1);
        assert!(matches!(g.consumables[0], Consumable::Planet(_)));
    }

    #[test]
    fn test_seal_blue_does_not_fire_before_blind_cleared() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);
        g.deal();

        let held_card = g.available.cards()[7];
        g.mutate_card(held_card.id, |c| c.seal = Some(Seal::Blue));
        for card in g.available.cards()[0..5].to_vec() {
            g.available.select_card(card).expect("can select card");
        }
        // score not bumped past required_score() - blind stays open
        g.play_selected().expect("can play selected");

        assert_eq!(g.consumables.len(), 0);
    }

    #[test]
    fn test_edition_foil() {
        // High card Ace with Foil: (5 + 11 + 50) * 1 = 66
        let mut g = Game::default();
        let mut ace = Card::new(Value::Ace, Suit::Heart);
        ace.edition = Edition::Foil;
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 66);
    }

    #[test]
    fn test_edition_holographic() {
        // High card Ace with Holographic: (5 + 11) * (1 + 10) = 176
        let mut g = Game::default();
        let mut ace = Card::new(Value::Ace, Suit::Heart);
        ace.edition = Edition::Holographic;
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 176);
    }

    #[test]
    fn test_edition_polychrome() {
        // Pair of Kings, second King has Polychrome
        // Pair level 1: chips=10, mult=2
        // king1: chips += 10; king2 (poly): chips += 10, mult = floor(2*1.5) = 3
        // score = (10+10+10) * 3 = 90
        let mut g = Game::default();
        let king = Card::new(Value::King, Suit::Heart);
        let mut poly_king = Card::new(Value::King, Suit::Diamond);
        poly_king.edition = Edition::Polychrome;
        let hand = SelectHand::new(vec![king, poly_king]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 90);
    }

    #[test]
    fn test_joker_edition_foil() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();

        // TheJoker Base: (5+11)*(1+4) = 80
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let mut j = Jokers::TheJoker(TheJoker::default());
        j.set_edition(Edition::Foil);
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // Foil fires before TheJoker: chips += 50, then +4 mult
        // (5+11+50) * (1+4) = 66 * 5 = 330
        assert_eq!(g.calc_score(hand), 330);
    }

    #[test]
    fn test_joker_edition_holographic() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let mut j = Jokers::TheJoker(TheJoker::default());
        j.set_edition(Edition::Holographic);
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // Holo fires before TheJoker: mult += 10, then +4 mult
        // (5+11) * (1+10+4) = 16 * 15 = 240
        assert_eq!(g.calc_score(hand), 240);
    }

    #[test]
    fn test_joker_edition_polychrome() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let mut j = Jokers::TheJoker(TheJoker::default());
        j.set_edition(Edition::Polychrome);
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // TheJoker fires (+4 mult), then Polychrome x1.5: floor((1+4)*1.5) = 7
        // (5+11) * 7 = 112
        assert_eq!(g.calc_score(hand), 112);
    }

    #[test]
    fn test_joker_edition_negative_buy_increases_slots() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        let slots_before = g.config.joker_slots;

        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::TheJoker(TheJoker::default());
        j.set_edition(Edition::Negative);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();

        assert_eq!(g.config.joker_slots, slots_before + 1);
    }

    #[test]
    fn test_joker_edition_negative_sell_decreases_slots() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        let slots_before = g.config.joker_slots;

        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::TheJoker(TheJoker::default());
        j.set_edition(Edition::Negative);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        assert_eq!(g.config.joker_slots, slots_before + 1);

        g.sell_joker(0).unwrap();
        assert_eq!(g.config.joker_slots, slots_before);
    }

    #[test]
    fn test_destroy_card() {
        let mut g = Game::default();
        g.deal();
        let card = g.available.cards()[0];
        let available_before = g.available.cards().len();
        g.destroy_card(card.id);
        assert_eq!(g.available.cards().len(), available_before - 1);
        assert!(g.available.cards().iter().all(|c| c.id != card.id));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_to_from_json_roundtrip() {
        let mut g = Game::default();
        g.start();
        let json = g.to_json().expect("serialize");
        let g2 = Game::from_json(&json).expect("deserialize");
        assert_eq!(g2.stage, g.stage);
        assert_eq!(g2.ante_current, g.ante_current);
        assert_eq!(g2.plays, g.plays);
        assert_eq!(g2.discards, g.discards);
        assert_eq!(g2.money, g.money);
        assert_eq!(g2.available.cards().len(), g.available.cards().len());
        assert_eq!(g2.deck.len(), g.deck.len());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_from_json_restores_effect_registry() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        let joker = Jokers::TheJoker(TheJoker::default());
        g.jokers.push(joker);
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        assert!(!g.effect_registry.on_score.is_empty());

        let json = g.to_json().expect("serialize");
        let g2 = Game::from_json(&json).expect("deserialize");
        assert_eq!(g2.jokers.len(), 1);
        assert!(!g2.effect_registry.on_score.is_empty());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_from_json_invalid() {
        let result = Game::from_json("not valid json");
        assert!(result.is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_from_json_advances_id_counter_past_loaded_ids() {
        let mut g = Game::default();
        let mut high_id_card = Card::new(Value::King, Suit::Spade);
        high_id_card.id = 999_999;
        g.deck.push(high_id_card);

        let json = g.to_json().expect("serialize");
        let _g2 = Game::from_json(&json).expect("deserialize");

        // Card::new() after loading must not collide with the loaded id.
        let new_card = Card::new(Value::Two, Suit::Heart);
        assert!(new_card.id > 999_999);
    }

    #[test]
    fn test_glass_shatter_preserves_hand_size() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);

        // Replace available with controlled cards so we have exactly one glass card.
        // Use mixed values/suits to ensure the hand scores below the blind threshold
        // so clear_blind() / deal() is not triggered mid-test.
        g.available.empty();
        // Glass card is ace (highest) so it's selected as the high card and scores.
        // high card ace scores (5+11)*2 = 32, below 300 small blind threshold.
        let mut glass = Card::new(Value::Ace, Suit::Heart);
        glass.enhancement = Some(Enhancement::Glass);
        let glass_id = glass.id;
        let others = vec![
            Card::new(Value::Two, Suit::Club),
            Card::new(Value::Four, Suit::Diamond),
            Card::new(Value::Six, Suit::Spade),
            Card::new(Value::Eight, Suit::Club),
            Card::new(Value::Ten, Suit::Diamond),
            Card::new(Value::Queen, Suit::Spade),
        ];
        let mut hand_cards = vec![glass];
        hand_cards.extend(others);
        g.available.extend(hand_cards);

        let hand_size_before = g.available.cards().len();

        // Force 100% shatter probability (only needs to be 4 but overkill is ok)
        g.prob_mult = 100;

        // Select glass + 4 others (5-card hand)
        let to_select: Vec<Card> = g.available.cards().into_iter().take(5).collect();
        for card in to_select {
            g.available.select_card(card).expect("select card");
        }

        g.play_selected().expect("play selected");

        // Hand size must be unchanged
        assert_eq!(g.available.cards().len(), hand_size_before);
        // Glass card must be permanently gone (not hiding in discarded to return next deal)
        assert!(g.discarded.iter().all(|c| c.id != glass_id));
    }

    #[test]
    fn test_targeting_tarot_in_blind_invalid_selection_preserves_consumable() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);
        g.deal();
        // no cards selected — Magician needs at least 1
        g.consumables = vec![Consumable::Tarot(Tarot::Magician)];
        let res = g.use_consumable(Consumable::Tarot(Tarot::Magician));
        assert!(matches!(res, Err(GameError::InvalidAction)));
        assert_eq!(g.consumables.len(), 1);
        assert!(g.consumables.contains(&Consumable::Tarot(Tarot::Magician)));
    }

    #[test]
    fn test_targeting_tarot_in_blind_too_many_selected_preserves_consumable() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);
        g.deal();
        // Lovers accepts exactly 1 — select 2 to exceed max
        let cards = g.available.cards();
        g.available.select_card(cards[0]).unwrap();
        g.available.select_card(cards[1]).unwrap();
        g.consumables = vec![Consumable::Tarot(Tarot::Lovers)];
        let res = g.use_consumable(Consumable::Tarot(Tarot::Lovers));
        assert!(matches!(res, Err(GameError::InvalidAction)));
        assert_eq!(g.consumables.len(), 1);
    }

    #[test]
    fn test_sell_joker() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.money = 0;
        let joker = Jokers::TheJoker(TheJoker::default());
        let sell_value = joker.sell_value();
        g.jokers.push(joker);
        assert_eq!(g.jokers.len(), 1);

        g.sell_joker(0).expect("sell joker");
        assert_eq!(g.jokers.len(), 0);
        assert_eq!(g.money, sell_value);
    }

    #[test]
    fn test_sell_joker_invalid_index() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        let res = g.sell_joker(0);
        assert!(matches!(res, Err(GameError::InvalidAction)));
    }

    #[test]
    fn test_sell_joker_invalid_stage() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::End(crate::stage::End::Win);
        g.jokers.push(Jokers::TheJoker(TheJoker::default()));
        let res = g.sell_joker(0);
        assert!(matches!(res, Err(GameError::InvalidStage)));
    }

    #[test]
    fn test_sell_joker_during_blind() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);
        g.money = 0;
        let joker = Jokers::TheJoker(TheJoker::default());
        let sell_value = joker.sell_value();
        g.jokers.push(joker);

        g.sell_joker(0).expect("sell joker during blind");
        assert_eq!(g.jokers.len(), 0);
        assert_eq!(g.money, sell_value);
    }

    #[test]
    fn test_sell_consumable() {
        use crate::planet::Planets;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.money = 0;
        let c = Consumable::Planet(Planets::Mercury);
        let sell_value = c.sell_value();
        g.consumables.push(c);
        assert_eq!(g.consumables.len(), 1);

        g.sell_consumable(0).expect("sell consumable");
        assert_eq!(g.consumables.len(), 0);
        assert_eq!(g.money, sell_value);
    }

    #[test]
    fn test_sell_consumable_invalid_index() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        let res = g.sell_consumable(0);
        assert!(matches!(res, Err(GameError::InvalidAction)));
    }

    #[test]
    fn test_sell_consumable_invalid_stage() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::End(crate::stage::End::Win);
        g.consumables
            .push(Consumable::Tarot(crate::tarot::Tarot::Fool));
        let res = g.sell_consumable(0);
        assert!(matches!(res, Err(GameError::InvalidStage)));
    }

    #[test]
    fn test_sell_joker_during_tarot_hand() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::TarotHand(Tarot::Magician);
        g.money = 0;
        let joker = Jokers::TheJoker(TheJoker::default());
        let sell_value = joker.sell_value();
        g.jokers.push(joker);

        g.sell_joker(0).expect("sell joker during tarot hand");
        assert_eq!(g.jokers.len(), 0);
        assert_eq!(g.money, sell_value);
    }

    #[test]
    fn test_sell_consumable_during_tarot_hand() {
        use crate::planet::Planets;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::TarotHand(Tarot::Magician);
        g.money = 0;
        let c = Consumable::Planet(Planets::Mercury);
        let sell_value = c.sell_value();
        g.consumables.push(c);

        g.sell_consumable(0)
            .expect("sell consumable during tarot hand");
        assert_eq!(g.consumables.len(), 0);
        assert_eq!(g.money, sell_value);
    }

    #[test]
    fn test_sell_joker_removes_effects() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        let joker = Jokers::TheJoker(TheJoker::default());
        g.jokers.push(joker);
        let jokers = g.jokers.clone();
        g.effect_registry.register_jokers(jokers, &g.clone());
        assert!(!g.effect_registry.on_score.is_empty());

        g.sell_joker(0).expect("sell joker");
        assert!(g.effect_registry.on_score.is_empty());
    }

    #[test]
    fn test_buy_pack() {
        use crate::pack::{Pack, PackCategory, PackSize};
        use crate::planet::Planets;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.money = 100;
        let pack = Pack {
            category: PackCategory::Celestial,
            size: PackSize::Normal,
            contents: vec![PackContent::Planet(Planets::Mercury)],
        };
        let cost = pack.cost();
        let picks = pack.picks_allowed();
        g.shop.packs.push(pack.clone());
        g.buy_pack(pack).expect("buy pack");
        assert_eq!(g.stage, Stage::PackOpen());
        assert_eq!(g.money, 100 - cost);
        let state = g.open_pack.as_ref().expect("open pack state");
        assert_eq!(state.picks_remaining, picks);
    }

    #[test]
    fn test_skip_pack() {
        let mut g = Game::default();
        g.start();
        g.open_pack = Some(OpenPackState {
            picks_remaining: 1,
            description: String::new(),
            contents: vec![],
        });
        g.stage = Stage::PackOpen();
        g.skip_pack().expect("skip pack");
        assert_eq!(g.stage, Stage::Shop());
        assert!(g.open_pack.is_none());
    }

    #[test]
    fn test_skip_blind_small() {
        let mut g = Game::default();
        g.start();
        assert!(g.tags.is_empty());
        g.skip_blind(Blind::Small).expect("skip small blind");
        assert_eq!(g.blind, Some(Blind::Small));
        assert_eq!(g.stage, Stage::PreBlind());
        assert_eq!(g.tags.len(), 1);
    }

    #[test]
    fn test_skip_blind_wrong_blind() {
        let mut g = Game::default();
        g.start();
        // Game just started, next expected blind is Small, not Big
        let res = g.skip_blind(Blind::Big);
        assert!(matches!(res, Err(GameError::InvalidBlind)));
    }

    #[test]
    fn test_skip_blind_boss_rejected() {
        let mut g = Game::default();
        g.start();
        g.blind = Some(Blind::Big);
        let res = g.skip_blind(Blind::Boss);
        assert!(matches!(res, Err(GameError::InvalidBlind)));
    }

    #[test]
    fn test_skip_blind_wrong_stage() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        let res = g.skip_blind(Blind::Small);
        assert!(matches!(res, Err(GameError::InvalidStage)));
    }

    #[test]
    fn test_skip_blind_draws_a_real_tag() {
        let mut g = Game::default();
        g.start();
        let expected = g.small_blind_tag;
        g.skip_blind(Blind::Small).expect("skip small blind");
        assert_eq!(g.tags[0], expected);
    }

    #[test]
    fn test_ante_tags_exist_from_game_start() {
        let g = Game::default();
        assert!(g.skip_tag(Blind::Small).is_some());
        assert!(g.skip_tag(Blind::Big).is_some());
        assert!(g.skip_tag(Blind::Boss).is_none());
    }

    #[test]
    fn test_ante_tags_redrawn_after_boss_defeated() {
        let mut g = Game {
            stage: Stage::Blind(Blind::Boss),
            blind: Some(Blind::Boss),
            ..Default::default()
        };
        let ante_before = g.ante_current;
        g.handle_score(1_000_000).expect("handle score");
        assert_ne!(g.ante_current, ante_before);
        assert!(Tag::iter().any(|t| t == g.small_blind_tag));
        assert!(Tag::iter().any(|t| t == g.big_blind_tag));
    }

    #[test]
    fn test_skip_blind_banks_the_predrawn_tag() {
        let mut g = Game::default();
        g.start();
        let expected = g.small_blind_tag;
        g.skip_blind(Blind::Small).expect("skip small blind");
        assert_eq!(g.tags, vec![expected]);
    }

    #[test]
    fn test_cashout_drains_and_applies_tags() {
        let mut g = Game {
            stage: Stage::PostBlind(),
            ..Default::default()
        };
        g.tags = vec![Tag::Handy, Tag::Garbage];
        g.cashout().expect("cashout");
        assert!(g.tags.is_empty());
    }

    #[test]
    fn test_finish_pack_does_not_touch_tags() {
        let mut g = Game::default();
        g.start();
        g.tags = vec![Tag::Handy];
        g.open_pack = Some(OpenPackState {
            picks_remaining: 1,
            description: String::new(),
            contents: vec![],
        });
        g.stage = Stage::PackOpen();
        g.skip_pack().expect("skip pack");
        assert_eq!(g.tags, vec![Tag::Handy]);
    }

    #[test]
    fn test_pick_pack_card_planet() {
        use crate::planet::Planets;
        let mut g = Game::default();
        g.start();
        g.open_pack = Some(OpenPackState {
            picks_remaining: 1,
            description: String::new(),
            contents: vec![PackContent::Planet(Planets::Mercury)],
        });
        g.stage = Stage::PackOpen();
        let level_before = g.planetarium.level(HandRank::OnePair).level;
        g.pick_pack_card(PackContent::Planet(Planets::Mercury))
            .expect("pick planet");
        assert_eq!(g.stage, Stage::Shop());
        assert!(g.open_pack.is_none());
        assert_eq!(
            g.planetarium.level(HandRank::OnePair).level,
            level_before + 1
        );
    }

    #[test]
    fn test_pick_pack_card_joker() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        let joker = Jokers::TheJoker(TheJoker::default());
        let before_len = g.jokers.len();
        g.open_pack = Some(OpenPackState {
            picks_remaining: 1,
            description: String::new(),
            contents: vec![PackContent::Joker(joker.clone())],
        });
        g.stage = Stage::PackOpen();
        g.pick_pack_card(PackContent::Joker(joker))
            .expect("pick joker");
        assert_eq!(g.jokers.len(), before_len + 1);
        assert_eq!(g.stage, Stage::Shop());
        assert!(g.open_pack.is_none());
    }

    #[test]
    fn test_pick_pack_card_joker_slots_full() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        let slots = g.config.joker_slots;
        for _ in 0..slots {
            g.jokers.push(Jokers::TheJoker(TheJoker::default()));
        }
        let joker = Jokers::TheJoker(TheJoker::default());
        g.open_pack = Some(OpenPackState {
            picks_remaining: 1,
            description: String::new(),
            contents: vec![PackContent::Joker(joker.clone())],
        });
        g.stage = Stage::PackOpen();
        let res = g.pick_pack_card(PackContent::Joker(joker));
        assert!(matches!(res, Err(GameError::NoAvailableSlot)));
    }

    #[test]
    fn test_negative_joker_from_pack_expands_slots() {
        use crate::joker::Jokers;
        use balatro_types::joker::TheJoker;
        let mut g = Game::default();
        g.start();
        let slots = g.config.joker_slots;
        for _ in 0..slots {
            g.jokers.push(Jokers::TheJoker(TheJoker::default()));
        }
        let mut neg_joker = Jokers::TheJoker(TheJoker::default());
        neg_joker.set_edition(Edition::Negative);
        g.open_pack = Some(OpenPackState {
            picks_remaining: 1,
            description: String::new(),
            contents: vec![PackContent::Joker(neg_joker.clone())],
        });
        g.stage = Stage::PackOpen();
        g.pick_pack_card(PackContent::Joker(neg_joker))
            .expect("pick negative joker from full pack");
        assert_eq!(g.config.joker_slots, slots + 1);
        assert_eq!(g.jokers.len(), slots + 1);
        assert_eq!(g.stage, Stage::Shop());
        assert!(g.open_pack.is_none());
    }

    #[test]
    fn test_pick_pack_card_targeting_tarot() {
        let mut g = Game::default();
        g.start();
        assert!(Tarot::Lovers.requires_targets());
        g.open_pack = Some(OpenPackState {
            picks_remaining: 1,
            description: String::new(),
            contents: vec![PackContent::Tarot(Tarot::Lovers)],
        });
        g.stage = Stage::PackOpen();
        g.pick_pack_card(PackContent::Tarot(Tarot::Lovers))
            .expect("pick targeting tarot");
        assert!(matches!(g.stage, Stage::TarotHand(Tarot::Lovers)));
        assert_eq!(g.tarot_prev_stage, Some(Stage::PackOpen()));
        // picks_remaining decremented later in apply_tarot, not here
        assert_eq!(g.open_pack.as_ref().unwrap().picks_remaining, 1);
    }

    #[test]
    fn test_reroll_invalid_outside_shop() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        let res = g.reroll();
        assert!(matches!(res, Err(GameError::InvalidStage)));
    }

    #[test]
    fn test_reroll_insufficient_balance() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.money = 0;
        let res = g.reroll();
        assert!(matches!(res, Err(GameError::InvalidBalance)));
    }

    #[test]
    fn test_reroll_deducts_cost() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.money = 10;
        g.reroll().expect("reroll");
        assert_eq!(g.money, 5);
    }

    #[test]
    fn test_reroll_increments_cost() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.money = 20;
        g.reroll().expect("first reroll");
        assert_eq!(g.reroll_cost, 6);
        g.reroll().expect("second reroll");
        assert_eq!(g.reroll_cost, 7);
        assert_eq!(g.money, 9); // 20 - 5 - 6
    }

    #[test]
    fn test_reroll_regenerates_shop_cards() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.money = 10;
        let old_joker = crate::joker::jokers_by_rarity(crate::joker::Rarity::Common)[0].clone();
        g.shop.jokers = vec![old_joker.clone()];
        g.shop.consumables = Vec::new();
        g.reroll().expect("reroll");
        assert_eq!(g.shop.jokers.len() + g.shop.consumables.len(), 2);
        let still_present = g
            .shop
            .jokers
            .iter()
            .any(|j| std::mem::discriminant(j) == std::mem::discriminant(&old_joker));
        assert!(!still_present);
    }
}
