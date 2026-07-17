use std::collections::{HashMap, HashSet};
use std::fmt;

use balatro_jkr::LuaValue;
use balatro_types::{
    Consumable, DeckVariant, Edition, HandRank, Jokers, Planets, Spectral, Stake, Tarot, Voucher,
};
use strum::IntoEnumIterator;

use crate::error::ProfileError;
use crate::fmt_num::format_number;
use crate::item_id::ItemId;
use crate::lua_ext::{as_num, as_str, get, require, str_entries};

#[derive(Debug, Clone, PartialEq)]
pub struct HighScoreEntry {
    pub label: String,
    pub amount: u64,
}

/// The `high_scores` table's fixed set of named entries.
#[derive(Debug, Clone, PartialEq)]
pub struct HighScores {
    pub collection: HighScoreEntry,
    pub furthest_round: HighScoreEntry,
    pub furthest_ante: HighScoreEntry,
    pub best_hand: HighScoreEntry,
    pub current_streak: HighScoreEntry,
    pub most_money: HighScoreEntry,
    pub boss_streak: HighScoreEntry,
    pub win_streak: HighScoreEntry,
    pub most_played_hand: HighScoreEntry,
}

/// `count`/`order` usage stats for a joker, consumable, or voucher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Usage {
    pub count: u64,
    pub order: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeckUsage {
    pub count: u64,
    pub order: u64,
    /// Ante number -> win count.
    pub wins_by_ante: HashMap<u64, u64>,
}

/// Flat counters from `career_stats`, defaulting missing fields to 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CareerStats {
    pub dollars_earned: u64,
    pub shop_dollars_spent: u64,
    pub tarots_bought: u64,
    pub planets_bought: u64,
    pub playing_cards_bought: u64,
    pub vouchers_bought: u64,
    pub tarot_reading_used: u64,
    pub planetarium_used: u64,
    pub shop_rerolls: u64,
    pub cards_played: u64,
    pub cards_discarded: u64,
    pub losses: u64,
    pub wins: u64,
    pub rounds: u64,
    pub hands_played: u64,
    pub face_cards_played: u64,
    pub jokers_sold: u64,
    pub cards_sold: u64,
    pub round_interest_cap_streak: u64,
    pub single_hand_round_streak: u64,
}

/// Stake/deck last browsed in the menu — not the current save's stake/deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LastPlayed {
    pub stake: Stake,
    pub deck: DeckVariant,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChallengeProgress {
    pub unlocked: HashSet<String>,
    pub completed: HashSet<String>,
    pub challenges_unlocked_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub name: String,
    pub unlocked: HashSet<ItemId>,
    pub discovered: HashSet<ItemId>,
    pub alerted: HashSet<ItemId>,
    pub high_scores: HighScores,
    pub joker_usage: HashMap<Jokers, Usage>,
    pub consumable_usage: HashMap<Consumable, Usage>,
    pub voucher_usage: HashMap<Voucher, Usage>,
    pub hand_usage: HashMap<HandRank, u64>,
    pub deck_usage: HashMap<DeckVariant, DeckUsage>,
    /// Shape unresolved — always empty in real fixtures, kept raw.
    pub deck_stakes: LuaValue,
    pub career_stats: CareerStats,
    pub last_played: LastPlayed,
    pub challenges: ChallengeProgress,
}

impl Profile {
    pub fn from_lua(meta: &LuaValue, profile: &LuaValue) -> Result<Profile, ProfileError> {
        Ok(Profile {
            name: as_str(require(profile, "name")?)
                .ok_or(ProfileError::WrongType {
                    field: "name",
                    expected: "string",
                })?
                .to_string(),
            unlocked: parse_item_id_set(require(meta, "unlocked")?),
            discovered: parse_item_id_set(require(meta, "discovered")?),
            alerted: parse_item_id_set(require(meta, "alerted")?),
            high_scores: parse_high_scores(require(profile, "high_scores")?)?,
            joker_usage: parse_usage_map(require(profile, "joker_usage")?, Jokers::from_id),
            consumable_usage: parse_usage_map(
                require(profile, "consumeable_usage")?,
                Consumable::from_id,
            ),
            voucher_usage: parse_usage_map(require(profile, "voucher_usage")?, Voucher::from_id),
            hand_usage: parse_hand_usage(require(profile, "hand_usage")?),
            deck_usage: parse_deck_usage(require(profile, "deck_usage")?),
            deck_stakes: require(profile, "deck_stakes")?.clone(),
            career_stats: parse_career_stats(require(profile, "career_stats")?),
            last_played: parse_last_played(profile)?,
            challenges: parse_challenge_progress(profile)?,
        })
    }

