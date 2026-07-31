use indexmap::IndexMap;
use itertools::Itertools;
#[cfg(feature = "python")]
use pyo3::pyclass;
use std::fmt;

use crate::card::Card;
use crate::card::Enhancement;
use crate::card::Suit;
use crate::card::Value;
use crate::error::PlayHandError;
use crate::rank::HandRank;

// Hand, SelectHand and MadeHand are all representations of a collection of Card,
// just at different phases in the cycle of selecting, executing and scoring cards.
// Hand represents all drawn cards, cards available for action (play/discard).
// SelectHand represents (up to 5) cards user selects from hand for action.
// MadeHand represents actual poker hand level and associated cards from a selected hand.

// Hand represents all drawn cards, cards available for action (play/discard)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Hand(Vec<Card>);

// MadeHand represents actual poker hand level and associated cards from a selected hand.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MadeHand {
    pub hand: SelectHand,
    pub rank: HandRank,
    pub all: Vec<Card>,
}

// SelectHand represents (up to 5) cards user selects from hand for action
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SelectHand(Vec<Card>);

impl SelectHand {
    pub fn new(cards: Vec<Card>) -> Self {
        Self(cards)
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    // Get all values in a hand. Sorted lowest to highest.
    fn values(&self) -> Vec<Value> {
        self.0.iter().map(|x| x.value).sorted().collect()
    }
    pub(crate) fn cards(&self) -> Vec<Card> {
        self.0.clone()
    }

    // Get map of each value with corresponding cards.
    // For example, Ks, Ah, Jh, Jc, Jd -> {A: [Ah], K: [Ks], J: [Jh, Jc: Jd]}
    fn values_freq(&self) -> IndexMap<Value, Vec<Card>> {
        let mut counts: IndexMap<Value, Vec<Card>> = IndexMap::new();
        for card in &self.0 {
            counts.entry(card.value).or_default().push(*card);
        }
        // Sorted by value, descending. Sorted in place rather than
        // re-collected — `best_hand` calls this once per rank check, so it
        // is the hottest allocation site in the engine.
        counts.sort_by(|a, _, b, _| Ord::cmp(b, a));
        counts
    }

    // Get map of each suit with corresponding cards.
    // For example, Ks, Ah, Jh, Jc, Jd -> {h: [Jh, Ah], s: [Ks], c: [Jc], d: [Jd]}
    #[allow(dead_code)]
    pub(crate) fn suits_freq(&self) -> IndexMap<Suit, Vec<Card>> {
        let mut counts: IndexMap<Suit, Vec<Card>> = IndexMap::new();
        for card in &self.0 {
            counts.entry(card.suit).or_default().push(*card);
        }
        // Sorted by suit, descending. See `values_freq` on the in-place sort.
        counts.sort_by(|a, _, b, _| Ord::cmp(b, a));
        counts
    }

    // Original relative order, values_freq()'s groups are order-preserving
    // individually, but concatenating two of them (two pair, full house)
    // isn't, without this.
    fn cards_of_values(&self, values: &[Value]) -> Vec<Card> {
        self.0
            .iter()
            .filter(|c| values.contains(&c.value))
            .cloned()
            .collect()
    }

    // A played hand is at most 5 cards, so per-value and per-suit tallies
    // fit in fixed buffers. The `is_*` methods build an
    // `IndexMap<_, Vec<Card>>` instead — fine for one-off calls, but
    // `best_hand` runs on every scored hand, and the allocations dominated
    // it. These three helpers are the allocation-free equivalents;
    // `test_best_hand_matches_the_original_is_chain_exhaustively` holds
    // them to the `is_*` methods' exact output.
    const MAX_PLAYED: usize = 5;

    /// Distinct values with their counts, highest value first — the same
    /// order `values_freq` yields. Only valid for a playable hand.
    fn value_counts(&self) -> ([(Value, u8); Self::MAX_PLAYED], usize) {
        debug_assert!(self.len() <= Self::MAX_PLAYED);
        let mut counts = [(Value::Two, 0u8); Self::MAX_PLAYED];
        let mut len = 0;
        for card in &self.0 {
            match counts[..len].iter_mut().find(|(v, _)| *v == card.value) {
                Some(slot) => slot.1 += 1,
                None => {
                    counts[len] = (card.value, 1);
                    len += 1;
                }
            }
        }
        counts[..len].sort_unstable_by(|a, b| Ord::cmp(&b.0, &a.0));
        (counts, len)
    }

    /// Card values sorted ascending, as `values()` returns them.
    fn sorted_values(&self) -> ([Value; Self::MAX_PLAYED], usize) {
        debug_assert!(self.len() <= Self::MAX_PLAYED);
        let mut values = [Value::Two; Self::MAX_PLAYED];
        let len = self.0.len().min(Self::MAX_PLAYED);
        for (slot, card) in values[..len].iter_mut().zip(&self.0) {
            *slot = card.value;
        }
        values[..len].sort_unstable();
        (values, len)
    }

    /// The flush cards, matching `is_flush`'s composition exactly: the
    /// first suit (in first-appearance order) holding enough non-wild
    /// cards, followed by every wild.
    fn flush_cards(&self) -> Option<Vec<Card>> {
        if self.len() < 5 {
            return None;
        }
        let is_wild = |c: &Card| c.enhancement == Some(Enhancement::Wild);
        let wild_count = self.0.iter().filter(|c| is_wild(c)).count();
        let needed = 5usize.saturating_sub(wild_count);
        if needed == 0 {
            return Some(self.0.clone());
        }

        let mut suits = [(Suit::Spade, 0u8); Self::MAX_PLAYED];
        let mut len = 0;
        for card in self.0.iter().filter(|c| !is_wild(c)) {
            match suits[..len].iter_mut().find(|(s, _)| *s == card.suit) {
                Some(slot) => slot.1 += 1,
                None => {
                    suits[len] = (card.suit, 1);
                    len += 1;
                }
            }
        }
        let (suit, _) = *suits[..len]
            .iter()
            .find(|(_, count)| *count as usize >= needed)?;

        let mut cards: Vec<Card> = self
            .0
            .iter()
            .filter(|c| c.suit == suit && !is_wild(c))
            .copied()
            .collect();
        cards.extend(self.0.iter().filter(|c| is_wild(c)).copied());
        Some(cards)
    }

    /// Whether the hand runs consecutively, including the low-ace wheel.
    /// Mirrors `is_straight`, which is only ever a straight at exactly 5.
    fn has_straight(values: &[Value]) -> bool {
        if values.len() != 5 {
            return false;
        }
        let consecutive = |vs: &[Value]| vs.windows(2).all(|v| (v[1] as u16 - v[0] as u16) == 1);
        if consecutive(values) {
            return true;
        }
        // A, 2, 3, 4, 5 sorts as (2, 3, 4, 5, A) with the ace high.
        values[4] == Value::Ace && values[0] == Value::Two && consecutive(&values[..4])
    }

    /// Can play any number of cards, it is our responsibility
    /// to determine the best hand. Higher tier hands take precedence
    /// over lower tier hands regardless of their level or scoring.
    /// For example, if hand is Kd Kd Kd Kd 2d, best hand will be a
    // Four of a Kind and never a Flush.
    //
    // Hand ranking:
    // FlushFive
    // FlushHouse
    // FiveOfAKind
    // RoyalFlush
    // StraightFlush
    // FourOfAKind
    // FullHouse
    // Flush
    // Straight
    // ThreeOfAKind
    // TwoPair
    // OnePair
    // HighCard
    pub fn best_hand(&self) -> Result<MadeHand, PlayHandError> {
        if self.0.is_empty() {
            return Err(PlayHandError::NoCards);
        }
        if self.len() > 5 {
            return Err(PlayHandError::TooManyCards);
        }

        // Each ingredient is derived exactly once and shared across the
        // rank checks below. The equivalent `is_*` chain re-derived them
        // per check — `is_flush_five` calls `is_flush`, so does
        // `is_flush_house`, `is_straight_flush` and (through it)
        // `is_royal_flush` — which cost ~12 `values_freq` rebuilds and 5
        // flush scans for a single hand. The `is_*` methods are kept as-is
        // for direct callers and tests; this is the hot path.
        let len = self.len();
        let (counts, n_values) = self.value_counts();
        let counts = &counts[..n_values];
        // Sorted by value descending, so the first group meeting a count is
        // also the highest-valued one — the same card the `is_*` methods'
        // `.find()` lands on.
        let n_of_kind = |n: u8| {
            counts
                .iter()
                .find(|(_, count)| *count >= n)
                .map(|(value, _)| *value)
        };
        let n_of_kind_other = |n: u8, exclude: Value| {
            counts
                .iter()
                .find(|(value, count)| *value != exclude && *count >= n)
                .map(|(value, _)| *value)
        };

        let flush = self.flush_cards();
        let (values, n_cards) = self.sorted_values();
        let values = &values[..n_cards];
        let straight = Self::has_straight(values);

        let five = (len >= 5).then(|| n_of_kind(5)).flatten();
        let four = (len >= 4).then(|| n_of_kind(4)).flatten();
        let three = (len >= 3).then(|| n_of_kind(3)).flatten();
        let pair = (len >= 2).then(|| n_of_kind(2)).flatten();

        // A full house needs a *different* value for its pair than its
        // triple; two pair likewise. Anchored separately because the
        // triple and the highest pair need not be the same value.
        let fullhouse = if len >= 5 {
            three.and_then(|t| n_of_kind_other(2, t).map(|p| (t, p)))
        } else {
            None
        };
        let two_pair = if len >= 4 {
            pair.and_then(|first| n_of_kind_other(2, first).map(|second| (first, second)))
        } else {
            None
        };

        let straight_flush = flush.is_some() && straight;
        let royal = straight_flush
            && values
                == [
                    Value::Ten,
                    Value::Jack,
                    Value::Queen,
                    Value::King,
                    Value::Ace,
                ];

        // Highest tier wins regardless of level or scoring, so the order
        // here is load-bearing — it mirrors the ranking comment above.
        let (hand, rank) = if flush.is_some() && five.is_some() {
            (self.clone(), HandRank::FlushFive)
        } else if flush.is_some() && fullhouse.is_some() {
            (self.clone(), HandRank::FlushHouse)
        } else if let Some(value) = five {
            (
                SelectHand::new(self.cards_of_values(&[value])),
                HandRank::FiveOfAKind,
            )
        } else if royal {
            (self.clone(), HandRank::RoyalFlush)
        } else if straight_flush {
            (self.clone(), HandRank::StraightFlush)
        } else if let Some(value) = four {
            (
                SelectHand::new(self.cards_of_values(&[value])),
                HandRank::FourOfAKind,
            )
        } else if let Some((three_val, two_val)) = fullhouse {
            (
                SelectHand::new(self.cards_of_values(&[three_val, two_val])),
                HandRank::FullHouse,
            )
        } else if let Some(flush) = flush {
            (SelectHand::new(flush), HandRank::Flush)
        } else if straight {
            (self.clone(), HandRank::Straight)
        } else if let Some(value) = three {
            (
                SelectHand::new(self.cards_of_values(&[value])),
                HandRank::ThreeOfAKind,
            )
        } else if let Some((first, second)) = two_pair {
            (
                SelectHand::new(self.cards_of_values(&[first, second])),
                HandRank::TwoPair,
            )
        } else if let Some(value) = pair {
            (
                SelectHand::new(self.cards_of_values(&[value])),
                HandRank::OnePair,
            )
        } else if let Some((value, _)) = counts.first() {
            (
                SelectHand::new(self.cards_of_values(&[*value])),
                HandRank::HighCard,
            )
        } else {
            return Err(PlayHandError::UnknownHand);
        };

        Ok(MadeHand {
            hand,
            rank,
            all: self.cards(),
        })
    }

    // Only `best_hand` and the joker predicates evaluate hands in
    // anger, and `best_hand` no longer routes through this method.
    // Kept because the differential test uses these as the oracle the
    // optimized `best_hand` is checked against.
    #[allow(dead_code)]
    pub(crate) fn is_highcard(&self) -> Option<SelectHand> {
        if self.len() < 1 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| !val.is_empty())?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    pub(crate) fn is_pair(&self) -> Option<SelectHand> {
        if self.len() < 2 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 2)?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    pub(crate) fn is_two_pair(&self) -> Option<SelectHand> {
        if self.len() < 4 {
            return None;
        }

        let (first_val, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 2)?;

        let (second_val, _) = self
            .values_freq()
            .into_iter()
            .find(|(key, val)| *key != first_val && val.len() >= 2)?;

        Some(SelectHand::new(
            self.cards_of_values(&[first_val, second_val]),
        ))
    }

    pub(crate) fn is_three_of_kind(&self) -> Option<SelectHand> {
        if self.len() < 3 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 3)?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    pub(crate) fn is_straight(&self) -> Option<SelectHand> {
        if self.len() != 5 {
            return None;
        }
        // Iterate our sorted values. Each value must be one more than the previous.
        let values = self.values();
        if values.windows(2).all(|v| (v[1] as u16 - v[0] as u16) == 1) {
            return Some(self.clone());
        }

        // Special case for low ace.
        // Values are sorted with Ace as high (2, 3, 4, 5, A)
        // Therefore, we can check that last value is ace, first value is two.
        // Then remove the last value (ace) from vec and check for incremental values
        // for everything else (2, 3, 4, 5).
        if values[4] == Value::Ace && values[0] == Value::Two {
            let skip_last: Vec<Value> = values.into_iter().rev().skip(1).rev().collect();
            if skip_last
                .windows(2)
                .all(|v| (v[1] as u16 - v[0] as u16) == 1)
            {
                return Some(self.clone());
            }
        }
        None
    }

    pub(crate) fn is_flush(&self) -> Option<SelectHand> {
        if self.len() < 5 {
            return None;
        }
        // Each wild card in hand reduces the number of a suit needed to make a flush
        // since a wild can be any suit.
        let wilds: Vec<Card> = self
            .0
            .iter()
            .filter(|c| c.enhancement == Some(Enhancement::Wild))
            .cloned()
            .collect();
        let wild_count = wilds.len();
        let needed = 5usize.saturating_sub(wild_count);

        if needed == 0 {
            return Some(self.clone());
        }

        let mut suit_groups: IndexMap<Suit, Vec<Card>> = IndexMap::new();
        for card in self
            .0
            .iter()
            .filter(|c| c.enhancement != Some(Enhancement::Wild))
        {
            suit_groups.entry(card.suit).or_default().push(*card);
        }

        if let Some((_, mut cards)) = suit_groups.into_iter().find(|(_, v)| v.len() >= needed) {
            cards.extend(wilds);
            return Some(SelectHand::new(cards));
        }
        None
    }

    // Only `best_hand` and the joker predicates evaluate hands in
    // anger, and `best_hand` no longer routes through this method.
    // Kept because the differential test uses these as the oracle the
    // optimized `best_hand` is checked against.
    #[allow(dead_code)]
    pub(crate) fn is_fullhouse(&self) -> Option<SelectHand> {
        if self.len() < 5 {
            return None;
        }

        let (three_val, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 3)?;

        let (two_val, _) = self
            .values_freq()
            .into_iter()
            .find(|(key, val)| *key != three_val && val.len() >= 2)?;

        Some(SelectHand::new(self.cards_of_values(&[three_val, two_val])))
    }

    pub(crate) fn is_four_of_kind(&self) -> Option<SelectHand> {
        if self.len() < 4 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 4)?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    // Only `best_hand` and the joker predicates evaluate hands in
    // anger, and `best_hand` no longer routes through this method.
    // Kept because the differential test uses these as the oracle the
    // optimized `best_hand` is checked against.
    #[allow(dead_code)]
    pub(crate) fn is_straight_flush(&self) -> Option<SelectHand> {
        if self.is_flush().is_some() && self.is_straight().is_some() {
            return Some(self.clone());
        }
        None
    }

    // Only `best_hand` and the joker predicates evaluate hands in
    // anger, and `best_hand` no longer routes through this method.
    // Kept because the differential test uses these as the oracle the
    // optimized `best_hand` is checked against.
    #[allow(dead_code)]
    pub(crate) fn is_royal_flush(&self) -> Option<SelectHand> {
        if self.is_straight_flush().is_some()
            && self.values().into_iter().eq(vec![
                Value::Ten,
                Value::Jack,
                Value::Queen,
                Value::King,
                Value::Ace,
            ])
        {
            return Some(self.clone());
        }
        None
    }

    // Only `best_hand` and the joker predicates evaluate hands in
    // anger, and `best_hand` no longer routes through this method.
    // Kept because the differential test uses these as the oracle the
    // optimized `best_hand` is checked against.
    #[allow(dead_code)]
    pub(crate) fn is_five_of_kind(&self) -> Option<SelectHand> {
        if self.len() < 5 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 5)?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    // Only `best_hand` and the joker predicates evaluate hands in
    // anger, and `best_hand` no longer routes through this method.
    // Kept because the differential test uses these as the oracle the
    // optimized `best_hand` is checked against.
    #[allow(dead_code)]
    pub(crate) fn is_flush_house(&self) -> Option<SelectHand> {
        if self.is_flush().is_some() && self.is_fullhouse().is_some() {
            return Some(self.clone());
        }
        None
    }

    // Only `best_hand` and the joker predicates evaluate hands in
    // anger, and `best_hand` no longer routes through this method.
    // Kept because the differential test uses these as the oracle the
    // optimized `best_hand` is checked against.
    #[allow(dead_code)]
    pub(crate) fn is_flush_five(&self) -> Option<SelectHand> {
        if self.is_flush().is_some() && self.is_five_of_kind().is_some() {
            return Some(self.clone());
        }
        None
    }
}

impl Default for SelectHand {
    fn default() -> Self {
        let cards: Vec<Card> = Vec::new();
        Self(cards)
    }
}

impl fmt::Display for SelectHand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for card in &self.0 {
            write!(f, "{}", card)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::Itertools;

