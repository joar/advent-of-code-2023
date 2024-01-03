use crate::data::Cardish;
use anyhow::{Context, Result};
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, EnumString, Hash)]
pub enum Card {
    J = -1,
    #[strum(serialize = "2")]
    Two = -2,
    #[strum(serialize = "3")]
    Three = -3,
    #[strum(serialize = "4")]
    Four = -4,
    #[strum(serialize = "5")]
    Five = -5,
    #[strum(serialize = "6")]
    Six = -6,
    #[strum(serialize = "7")]
    Seven = -7,
    #[strum(serialize = "8")]
    Eight = -8,
    #[strum(serialize = "9")]
    Nine = -9,
    T = -10,
    Q = -12,
    K = -13,
    A = -14,
}

impl Cardish for Card {
    fn as_char(&self) -> char {
        match self {
            Card::Two => '2',
            Card::Three => '3',
            Card::Four => '4',
            Card::Five => '5',
            Card::Six => '6',
            Card::Seven => '7',
            Card::Eight => '8',
            Card::Nine => '9',
            Card::T => 'T',
            Card::J => 'J',
            Card::Q => 'Q',
            Card::K => 'K',
            Card::A => 'A',
        }
    }

    fn parse(s: &str) -> Result<Self> {
        Self::from_str(s).with_context(|| format!("Could not parse card from {s:?}", s = s))
    }
}

#[cfg(test)]
mod test {
    use crate::data::part2::Card;
    use std::str::FromStr;

    #[test]
    fn test_card_from_str() {
        let cards = "23456789TJQKA";
        let actual: Vec<Card> = cards
            .chars()
            .map(|c| Card::from_str(c.to_string().as_str()).unwrap())
            .collect();
        assert_eq!(
            actual,
            vec![
                Card::Two,
                Card::Three,
                Card::Four,
                Card::Five,
                Card::Six,
                Card::Seven,
                Card::Eight,
                Card::Nine,
                Card::T,
                Card::J,
                Card::Q,
                Card::K,
                Card::A,
            ]
        )
    }

    #[test]
    fn test_sort_cards() {
        let actual = {
            let mut cards = vec![
                Card::Three,
                Card::Four,
                Card::Six,
                Card::Five,
                Card::Eight,
                Card::Two,
                Card::Q,
                Card::Seven,
                Card::Nine,
                Card::T,
                Card::J,
                Card::K,
                Card::A,
            ];
            cards.sort();
            cards
        };

        assert_eq!(
            actual,
            vec![
                Card::A,
                Card::K,
                Card::Q,
                Card::T,
                Card::Nine,
                Card::Eight,
                Card::Seven,
                Card::Six,
                Card::Five,
                Card::Four,
                Card::Three,
                Card::Two,
                Card::J,
            ]
        )
    }
}