    /// A short, grouped overview — see [`ProfileSummary`].
    pub fn summary(&self) -> ProfileSummary<'_> {
        ProfileSummary(self)
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Profile: {}", self.name)?;
        writeln!(
            f,
            "Last played: {}, {:?} Stake",
            self.last_played.deck.name(),
            self.last_played.stake
        )?;

        write_item_status_list(f, &self.unlocked, &self.discovered, &self.alerted)?;

        writeln!(f, "\nHigh Scores:")?;
        for (label, e) in [
            ("Collection", &self.high_scores.collection),
            ("Furthest Round", &self.high_scores.furthest_round),
            ("Furthest Ante", &self.high_scores.furthest_ante),
            ("Best Hand", &self.high_scores.best_hand),
            ("Current Streak", &self.high_scores.current_streak),
            ("Most Money", &self.high_scores.most_money),
            ("Boss Streak", &self.high_scores.boss_streak),
            ("Win Streak", &self.high_scores.win_streak),
            ("Most Played Hand", &self.high_scores.most_played_hand),
        ] {
            writeln!(
                f,
                "  {label}: {} ({})",
                format_number(e.amount as i64),
                e.label
            )?;
        }

        writeln!(f, "\nJoker Usage ({}):", self.joker_usage.len())?;
        for (j, u) in sorted_by_count(&self.joker_usage, |j| j.name().to_string(), |u| u.count) {
            writeln!(
                f,
                "  {}: played {}x (discovered #{})",
                j.name(),
                format_number(u.count as i64),
                u.order
            )?;
        }

        writeln!(f, "\nConsumable Usage ({}):", self.consumable_usage.len())?;
        for (c, u) in sorted_by_count(&self.consumable_usage, |c| c.name(), |u| u.count) {
            writeln!(
                f,
                "  {}: used {}x (discovered #{})",
                c.name(),
                format_number(u.count as i64),
                u.order
            )?;
        }

        writeln!(f, "\nVoucher Usage ({}):", self.voucher_usage.len())?;
        for (v, u) in sorted_by_count(&self.voucher_usage, |v| v.name().to_string(), |u| u.count) {
            writeln!(
                f,
                "  {}: bought {}x (discovered #{})",
                v.name(),
                format_number(u.count as i64),
                u.order
            )?;
        }

        writeln!(f, "\nHand Usage ({}):", self.hand_usage.len())?;
        let mut hands: Vec<_> = self.hand_usage.iter().collect();
        hands.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.id().cmp(b.0.id())));
        for (h, count) in hands {
            writeln!(f, "  {}: {}x", h.id(), format_number(*count as i64))?;
        }

        writeln!(f, "\nDeck Usage ({}):", self.deck_usage.len())?;
        for (d, u) in sorted_by_count(&self.deck_usage, |d| d.name().to_string(), |u| u.count) {
            let mut wins: Vec<_> = u.wins_by_ante.iter().collect();
            wins.sort();
            let wins_str = wins
                .iter()
                .map(|(ante, n)| format!("ante {ante}: {n}"))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                f,
                "  {}: played {}x (discovered #{}) — wins: [{wins_str}]",
                d.name(),
                format_number(u.count as i64),
                u.order
            )?;
        }

        writeln!(
            f,
            "\nDeck Stakes: UNRESOLVED (raw value: {:?})",
            self.deck_stakes
        )?;

        writeln!(f, "\nCareer Stats:")?;
        let cs = &self.career_stats;
        for (label, value) in [
            ("Dollars earned", cs.dollars_earned),
            ("Shop dollars spent", cs.shop_dollars_spent),
            ("Tarots bought", cs.tarots_bought),
            ("Planets bought", cs.planets_bought),
            ("Playing cards bought", cs.playing_cards_bought),
            ("Vouchers bought", cs.vouchers_bought),
            ("Tarot reading used", cs.tarot_reading_used),
            ("Planetarium used", cs.planetarium_used),
            ("Shop rerolls", cs.shop_rerolls),
            ("Cards played", cs.cards_played),
            ("Cards discarded", cs.cards_discarded),
            ("Losses", cs.losses),
            ("Wins", cs.wins),
            ("Rounds", cs.rounds),
            ("Hands played", cs.hands_played),
            ("Face cards played", cs.face_cards_played),
            ("Jokers sold", cs.jokers_sold),
            ("Cards sold", cs.cards_sold),
            ("Round interest cap streak", cs.round_interest_cap_streak),
            ("Single hand round streak", cs.single_hand_round_streak),
        ] {
            writeln!(f, "  {label}: {}", format_number(value as i64))?;
        }

        writeln!(
            f,
            "\nChallenges: {} unlocked, {} completed ({} unlock count reported)",
            self.challenges.unlocked.len(),
            self.challenges.completed.len(),
            self.challenges.challenges_unlocked_count
        )?;

        Ok(())
    }
}