    /// The original `best_hand` implementation, expressed as the `is_*`
    /// chain it used to be. `best_hand` was rewritten to derive each
    /// ingredient once instead of re-deriving it per check; this is the
    /// oracle that rewrite is held to.
    fn reference_best_hand(h: &SelectHand) -> Result<MadeHand, PlayHandError> {
        if h.0.is_empty() {
            return Err(PlayHandError::NoCards);
        }
        if h.len() > 5 {
            return Err(PlayHandError::TooManyCards);
        }
        let (hand, rank) = if let Some(hand) = h.is_flush_five() {
            (hand, HandRank::FlushFive)
        } else if let Some(hand) = h.is_flush_house() {
            (hand, HandRank::FlushHouse)
        } else if let Some(hand) = h.is_five_of_kind() {
            (hand, HandRank::FiveOfAKind)
        } else if let Some(hand) = h.is_royal_flush() {
            (hand, HandRank::RoyalFlush)
        } else if let Some(hand) = h.is_straight_flush() {
            (hand, HandRank::StraightFlush)
        } else if let Some(hand) = h.is_four_of_kind() {
            (hand, HandRank::FourOfAKind)
        } else if let Some(hand) = h.is_fullhouse() {
            (hand, HandRank::FullHouse)
        } else if let Some(hand) = h.is_flush() {
            (hand, HandRank::Flush)
        } else if let Some(hand) = h.is_straight() {
            (hand, HandRank::Straight)
        } else if let Some(hand) = h.is_three_of_kind() {
            (hand, HandRank::ThreeOfAKind)
        } else if let Some(hand) = h.is_two_pair() {
            (hand, HandRank::TwoPair)
        } else if let Some(hand) = h.is_pair() {
            (hand, HandRank::OnePair)
        } else if let Some(hand) = h.is_highcard() {
            (hand, HandRank::HighCard)
        } else {
            return Err(PlayHandError::UnknownHand);
        };
        Ok(MadeHand {
            hand,
            rank,
            all: h.cards(),
        })
    }

