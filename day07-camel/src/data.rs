use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::str::FromStr;

use anyhow::Result;
use anyhow::{anyhow, Context};
use strum_macros::EnumString;
use thiserror::Error;
use tracing::{instrument, trace};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
pub enum Type {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
}

impl Type {
    #[instrument(ret)]
    pub fn from_cards<T>(hand: CardsOnHand<T>) -> Self
    where
        T: Cardish,
    {
        let cards = hand.as_vec();
        let count_by_card = {
            let mut map: HashMap<T, u8> = HashMap::new();
            for card in cards.as_slice() {
                *map.entry(*card).or_insert(0) += 1;
            }
            map
        };

        let num_with_count = {
            let mut map: HashMap<usize, usize> = HashMap::new();
            for count in (0..=5).rev() {
                map.insert(
                    count,
                    count_by_card
                        .iter()
                        .filter(|&(_k, &v)| v as usize == count)
                        .map(|(k, _v)| {
                            trace!("{count:?} of {k:?}", count = count, k = k);
                        })
                        .count(),
                );
            }
            map
        };

        if num_with_count.get(&5).cloned().unwrap_or(0) == 1 {
            Self::FiveOfAKind
        } else if num_with_count.get(&4).cloned().unwrap_or(0) == 1 {
            Self::FourOfAKind
        } else if num_with_count.get(&3).cloned().unwrap_or(0) == 1 {
            if num_with_count.get(&2).cloned().unwrap_or(0) == 1 {
                Self::FullHouse
            } else {
                Self::ThreeOfAKind
            }
        } else if num_with_count.get(&2).cloned().unwrap_or(0) == 2 {
            Self::TwoPair
        } else if num_with_count.get(&2).cloned().unwrap_or(0) == 1 {
            Self::OnePair
        } else {
            Self::HighCard
        }
    }
}

impl<T> TryFrom<CardsOnHand<T>> for Type
where
    T: Cardish,
{
    type Error = anyhow::Error;

    fn try_from(value: CardsOnHand<T>) -> std::result::Result<Self, Self::Error> {
        Ok(Self::from_cards(value))
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, EnumString, Hash)]
pub enum Card {
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
    J = -11,
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

pub trait Cardish:
    Debug + Ord + PartialOrd + Eq + PartialEq + Sized + Copy + Clone + Hash + FromStr
{
    fn as_char(&self) -> char;
    fn parse(s: &str) -> Result<Self>;
}

mod part2;

/// A hand of five cards
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct CardsOnHand<T>
where
    T: Cardish,
{
    a: T,
    b: T,
    c: T,
    d: T,
    e: T,
}

impl<T> CardsOnHand<T>
where
    T: Cardish,
{
    pub fn new(a: T, b: T, c: T, d: T, e: T) -> Self {
        CardsOnHand { a, b, c, d, e }
    }

    pub fn as_vec(&self) -> Vec<T> {
        vec![self.a, self.b, self.c, self.d, self.e]
    }

    pub fn parse(s: &str) -> Result<Self> {
        Self::from_str(s)
    }
}

#[derive(Error, Debug)]
#[error("{source}")]
pub struct CardsOnHandFromStrError {
    #[source]
    source: anyhow::Error,
}

impl<T> FromStr for CardsOnHand<T>
where
    T: Cardish,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.len() != 5 {
            return Err(anyhow!(
                "Expected a string of length 5, was length {}",
                s.len()
            ));
        }
        let cards = s
            .char_indices()
            .map(|(i, chr)| {
                T::parse(chr.to_string().as_str()).with_context(|| {
                    format!(
                        "{char:?} at index {idx} is not a valid card",
                        char = chr,
                        idx = i,
                    )
                })
            })
            .collect::<Result<Vec<_>>>()
            .with_context(|| format!("Could not parse cards from {string:?}", string = s))?;
        match cards.as_slice().to_owned()[..] {
            [a, b, c, d, e] => Ok(CardsOnHand::new(a, b, c, d, e)),
            _ => Err(anyhow!(
                "Expected to parse 5 cards from {source:?}, got {num_cards:?} cards: {cards:?}",
                source = s,
                num_cards = cards.len(),
                cards = cards
            )),
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Hand<T>
where
    T: Cardish,
{
    r#type: Type,
    cards: CardsOnHand<T>,
}

impl<T> Debug for Hand<T>
where
    T: Cardish,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "Hand {{ type: {:?}, cards: {:?} }}",
                self.r#type,
                self.cards
                    .as_vec()
                    .iter()
                    .map(|card| card.as_char().to_string())
                    .collect::<Vec<_>>()
                    .join("")
            )
            .as_str(),
        )
    }
}

impl<T> Hand<T>
where
    T: Cardish,
{
    pub fn new(r#type: Type, cards: CardsOnHand<T>) -> Self {
        Self { r#type, cards }
    }
    #[instrument(ret)]
    pub fn parse(s: &str) -> Result<Self> {
        let cards_on_hand =
            CardsOnHand::parse(s).with_context(|| format!("Could not parse Hand from {:?}", s))?;
        let r#type = Type::from_cards(cards_on_hand);
        Ok(Self {
            r#type,
            cards: cards_on_hand,
        })
    }
    pub fn r#type(&self) -> Type {
        self.r#type
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Bid {
    amount: u32,
}

impl Bid {
    pub fn new(amount: u32) -> Self {
        Self { amount }
    }
    pub fn amount(&self) -> u32 {
        self.amount
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;
    use ctor::ctor;

    use aoc2023lib::init_logging;

    use crate::data::{Card, CardsOnHand, Type};

    #[ctor]
    fn init() {
        init_logging();
    }

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
    fn test_cards_on_hand_parse() {
        let hand = "AKJQQ";
        let actual = CardsOnHand::parse(hand).unwrap();
        assert_eq!(
            actual,
            CardsOnHand::new(Card::A, Card::K, Card::J, Card::Q, Card::Q)
        );
    }

    #[test]
    fn test_cards_on_hand_parse_error() {
        let hand = "AKXQQ";
        let actual = CardsOnHand::<Card>::parse(hand);
        assert!(actual.is_err());
    }

    #[test]
    fn test_type_from_cards() {
        let actual = ["32T3K", "T55J5", "KK677", "KTJJT", "QQQJA"]
            .iter()
            .map(|s| CardsOnHand::<Card>::parse(s)?.try_into())
            .collect::<Result<Vec<Type>>>()
            .unwrap();

        assert_eq!(
            actual,
            vec![
                Type::OnePair,
                Type::ThreeOfAKind,
                Type::TwoPair,
                Type::TwoPair,
                Type::ThreeOfAKind
            ]
        )
    }
}