/// One row per item across `unlocked`/`discovered`/`alerted`, with status
/// flags, instead of three separate lists.
fn write_item_status_list(
    f: &mut fmt::Formatter<'_>,
    unlocked: &HashSet<ItemId>,
    discovered: &HashSet<ItemId>,
    alerted: &HashSet<ItemId>,
) -> fmt::Result {
    let mut all: HashSet<&ItemId> = HashSet::new();
    all.extend(unlocked.iter());
    all.extend(discovered.iter());
    all.extend(alerted.iter());

    let mut rows: Vec<(String, &ItemId)> = all.into_iter().map(|i| (i.to_string(), i)).collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));

    writeln!(
        f,
        "\nCollection ({}) — [U]nlocked [D]iscovered [A]lerted:",
        rows.len()
    )?;
    for (name, item) in rows {
        let u = if unlocked.contains(item) { 'U' } else { '-' };
        let d = if discovered.contains(item) { 'D' } else { '-' };
        let a = if alerted.contains(item) { 'A' } else { '-' };
        writeln!(f, "  [{u}{d}{a}] {name}")?;
    }
    Ok(())
}

/// Sorts a usage map by descending count, tie-broken by name.
fn sorted_by_count<K, V>(
    map: &HashMap<K, V>,
    name: impl Fn(&K) -> String,
    count: impl Fn(&V) -> u64,
) -> Vec<(&K, &V)> {
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by(|a, b| {
        count(b.1)
            .cmp(&count(a.1))
            .then_with(|| name(a.0).cmp(&name(b.0)))
    });
    entries
}

/// Short overview of a [`Profile`].
pub struct ProfileSummary<'a>(pub &'a Profile);

impl fmt::Display for ProfileSummary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = self.0;
        let cs = &p.career_stats;

        writeln!(f, "{}", p.name)?;
        writeln!(
            f,
            "Last played: {}, {:?} Stake",
            p.last_played.deck.name(),
            p.last_played.stake
        )?;
        writeln!(f)?;

        writeln!(
            f,
            "Best hand score: {}",
            format_number(p.high_scores.best_hand.amount as i64)
        )?;
        writeln!(
            f,
            "Furthest ante: {}",
            format_number(p.high_scores.furthest_ante.amount as i64)
        )?;
        let win_rate = if cs.wins + cs.losses > 0 {
            cs.wins * 100 / (cs.wins + cs.losses)
        } else {
            0
        };
        writeln!(
            f,
            "Wins: {} / Rounds: {} ({win_rate}%)",
            format_number(cs.wins as i64),
            format_number(cs.rounds as i64)
        )?;
        writeln!(
            f,
            "Best win streak: {}",
            format_number(p.high_scores.win_streak.amount as i64)
        )?;
        writeln!(f, "Hands played: {}", format_number(cs.hands_played as i64))?;
        writeln!(f, "Top hands: {}", top_n_line(&self.hand_counts(), 3))?;
        writeln!(
            f,
            "Most money: ${} (earned ${}, spent ${})",
            format_number(p.high_scores.most_money.amount as i64),
            format_number(cs.dollars_earned as i64),
            format_number(cs.shop_dollars_spent as i64)
        )?;
        writeln!(f)?;

        let jokers_unlocked = p
            .unlocked
            .iter()
            .filter(|i| matches!(i, ItemId::Joker(_)))
            .count();
        let jokers_bought: u64 = p.joker_usage.values().map(|u| u.count).sum();
        writeln!(
            f,
            "Jokers: {jokers_unlocked} unlocked, {} played, {} bought, {} sold",
            p.joker_usage.len(),
            format_number(jokers_bought as i64),
            format_number(cs.jokers_sold as i64)
        )?;
        let top_jokers: Vec<(String, u64)> =
            sorted_by_count(&p.joker_usage, |j| j.name().to_string(), |u| u.count)
                .into_iter()
                .map(|(j, u)| (j.name().to_string(), u.count))
                .collect();
        writeln!(f, "Top jokers: {}", top_n_line(&top_jokers, 3))?;

        let consumables_discovered = p
            .discovered
            .iter()
            .filter(|i| matches!(i, ItemId::Consumable(_)))
            .count();
        let consumables_used: u64 = p.consumable_usage.values().map(|u| u.count).sum();
        writeln!(
            f,
            "Consumables: {consumables_discovered}/{} discovered, {} used",
            Tarot::iter().count() + Planets::iter().count() + Spectral::iter().count(),
            format_number(consumables_used as i64)
        )?;
        writeln!(f)?;

        let vouchers_unlocked = p
            .unlocked
            .iter()
            .filter(|i| matches!(i, ItemId::Voucher(_)))
            .count();
        let decks_unlocked = p
            .unlocked
            .iter()
            .filter(|i| matches!(i, ItemId::Deck(_)))
            .count();
        writeln!(
            f,
            "Vouchers unlocked: {vouchers_unlocked}/{}",
            Voucher::iter().count()
        )?;
        write!(
            f,
            "Decks unlocked: {decks_unlocked}/{}",
            DeckVariant::iter().count()
        )?;

        Ok(())
    }
}

