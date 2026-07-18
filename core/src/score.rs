use crate::card::Card;
use crate::joker::{joker_display, Jokers};

/// What caused a `ScoreStep`'s chips/mult change.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ScoreSource {
    HandLevel,
    PlayedCard(Card),
    // stone cards always score +50 chips, even as kickers outside the
    // played hand's own scoring subset
    StoneKicker(Card),
    HeldCard(Card),
    Joker(Jokers),
}

/// One step in a scoring pass
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreStep {
    pub source: ScoreSource,
    pub chips_before: usize,
    pub chips_after: usize,
    pub mult_before: usize,
    pub mult_after: usize,
    pub retrigger: bool,
}

impl ScoreStep {
    pub fn delta_chips(&self) -> i64 {
        self.chips_after as i64 - self.chips_before as i64
    }

    pub fn delta_mult(&self) -> i64 {
        self.mult_after as i64 - self.mult_before as i64
    }

    /// Human-readable summary, e.g. "K♠: +10 Chips" or "Mime (retrigger)".
    pub fn describe(&self) -> String {
        let label = match &self.source {
            ScoreSource::HandLevel => "Hand level".to_string(),
            ScoreSource::PlayedCard(c) => c.to_string(),
            ScoreSource::StoneKicker(c) => format!("{c} (stone kicker)"),
            ScoreSource::HeldCard(c) => format!("{c} (held)"),
            ScoreSource::Joker(j) => joker_display(j),
        };

        let mut parts = Vec::new();
        let dc = self.delta_chips();
        if dc != 0 {
            parts.push(format!("{dc:+} Chips"));
        }
        let dm = self.delta_mult();
        if dm != 0 {
            parts.push(format!("{dm:+} Mult"));
        }

        let suffix = if self.retrigger { " (retrigger)" } else { "" };
        if parts.is_empty() {
            format!("{label}{suffix}")
        } else {
            format!("{label}: {}{suffix}", parts.join(", "))
        }
    }
}

/// Ordered ledger of every `ScoreStep` a `calc_score_traced` call produced.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScoreTrace(pub Vec<ScoreStep>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_formats_chips_and_mult() {
        let step = ScoreStep {
            source: ScoreSource::HandLevel,
            chips_before: 0,
            chips_after: 10,
            mult_before: 1,
            mult_after: 3,
            retrigger: false,
        };
        assert_eq!(step.describe(), "Hand level: +10 Chips, +2 Mult");
    }

    #[test]
    fn test_describe_marks_retrigger() {
        let card = Card::new(crate::card::Value::King, crate::card::Suit::Spade);
        let step = ScoreStep {
            source: ScoreSource::PlayedCard(card),
            chips_before: 10,
            chips_after: 20,
            mult_before: 2,
            mult_after: 2,
            retrigger: true,
        };
        assert_eq!(step.describe(), "K♠: +10 Chips (retrigger)");
    }

    #[test]
    fn test_describe_no_delta() {
        let card = Card::new(crate::card::Value::Ace, crate::card::Suit::Heart);
        let step = ScoreStep {
            source: ScoreSource::HeldCard(card),
            chips_before: 5,
            chips_after: 5,
            mult_before: 1,
            mult_after: 1,
            retrigger: false,
        };
        assert_eq!(step.describe(), "A♥ (held)");
    }
}