    /// Small enough to enumerate exhaustively, wide enough to reach every
    /// rank: straights (including the low-ace case), royal values,
    /// repeated values for pairs through five-of-a-kind, a heart-heavy
    /// skew so same-suit full houses and five-of-a-kinds are reachable,
    /// and a Wild for the wildcard-flush path.
    fn probe_pool() -> Vec<Card> {
        let mut cards = Vec::new();
        // Full heart run: flushes, straight flushes, the royal.
        for value in [
            Value::Two,
            Value::Three,
            Value::Four,
            Value::Five,
            Value::Ten,
            Value::Jack,
            Value::Queen,
            Value::King,
            Value::Ace,
        ] {
            cards.push(Card::new(value, Suit::Heart));
        }
        // Off-suit partners: mixed straights and pairs.
        for value in [
            Value::Two,
            Value::Three,
            Value::Four,
            Value::Five,
            Value::Ace,
        ] {
            cards.push(Card::new(value, Suit::Spade));
        }
        // Extra hearts so KKK QQ (flush house) and KKKK+wild (flush five)
        // are both reachable.
        for _ in 0..3 {
            cards.push(Card::new(Value::King, Suit::Heart));
        }
        cards.push(Card::new(Value::Queen, Suit::Heart));
        // Off-suit kings for plain four/five-of-a-kind.
        cards.push(Card::new(Value::King, Suit::Spade));
        cards.push(Card::new(Value::King, Suit::Club));
        let mut wild = Card::new(Value::King, Suit::Diamond);
        wild.enhancement = Some(Enhancement::Wild);
        cards.push(wild);
        cards
    }