impl ProfileSummary<'_> {
    fn hand_counts(&self) -> Vec<(String, u64)> {
        self.0
            .hand_usage
            .iter()
            .map(|(h, count)| (format!("{h:?}"), *count))
            .collect()
    }
}

/// Formats the top `n` entries by count as `"Name (Nx), Name (Nx), ..."`.
fn top_n_line(entries: &[(String, u64)], n: usize) -> String {
    let mut sorted: Vec<_> = entries.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted
        .into_iter()
        .take(n)
        .map(|(name, count)| format!("{name} ({}x)", format_number(*count as i64)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Builds an `ItemId` set, skipping `c_base` and `e_base`.
fn parse_item_id_set(v: &LuaValue) -> HashSet<ItemId> {
    str_entries(v)
        .filter_map(|(id, _val)| ItemId::from_id(id))
        .filter(|id| !matches!(id, ItemId::Edition(Edition::Base)))
        .collect()
}

fn parse_high_scores(v: &LuaValue) -> Result<HighScores, ProfileError> {
    let entry = |key: &'static str| -> Result<HighScoreEntry, ProfileError> {
        let e = require(v, key)?;
        Ok(HighScoreEntry {
            label: as_str(require(e, "label")?).unwrap_or("").to_string(),
            amount: as_num(require(e, "amt")?).unwrap_or(0.0) as u64,
        })
    };
    Ok(HighScores {
        collection: entry("collection")?,
        furthest_round: entry("furthest_round")?,
        furthest_ante: entry("furthest_ante")?,
        best_hand: entry("hand")?,
        current_streak: entry("current_streak")?,
        most_money: entry("most_money")?,
        boss_streak: entry("boss_streak")?,
        win_streak: entry("win_streak")?,
        most_played_hand: entry("poker_hand")?,
    })
}

fn parse_usage_map<K: std::hash::Hash + Eq>(
    v: &LuaValue,
    from_id: impl Fn(&str) -> Option<K>,
) -> HashMap<K, Usage> {
    str_entries(v)
        .filter_map(|(id, val)| {
            let key = from_id(id)?;
            let count = get(val, "count").and_then(as_num).unwrap_or(0.0) as u64;
            let order = get(val, "order").and_then(as_num).unwrap_or(0.0) as u64;
            Some((key, Usage { count, order }))
        })
        .collect()
}

/// `hand_usage`'s `order` is a display-name string, not a number, so only
/// `count` is kept.
fn parse_hand_usage(v: &LuaValue) -> HashMap<HandRank, u64> {
    str_entries(v)
        .filter_map(|(id, val)| {
            let rank = HandRank::from_id(id)?;
            let count = get(val, "count").and_then(as_num).unwrap_or(0.0) as u64;
            Some((rank, count))
        })
        .collect()
}

fn parse_deck_usage(v: &LuaValue) -> HashMap<DeckVariant, DeckUsage> {
    str_entries(v)
        .filter_map(|(id, val)| {
            let deck = DeckVariant::from_id(id)?;
            let count = get(val, "count").and_then(as_num).unwrap_or(0.0) as u64;
            let order = get(val, "order").and_then(as_num).unwrap_or(0.0) as u64;
            let wins_by_ante = get(val, "wins")
                .map(|wins| {
                    str_entries(wins)
                        .filter_map(|(k, v)| Some((k.parse().ok()?, as_num(v)? as u64)))
                        .collect()
                })
                .unwrap_or_default();
            Some((
                deck,
                DeckUsage {
                    count,
                    order,
                    wins_by_ante,
                },
            ))
        })
        .collect()
}

fn field_u64(v: &LuaValue, key: &str) -> u64 {
    get(v, key).and_then(as_num).unwrap_or(0.0) as u64
}

fn parse_career_stats(v: &LuaValue) -> CareerStats {
    CareerStats {
        dollars_earned: field_u64(v, "c_dollars_earned"),
        shop_dollars_spent: field_u64(v, "c_shop_dollars_spent"),
        tarots_bought: field_u64(v, "c_tarots_bought"),
        planets_bought: field_u64(v, "c_planets_bought"),
        playing_cards_bought: field_u64(v, "c_playing_cards_bought"),
        vouchers_bought: field_u64(v, "c_vouchers_bought"),
        tarot_reading_used: field_u64(v, "c_tarot_reading_used"),
        planetarium_used: field_u64(v, "c_planetarium_used"),
        shop_rerolls: field_u64(v, "c_shop_rerolls"),
        cards_played: field_u64(v, "c_cards_played"),
        cards_discarded: field_u64(v, "c_cards_discarded"),
        losses: field_u64(v, "c_losses"),
        wins: field_u64(v, "c_wins"),
        rounds: field_u64(v, "c_rounds"),
        hands_played: field_u64(v, "c_hands_played"),
        face_cards_played: field_u64(v, "c_face_cards_played"),
        jokers_sold: field_u64(v, "c_jokers_sold"),
        cards_sold: field_u64(v, "c_cards_sold"),
        round_interest_cap_streak: field_u64(v, "c_round_interest_cap_streak"),
        single_hand_round_streak: field_u64(v, "c_single_hand_round_streak"),
    }
}

fn parse_last_played(profile: &LuaValue) -> Result<LastPlayed, ProfileError> {
    let memory = require(profile, "MEMORY")?;
    let stake_n = as_num(require(memory, "stake")?).unwrap_or(0.0) as u8;
    let stake =
        Stake::from_id(stake_n).ok_or(ProfileError::UnknownId(format!("stake {stake_n}")))?;
    let deck_name = as_str(require(memory, "deck")?).unwrap_or("");
    let deck = deck_variant_from_name(deck_name)
        .ok_or_else(|| ProfileError::UnknownId(deck_name.to_string()))?;
    Ok(LastPlayed { stake, deck })
}

/// `MEMORY.deck` stores the display name, not an id.
fn deck_variant_from_name(name: &str) -> Option<DeckVariant> {
    use strum::IntoEnumIterator;
    DeckVariant::iter().find(|d| d.name() == name)
}

fn parse_challenge_progress(profile: &LuaValue) -> Result<ChallengeProgress, ProfileError> {
    let cp = require(profile, "challenge_progress")?;
    let unlocked = str_entries(require(cp, "unlocked")?)
        .map(|(k, _)| k.to_string())
        .collect();
    let completed = str_entries(require(cp, "completed")?)
        .map(|(k, _)| k.to_string())
        .collect();
    let challenges_unlocked_count = field_u64(profile, "challenges_unlocked");
    Ok(ChallengeProgress {
        unlocked,
        completed,
        challenges_unlocked_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture(name: &str) -> LuaValue {
        let bytes = fs::read(format!("tests/fixtures/real/{name}")).expect("fixture file present");
        balatro_jkr::decode(&bytes).expect("valid jkr")
    }

    #[test]
    fn test_parses_real_1_profile() {
        let meta = fixture("1-meta.jkr");
        let profile = fixture("1-profile.jkr");
        let p = Profile::from_lua(&meta, &profile).expect("parses");
        assert_eq!(p.name, "EVAN");
        assert!(!p.unlocked.is_empty());
        assert!(
            p.unlocked
                .contains(&ItemId::Joker(Jokers::ScaryFace(Default::default())))
        );
        assert_eq!(p.last_played.deck, DeckVariant::Red);
        assert_eq!(p.last_played.stake, Stake::Purple);
        assert_eq!(p.career_stats.wins, 27);
        assert_eq!(p.career_stats.rounds, 1272);
        // deck_stakes is deliberately untyped and confirmed empty here.
        assert_eq!(p.deck_stakes, LuaValue::Table(vec![]));
        // structural defaults never leak into the unlocked set
        assert!(
            !p.unlocked
                .iter()
                .any(|id| matches!(id, ItemId::Edition(Edition::Base)))
        );
    }

    #[test]
    fn test_parses_real_2_and_3_profiles() {
        for n in [2, 3] {
            let meta = fixture(&format!("{n}-meta.jkr"));
            let profile = fixture(&format!("{n}-profile.jkr"));
            let p = Profile::from_lua(&meta, &profile).expect("parses");
            assert!(!p.name.is_empty());
            assert_eq!(p.last_played.stake, Stake::White);
        }
    }
}