    #[test]
    fn test_best_hand_matches_the_original_is_chain_exhaustively() {
        let pool = probe_pool();
        let mut checked = 0usize;
        let mut ranks_seen = std::collections::HashSet::new();

        for size in 1..=5 {
            for combo in pool.iter().copied().combinations(size) {
                let hand = SelectHand::new(combo);
                let got = hand.best_hand();
                let want = reference_best_hand(&hand);
                match (&got, &want) {
                    (Ok(a), Ok(b)) => {
                        assert_eq!(a.rank, b.rank, "rank mismatch for {hand}");
                        assert_eq!(
                            a.hand.cards(),
                            b.hand.cards(),
                            "scoring cards mismatch for {hand} ({:?})",
                            a.rank
                        );
                        assert_eq!(a.all, b.all, "played cards mismatch for {hand}");
                        ranks_seen.insert(a.rank);
                    }
                    (Err(_), Err(_)) => {}
                    _ => panic!("disagreement on {hand}: {got:?} vs {want:?}"),
                }
                checked += 1;
            }
        }

        assert!(checked > 5_000, "only checked {checked} hands");
        // A pool that never reaches the high tiers would make the above
        // vacuous, so assert the interesting ranks were actually exercised.
        for rank in [
            HandRank::HighCard,
            HandRank::OnePair,
            HandRank::TwoPair,
            HandRank::ThreeOfAKind,
            HandRank::Straight,
            HandRank::Flush,
            HandRank::FullHouse,
            HandRank::FourOfAKind,
            HandRank::StraightFlush,
            HandRank::RoyalFlush,
            HandRank::FiveOfAKind,
            HandRank::FlushHouse,
            HandRank::FlushFive,
        ] {
            assert!(ranks_seen.contains(&rank), "{rank:?} never produced");
        }
    }

    /// The exhaustive pool above is hand-picked to reach every rank; this
    /// covers the rest of the space — every suit, every value, and the
    /// enhancements that change hand detection — by sampling real decks.
    #[test]
    fn test_best_hand_matches_the_original_on_random_full_decks() {
        use rand::seq::SliceRandom;
        use rand::{Rng, SeedableRng};
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0xB357_11A9D);

        let mut deck: Vec<Card> = Vec::new();
        for value in [
            Value::Two,
            Value::Three,
            Value::Four,
            Value::Five,
            Value::Six,
            Value::Seven,
            Value::Eight,
            Value::Nine,
            Value::Ten,
            Value::Jack,
            Value::Queen,
            Value::King,
            Value::Ace,
        ] {
            for suit in [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond] {
                deck.push(Card::new(value, suit));
            }
        }

        for _ in 0..50_000 {
            let size = rng.gen_range(1..=5);
            let mut cards: Vec<Card> = deck.choose_multiple(&mut rng, size).copied().collect();
            // Wild and Stone both alter detection, so mix them in.
            for card in cards.iter_mut() {
                match rng.gen_range(0..10) {
                    0 => card.enhancement = Some(Enhancement::Wild),
                    1 => card.enhancement = Some(Enhancement::Stone),
                    _ => {}
                }
            }

            let hand = SelectHand::new(cards);
            match (hand.best_hand(), reference_best_hand(&hand)) {
                (Ok(a), Ok(b)) => {
                    assert_eq!(a.rank, b.rank, "rank mismatch for {hand}");
                    assert_eq!(a.hand.cards(), b.hand.cards(), "cards mismatch for {hand}");
                    assert_eq!(a.all, b.all, "played cards mismatch for {hand}");
                }
                (Err(_), Err(_)) => {}
                (got, want) => panic!("disagreement on {hand}: {got:?} vs {want:?}"),
            }
        }
    }

    #[test]
    fn test_values() {
        let c3 = Card::new(Value::Two, Suit::Heart);
        let c4 = Card::new(Value::Three, Suit::Diamond);
        let c5 = Card::new(Value::Jack, Suit::Heart);
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::Ace, Suit::Spade);

        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let values = hand.values();

        // Should have 5 values
        assert_eq!(values.len(), 5);

        // Expect sorted (2, 3, J, K, A)
        assert_eq!(values[0], Value::Two);
        assert_eq!(values[1], Value::Three);
        assert_eq!(values[2], Value::Jack);
        assert_eq!(values[3], Value::King);
        assert_eq!(values[4], Value::Ace);
    }

    #[test]
    fn test_values_freq() {
        let c1 = Card::new(Value::Two, Suit::Heart);
        let c2 = Card::new(Value::Three, Suit::Diamond);
        let c3 = Card::new(Value::Four, Suit::Heart);
        let c4 = Card::new(Value::King, Suit::Heart);
        let c5 = Card::new(Value::King, Suit::Spade);

        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let freq = hand.values_freq();

        // Should have 4 values (K, 2, 3, 4)
        assert_eq!(freq.len(), 4);

        // Expect 2 kings and 1 each of 2, 3, 4
        assert_eq!(freq.get(&Value::King).unwrap().len(), 2);
        assert_eq!(freq.get(&Value::Two).unwrap().len(), 1);
        assert_eq!(freq.get(&Value::Three).unwrap().len(), 1);
        assert_eq!(freq.get(&Value::Four).unwrap().len(), 1);

        // No extra cards
        assert_eq!(freq.get(&Value::Five), None);
        assert_eq!(freq.get(&Value::Nine), None);

        // Can also check the cards in the vec are as expected
        assert_eq!(freq.get(&Value::King).unwrap()[0].value, Value::King);
        assert_eq!(freq.get(&Value::King).unwrap()[1].value, Value::King);
        assert_eq!(freq.get(&Value::Two).unwrap()[0].value, Value::Two);
        assert_eq!(freq.get(&Value::Three).unwrap()[0].value, Value::Three);
        assert_eq!(freq.get(&Value::Four).unwrap()[0].value, Value::Four);

        // Check ordered by value
        assert_eq!(freq.into_iter().next().unwrap().0, Value::King)
    }

    #[test]
    fn test_suits_freq() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Spade);
        let c3 = Card::new(Value::Two, Suit::Heart);
        let c4 = Card::new(Value::Three, Suit::Diamond);
        let c5 = Card::new(Value::Four, Suit::Heart);

        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let freq = hand.suits_freq();

        // Should have 3 values (heart, spade, diamond)
        assert_eq!(freq.len(), 3);

        // Expect 3 hearts and 1 each of spade and diamond
        assert_eq!(freq.get(&Suit::Heart).unwrap().len(), 3);
        assert_eq!(freq.get(&Suit::Spade).unwrap().len(), 1);
        assert_eq!(freq.get(&Suit::Diamond).unwrap().len(), 1);

        // No clubs to be found
        assert_eq!(freq.get(&Suit::Club), None);

        // Can also check the cards in the vec are as expected
        assert_eq!(freq.get(&Suit::Heart).unwrap()[0].suit, Suit::Heart);
        assert_eq!(freq.get(&Suit::Heart).unwrap()[1].suit, Suit::Heart);
        assert_eq!(freq.get(&Suit::Heart).unwrap()[2].suit, Suit::Heart);
        assert_eq!(freq.get(&Suit::Spade).unwrap()[0].suit, Suit::Spade);
        assert_eq!(freq.get(&Suit::Diamond).unwrap()[0].suit, Suit::Diamond);
    }

    #[test]
    fn test_best_hand() {
        let c1 = Card::new(Value::Ace, Suit::Heart);
        let c2 = Card::new(Value::Two, Suit::Heart);
        let c3 = Card::new(Value::Three, Suit::Diamond);

        // Best hand is flush five (Ah, Ah, Ah, Ah, Ah)
        let hand = SelectHand::new(vec![c1, c1, c1, c1, c1]);
        let best = hand.best_hand().expect("is best hand");
        assert_eq!(best.rank, HandRank::FlushFive);
        assert_eq!(best.hand.len(), 5);

        // 4ok is better than flush (Ah, Ah, Ah, Ah, 2h)
        let hand = SelectHand::new(vec![c1, c1, c1, c1, c2]);
        let best = hand.best_hand().expect("is best hand");
        assert_eq!(best.clone().rank, HandRank::FourOfAKind);
        assert_eq!(best.hand.len(), 4);

        // Two pair is better than pair (Ah, Ah, 2h, 2h, 3d)
        let hand = SelectHand::new(vec![c1, c1, c2, c2, c3]);
        let best = hand.best_hand().expect("is best hand");
        assert_eq!(best.clone().rank, HandRank::TwoPair);
        assert_eq!(best.hand.len(), 4);

        // At worst, we get a high card (Ah, 2h, 3d)
        let hand = SelectHand::new(vec![c1, c2, c3]);
        let best = hand.best_hand().expect("is best hand");
        assert_eq!(best.clone().rank, HandRank::HighCard);
        assert_eq!(best.hand.len(), 1);
    }

    #[test]
    fn test_highcard() {
        let c1 = Card::new(Value::Ace, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Heart);
        let c3 = Card::new(Value::Three, Suit::Diamond);
        let c4 = Card::new(Value::Four, Suit::Diamond);
        let c5 = Card::new(Value::Five, Suit::Diamond);
        let c6 = Card::new(Value::Six, Suit::Diamond);

        // Valid 5 (A, K, 3, 4, 5)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let hc = hand.is_highcard();
        assert_eq!(hc.clone().unwrap().len(), 1);
        assert_eq!(hc.unwrap().0[0].value, Value::Ace);

        // Valid 5 (K, A, 3, 4, 5)
        let hand = SelectHand::new(vec![c2, c1, c3, c4, c5]);
        let hc = hand.is_highcard();
        assert_eq!(hc.clone().unwrap().len(), 1);
        assert_eq!(hc.unwrap().0[0].value, Value::Ace);

        // Valid 5 (K, 3, 4, 5, 6)
        let hand = SelectHand::new(vec![c2, c3, c4, c5, c6]);
        let hc = hand.is_highcard();
        assert_eq!(hc.clone().unwrap().len(), 1);
        assert_eq!(hc.unwrap().0[0].value, Value::King);

        // Valid 4 (K, 3, 4, 5)
        let hand = SelectHand::new(vec![c2, c3, c4, c5]);
        let hc = hand.is_highcard();
        assert_eq!(hc.clone().unwrap().len(), 1);
        assert_eq!(hc.unwrap().0[0].value, Value::King);

        // Valid 3 (K, 3, 4)
        let hand = SelectHand::new(vec![c2, c3, c4]);
        let hc = hand.is_highcard();
        assert_eq!(hc.clone().unwrap().len(), 1);
        assert_eq!(hc.unwrap().0[0].value, Value::King);

        // Valid 2 (K, 3)
        let hand = SelectHand::new(vec![c2, c3]);
        let hc = hand.is_highcard();
        assert_eq!(hc.clone().unwrap().len(), 1);
        assert_eq!(hc.unwrap().0[0].value, Value::King);

        // Valid 1 (K)
        let hand = SelectHand::new(vec![c2]);
        let hc = hand.is_highcard();
        assert_eq!(hc.clone().unwrap().len(), 1);
        assert_eq!(hc.unwrap().0[0].value, Value::King);
    }

    #[test]
    fn test_highcard_preserves_order() {
        let ah = Card::new(Value::Ace, Suit::Heart);
        let as_ = Card::new(Value::Ace, Suit::Spade);
        let two = Card::new(Value::Two, Suit::Diamond);

        // Two aces tie for high card — order should follow input order.
        let hand = SelectHand::new(vec![ah, two, as_]);
        let hc = hand.is_highcard().unwrap();
        assert_eq!(hc.cards(), vec![ah, as_]);
    }

    #[test]
    fn test_pair() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Diamond);
        let c3 = Card::new(Value::Three, Suit::Diamond);
        let c4 = Card::new(Value::Four, Suit::Diamond);
        let c5 = Card::new(Value::Five, Suit::Diamond);
        let c6 = Card::new(Value::Six, Suit::Diamond);

        // Valid 5 (K, K, 3, 4, 5)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2.unwrap().len(), 2);

        // Valid 4 (K, K, 3, 4)
        let hand = SelectHand::new(vec![c1, c2, c3, c4]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2.unwrap().len(), 2);

        // Valid 3 (K, K, 3)
        let hand = SelectHand::new(vec![c1, c2, c3]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2.unwrap().len(), 2);

        // Valid 2 (K, K)
        let hand = SelectHand::new(vec![c1, c2]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2.unwrap().len(), 2);

        // Invalid 1 (K)
        let hand = SelectHand::new(vec![c1]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2, None);

        // Invalid 2 (K, 3)
        let hand = SelectHand::new(vec![c1, c3]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2, None);

        // Invalid 3 (K, 3, 4)
        let hand = SelectHand::new(vec![c1, c3, c4]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2, None);

        // Invalid 4 (K, 3, 4, 5)
        let hand = SelectHand::new(vec![c1, c3, c4, c5]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2, None);

        // Invalid 5 (K, 3, 4, 5, 6)
        let hand = SelectHand::new(vec![c1, c3, c4, c5, c6]);
        let is_2 = hand.is_pair();
        assert_eq!(is_2, None);
    }

    #[test]
    fn test_pair_preserves_order() {
        let kh = Card::new(Value::King, Suit::Heart);
        let ks = Card::new(Value::King, Suit::Spade);
        let ah = Card::new(Value::Ace, Suit::Heart);
        let two = Card::new(Value::Two, Suit::Diamond);

        // Kings arranged after the ace kicker, result follows input order.
        let hand = SelectHand::new(vec![ah, kh, two, ks]);
        let pair = hand.is_pair().unwrap();
        assert_eq!(pair.cards(), vec![kh, ks]);
    }

    #[test]
    fn test_two_pair() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Spade);
        let c3 = Card::new(Value::Four, Suit::Diamond);
        let c4 = Card::new(Value::Four, Suit::Heart);
        let not1 = Card::new(Value::Two, Suit::Heart);
        let not2 = Card::new(Value::Three, Suit::Heart);

        // Valid 5 (K, K, 4, 4, 2)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not1]);
        let tp = hand.is_two_pair();
        assert_eq!(tp.unwrap().len(), 4);

        // Valid 4 (K, K, 4, 4)
        let hand = SelectHand::new(vec![c1, c2, c3, c4]);
        let tp = hand.is_two_pair();
        assert_eq!(tp.unwrap().len(), 4);

        // Invalid 5 (K, K, K, K, 2)
        let hand = SelectHand::new(vec![c1, c1, c2, c2, not1]);
        let tp = hand.is_two_pair();
        assert_eq!(tp, None);

        // Invalid 5 (K, 4, 3, 2, 2)
        let hand = SelectHand::new(vec![c1, c4, not1, not2, not2]);
        let tp = hand.is_two_pair();
        assert_eq!(tp, None);

        // Invalid 5 (K, K, 4, 3, 2)
        let hand = SelectHand::new(vec![c1, c1, c4, not1, not2]);
        let tp = hand.is_two_pair();
        assert_eq!(tp, None);

        // Invalid 4 (K, K, 4, 2)
        let hand = SelectHand::new(vec![c1, c2, c4, not1]);
        let tp = hand.is_two_pair();
        assert_eq!(tp, None);
    }

    #[test]
    fn test_two_pair_preserves_order() {
        let two_h = Card::new(Value::Two, Suit::Heart);
        let two_s = Card::new(Value::Two, Suit::Spade);
        let kh = Card::new(Value::King, Suit::Heart);
        let ks = Card::new(Value::King, Suit::Spade);

        // Old code always returned the higher-valued pair first regardless
        // of arrangement, this pins the fix.
        let hand = SelectHand::new(vec![two_h, two_s, kh, ks]);
        let tp = hand.is_two_pair().unwrap();
        assert_eq!(tp.cards(), vec![two_h, two_s, kh, ks]);

        // Interleaved arrangement should come out interleaved.
        let hand = SelectHand::new(vec![two_h, kh, two_s, ks]);
        let tp = hand.is_two_pair().unwrap();
        assert_eq!(tp.cards(), vec![two_h, kh, two_s, ks]);
    }

    #[test]
    fn test_three_of_kind() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Spade);
        let c3 = Card::new(Value::King, Suit::Heart);
        let not1 = Card::new(Value::Ace, Suit::Heart);
        let not2 = Card::new(Value::Two, Suit::Heart);

        // Valid 5 (K, K, K, A, 2)
        let hand = SelectHand::new(vec![c1, c2, c3, not1, not2]);
        let is_3 = hand.is_three_of_kind();
        assert_eq!(is_3.unwrap().len(), 3);

        // Valid 4 (K, K, K, A)
        let hand = SelectHand::new(vec![c1, c2, c3, not1]);
        let is_3 = hand.is_three_of_kind();
        assert_eq!(is_3.unwrap().len(), 3);

        // Valid 3 (K, K, K)
        let hand = SelectHand::new(vec![c1, c2, c3]);
        let is_3 = hand.is_three_of_kind();
        assert_eq!(is_3.unwrap().len(), 3);

        // Invalid 3 (K, K, A)
        let hand = SelectHand::new(vec![c1, c2, not1]);
        let is_3 = hand.is_three_of_kind();
        assert_eq!(is_3, None);

        // Invalid 4 (K, K, A, A),
        let hand = SelectHand::new(vec![c1, c2, not1, not1]);
        let is_3 = hand.is_three_of_kind();
        assert_eq!(is_3, None);

        // Invalid 5 (K, K, A, A, 2),
        let hand = SelectHand::new(vec![c1, c2, not1, not1, not2]);
        let is_3 = hand.is_three_of_kind();
        assert_eq!(is_3, None);

        // Invalid 2 (K, K)
        let hand = SelectHand::new(vec![c1, c2]);
        let is_3 = hand.is_three_of_kind();
        assert_eq!(is_3, None);
    }

    #[test]
    fn test_three_of_kind_preserves_order() {
        let kh = Card::new(Value::King, Suit::Heart);
        let ks = Card::new(Value::King, Suit::Spade);
        let kd = Card::new(Value::King, Suit::Diamond);
        let ah = Card::new(Value::Ace, Suit::Heart);

        // Kings not adjacent in the input.
        let hand = SelectHand::new(vec![kh, ah, ks, kd]);
        let is_3 = hand.is_three_of_kind().unwrap();
        assert_eq!(is_3.cards(), vec![kh, ks, kd]);
    }

    #[test]
    fn test_straight() {
        let c1 = Card::new(Value::Ace, Suit::Heart);
        let c2 = Card::new(Value::Two, Suit::Heart);
        let c3 = Card::new(Value::Three, Suit::Heart);
        let c4 = Card::new(Value::Four, Suit::Heart);
        let c5 = Card::new(Value::Five, Suit::Heart);
        let c6 = Card::new(Value::Six, Suit::Diamond);
        let c7 = Card::new(Value::Seven, Suit::Diamond);

        // Valid 5 (2, 3, 4 ,5 ,6)
        let hand = SelectHand::new(vec![c2, c3, c4, c5, c6]);
        let straight = hand.is_straight();
        assert_eq!(straight.unwrap().len(), 5);

        // Valid 5 with low ace (A, 2, 3, 4 ,5)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let straight = hand.is_straight();
        assert_eq!(straight.unwrap().len(), 5);

        // Invalid 5 (2, 3, 4, 5, 7)
        let hand = SelectHand::new(vec![c2, c3, c4, c5, c7]);
        let straight = hand.is_straight();
        assert_eq!(straight, None);

        // Invalid 5 with low ace (A, 2, 3, 4, 7)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c7]);
        let straight = hand.is_straight();
        assert_eq!(straight, None);

        // Invalid 4 (2, 3, 4, 5)
        let hand = SelectHand::new(vec![c2, c3, c4, c5]);
        let straight = hand.is_straight();
        assert_eq!(straight, None);
    }

    #[test]
    fn test_flush() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::Queen, Suit::Heart);
        let c3 = Card::new(Value::Jack, Suit::Heart);
        let c4 = Card::new(Value::Seven, Suit::Heart);
        let c5 = Card::new(Value::Eight, Suit::Heart);
        let not = Card::new(Value::Ace, Suit::Diamond);

        // Valid 5 (h, h, h, h, h)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let flush = hand.is_flush();
        assert_eq!(flush.unwrap().len(), 5);

        // Valid 5 from 7 cards (h, h, h, h, h, d, d)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5, not, not]);
        let flush = hand.is_flush();
        assert_eq!(flush.unwrap().len(), 5);

        // Invalid 5 (h, h, h, h, d)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not]);
        let flush = hand.is_flush();
        assert_eq!(flush, None);

        // Invalid 4 (h, h, h, h)
        let hand = SelectHand::new(vec![c1, c2, c3, c4]);
        let flush = hand.is_flush();
        assert_eq!(flush, None);
    }

    #[test]
    fn test_fullhouse() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Spade);
        let c3 = Card::new(Value::King, Suit::Heart);
        let c4 = Card::new(Value::Four, Suit::Diamond);
        let c5 = Card::new(Value::Four, Suit::Heart);
        let not1 = Card::new(Value::Two, Suit::Heart);
        let not2 = Card::new(Value::Three, Suit::Heart);

        // Valid 5 (K, K, K, 4, 4)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let is_fh = hand.is_fullhouse();
        assert_eq!(is_fh.unwrap().len(), 5);

        // Valid 5 from 7 cards (K, K, K, 4, 4, 2, 3)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5, not1, not2]);
        let is_fh = hand.is_fullhouse();
        assert_eq!(is_fh.unwrap().len(), 5);

        // Invalid 5 (K, K, K, K, 2)
        let hand = SelectHand::new(vec![c1, c2, c3, c3, not1]);
        let is_fh = hand.is_fullhouse();
        assert_eq!(is_fh, None);

        // Invalid 5 (K, K, 4, 4, 2)
        let hand = SelectHand::new(vec![c1, c2, c4, c5, not1]);
        let is_fh = hand.is_fullhouse();
        assert_eq!(is_fh, None);

        // Invalid 4 (K, K, 4, 4)
        let hand = SelectHand::new(vec![c1, c2, c4, c5]);
        let is_fh = hand.is_fullhouse();
        assert_eq!(is_fh, None);
    }

    #[test]
    fn test_fullhouse_preserves_order() {
        let two_h = Card::new(Value::Two, Suit::Heart);
        let two_s = Card::new(Value::Two, Suit::Spade);
        let kh = Card::new(Value::King, Suit::Heart);
        let ks = Card::new(Value::King, Suit::Spade);
        let kd = Card::new(Value::King, Suit::Diamond);

        // Pair arranged before the trips, old code always returned the
        // three-of-a-kind group first regardless of arrangement.
        let hand = SelectHand::new(vec![two_h, two_s, kh, ks, kd]);
        let is_fh = hand.is_fullhouse().unwrap();
        assert_eq!(is_fh.cards(), vec![two_h, two_s, kh, ks, kd]);
    }

    #[test]
    fn test_four_of_kind() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Spade);
        let c3 = Card::new(Value::King, Suit::Heart);
        let c4 = Card::new(Value::King, Suit::Diamond);
        let not = Card::new(Value::Ace, Suit::Heart);

        // Valid 4 (K, K, K, K)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not]);
        let is_4 = hand.is_four_of_kind();
        assert_eq!(is_4.unwrap().len(), 4);

        // Valid 4 from 7 cards (K, K, K, K, A, A, A)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not, not, not]);
        let is_4 = hand.is_four_of_kind();
        assert_eq!(is_4.unwrap().len(), 4);

        // Invalid 4 (K, K, K, A)
        let hand = SelectHand::new(vec![c1, c2, c3, not]);
        let is_4 = hand.is_four_of_kind();
        assert_eq!(is_4, None);

        // Invalid 3 (K, K, K)
        let hand = SelectHand::new(vec![c1, c2, c3]);
        let is_4 = hand.is_four_of_kind();
        assert_eq!(is_4, None);
    }

    #[test]
    fn test_four_of_kind_preserves_order() {
        let kh = Card::new(Value::King, Suit::Heart);
        let ks = Card::new(Value::King, Suit::Spade);
        let kd = Card::new(Value::King, Suit::Diamond);
        let kc = Card::new(Value::King, Suit::Club);
        let ah = Card::new(Value::Ace, Suit::Heart);

        let hand = SelectHand::new(vec![kh, ah, ks, kd, kc]);
        let is_4 = hand.is_four_of_kind().unwrap();
        assert_eq!(is_4.cards(), vec![kh, ks, kd, kc]);
    }

    #[test]
    fn test_straight_flush() {
        let c1 = Card::new(Value::Ace, Suit::Heart);
        let c2 = Card::new(Value::Two, Suit::Heart);
        let c3 = Card::new(Value::Three, Suit::Heart);
        let c4 = Card::new(Value::Four, Suit::Heart);
        let c5 = Card::new(Value::Five, Suit::Heart);
        let c6 = Card::new(Value::Six, Suit::Heart);
        let not1 = Card::new(Value::Seven, Suit::Heart);
        let not2 = Card::new(Value::Six, Suit::Diamond);

        // Valid 5 (2h, 3h, 4h, 5h ,6h)
        let hand = SelectHand::new(vec![c2, c3, c4, c5, c6]);
        let sf = hand.is_straight_flush();
        assert_eq!(sf.unwrap().len(), 5);

        // Valid 5 with low ace (Ah, 2h, 3h, 4h, 5h)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let sf = hand.is_straight_flush();
        assert_eq!(sf.unwrap().len(), 5);

        // Invalid 5, wrong value (2h, 3h, 4h, 5h, 7h)
        let hand = SelectHand::new(vec![c2, c3, c4, c5, not1]);
        let sf = hand.is_straight_flush();
        assert_eq!(sf, None);

        // Invalid 5, wrong suit (2h, 3h, 4h, 5h, 6d)
        let hand = SelectHand::new(vec![c2, c3, c4, c5, not2]);
        let sf = hand.is_straight_flush();
        assert_eq!(sf, None);

        // Invalid 4 (2h, 3h, 4h, 5h)
        let hand = SelectHand::new(vec![c2, c3, c4, c5]);
        let sf = hand.is_straight_flush();
        assert_eq!(sf, None);
    }

    #[test]
    fn test_royal_flush() {
        let c1 = Card::new(Value::Ten, Suit::Spade);
        let c2 = Card::new(Value::Jack, Suit::Spade);
        let c3 = Card::new(Value::Queen, Suit::Spade);
        let c4 = Card::new(Value::King, Suit::Spade);
        let c5 = Card::new(Value::Ace, Suit::Spade);
        let not1 = Card::new(Value::Nine, Suit::Spade);
        let not2 = Card::new(Value::Ace, Suit::Diamond);

        // Valid 5 (10s, Js, Qs, Ks, As)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let rf = hand.is_royal_flush();
        assert_eq!(rf.unwrap().len(), 5);

        // Valid 5, scrambled input order (Js, 10s, Ks, Qs, As)
        let hand = SelectHand::new(vec![c2, c1, c4, c3, c5]);
        let rf = hand.is_royal_flush();
        assert_eq!(rf.unwrap().len(), 5);

        // Invalid 5, wrong value (9s, Js, Qs, Ks, As)
        let hand = SelectHand::new(vec![not1, c2, c3, c4, c5]);
        let rf = hand.is_royal_flush();
        assert_eq!(rf, None);

        // Invalid 5, wrong suit (10s, Js, Qs, Ks, Ad)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not2]);
        let rf = hand.is_royal_flush();
        assert_eq!(rf, None);

        // Invalid 4 (2h, 3h, 4h, 5h)
        let hand = SelectHand::new(vec![c2, c3, c4, c5]);
        let rf = hand.is_royal_flush();
        assert_eq!(rf, None);
    }

    #[test]
    fn test_five_of_kind() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Spade);
        let c3 = Card::new(Value::King, Suit::Heart);
        let c4 = Card::new(Value::King, Suit::Diamond);
        let c5 = Card::new(Value::King, Suit::Heart);
        let not = Card::new(Value::Ace, Suit::Heart);

        // Valid 5 (K, K, K, K, K)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let is_5 = hand.is_five_of_kind();
        assert_eq!(is_5.unwrap().len(), 5);

        // Valid 5 from 7 cards (K, K, K, K, K, A, A)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5, not, not]);
        let is_5 = hand.is_five_of_kind();
        assert_eq!(is_5.unwrap().len(), 5);

        // Invalid 5 (K, K, K, K, A)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not]);
        let is_5 = hand.is_five_of_kind();
        assert_eq!(is_5, None);

        // Invalid 4 (K, K, K, K)
        let hand = SelectHand::new(vec![c1, c2, c3, c4]);
        let is_5 = hand.is_five_of_kind();
        assert_eq!(is_5, None);
    }

    #[test]
    fn test_five_of_kind_preserves_order() {
        let k1 = Card::new(Value::King, Suit::Heart);
        let k2 = Card::new(Value::King, Suit::Spade);
        let k3 = Card::new(Value::King, Suit::Diamond);
        let k4 = Card::new(Value::King, Suit::Club);
        let k5 = Card::new(Value::King, Suit::Heart);
        let ah = Card::new(Value::Ace, Suit::Heart);

        let hand = SelectHand::new(vec![k1, ah, k2, k3, k4, k5]);
        let is_5 = hand.is_five_of_kind().unwrap();
        assert_eq!(is_5.cards(), vec![k1, k2, k3, k4, k5]);
    }

    #[test]
    fn test_flush_house() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Heart);
        let c3 = Card::new(Value::King, Suit::Heart);
        let c4 = Card::new(Value::Ace, Suit::Heart);
        let c5 = Card::new(Value::Ace, Suit::Heart);
        let not1 = Card::new(Value::Two, Suit::Heart);
        let not2 = Card::new(Value::Ace, Suit::Diamond);

        // Valid 5 (Kh, Kh, Kh, Ah, Ah)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let fh = hand.is_flush_house();
        assert_eq!(fh.unwrap().len(), 5);

        // Invalid 5 (Kh, Kh, Kh, Ah, 2h)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not1]);
        let fh = hand.is_flush_house();
        assert_eq!(fh, None);

        // Invalid 5 (Kh, Kh, Kh, Ah, Ad)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not2]);
        let fh = hand.is_flush_house();
        assert_eq!(fh, None);

        // Invalid 4 (Kh, Kh, Kh, Ah)
        let hand = SelectHand::new(vec![c1, c2, c3, c4]);
        let fh = hand.is_flush_house();
        assert_eq!(fh, None);
    }

    #[test]
    fn test_flush_with_wild() {
        // 4 hearts + 1 Wild = flush
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::Queen, Suit::Heart);
        let c3 = Card::new(Value::Jack, Suit::Heart);
        let c4 = Card::new(Value::Seven, Suit::Heart);
        let mut wild = Card::new(Value::Two, Suit::Diamond);
        wild.enhancement = Some(Enhancement::Wild);

        let hand = SelectHand::new(vec![c1, c2, c3, c4, wild]);
        assert!(hand.is_flush().is_some());
        assert_eq!(hand.is_flush().unwrap().len(), 5);
    }

    #[test]
    fn test_flush_wild_insufficient() {
        // 3 hearts + 1 diamond + 1 Wild = not a flush
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::Queen, Suit::Heart);
        let c3 = Card::new(Value::Jack, Suit::Heart);
        let c4 = Card::new(Value::Ace, Suit::Diamond);
        let mut wild = Card::new(Value::Two, Suit::Club);
        wild.enhancement = Some(Enhancement::Wild);

        let hand = SelectHand::new(vec![c1, c2, c3, c4, wild]);
        assert!(hand.is_flush().is_none());
    }

    #[test]
    fn test_flush_two_wilds() {
        // 3 hearts + 2 Wilds = flush
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::Queen, Suit::Heart);
        let c3 = Card::new(Value::Jack, Suit::Heart);
        let mut wild1 = Card::new(Value::Two, Suit::Diamond);
        wild1.enhancement = Some(Enhancement::Wild);
        let mut wild2 = Card::new(Value::Three, Suit::Spade);
        wild2.enhancement = Some(Enhancement::Wild);

        let hand = SelectHand::new(vec![c1, c2, c3, wild1, wild2]);
        assert!(hand.is_flush().is_some());
    }

    #[test]
    fn test_flush_five() {
        let c1 = Card::new(Value::King, Suit::Heart);
        let c2 = Card::new(Value::King, Suit::Heart);
        let c3 = Card::new(Value::King, Suit::Heart);
        let c4 = Card::new(Value::King, Suit::Heart);
        let c5 = Card::new(Value::King, Suit::Heart);
        let not1 = Card::new(Value::Two, Suit::Heart);
        let not2 = Card::new(Value::King, Suit::Diamond);

        // Valid 5 (Kh, Kh, Kh, Kh, Kh)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
        let ff = hand.is_flush_five();
        assert_eq!(ff.unwrap().len(), 5);

        // Invalid 5 (Kh, Kh, Kh, Kh, 2h)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not1]);
        let ff = hand.is_flush_five();
        assert_eq!(ff, None);

        // Invalid 5 (Kh, Kh, Kh, Kh, Kd)
        let hand = SelectHand::new(vec![c1, c2, c3, c4, not2]);
        let ff = hand.is_flush_five();
        assert_eq!(ff, None);

        // Invalid 4 (Kh, Kh, Kh, Kh)
        let hand = SelectHand::new(vec![c1, c2, c3, c4]);
        let ff = hand.is_flush_five();
        assert_eq!(ff, None);
    }
}
