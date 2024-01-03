use std::fs::read_to_string;

use anyhow::Result;

use aoc2023lib::init_logging;

use crate::data::{Bid, Card, Cardish, Hand};
use crate::parse::parse_input;

#[cfg(test)]
static TEST_INPUT: &str = "32T3K 765
T55J5 684
KK677 28\x20
KTJJT 220
QQQJA 483";

fn main() -> Result<()> {
    init_logging();
    // Part one
    {
        let parsed = parse_input::<Card>(read_to_string("day07-camel/input")?.as_str())?;
        let winnings = calculate_winnings(calculate_ranks(parsed));
        println!("Part one answer: {}", winnings);
    }
    Ok(())
}

fn calculate_ranks<T>(hand_bids: Vec<(Hand<T>, Bid)>) -> Vec<(usize, (Hand<T>, Bid))>
where
    T: Cardish,
{
    let mut sorted_hands = hand_bids.clone();
    sorted_hands.sort();
    sorted_hands
        .into_iter()
        .rev()
        .enumerate()
        .map(|(rank, x)| (rank + 1, x))
        .collect()
}

fn calculate_winnings<T>(ranks: Vec<(usize, (Hand<T>, Bid))>) -> usize
where
    T: Cardish,
{
    ranks
        .into_iter()
        .map(|(rank, (_, bid))| rank * bid.amount() as usize)
        .sum()
}

#[cfg(test)]
mod test {
    use crate::data::{Card, Hand};
    use crate::parse::{parse_input, parse_line};
    use crate::{calculate_ranks, calculate_winnings, TEST_INPUT};

    #[test]
    fn test_calculate_rank() {
        let parsed = parse_input::<Card>(TEST_INPUT).unwrap();
        let actual: Vec<_> = calculate_ranks(parsed)
            .into_iter()
            .map(|(rank, (hand, _bid))| (rank, hand))
            .collect();
        assert_eq!(
            actual,
            vec![
                (1, Hand::parse("32T3K").unwrap()),
                (2, Hand::parse("KTJJT").unwrap()),
                (3, Hand::parse("KK677").unwrap()),
                (4, Hand::parse("T55J5").unwrap()),
                (5, Hand::parse("QQQJA").unwrap()),
            ]
        )
    }

    #[test]
    fn test_calculate_winnings() {
        let parsed: Vec<_> = crate::TEST_INPUT
            .lines()
            .map(parse_line::<Card>)
            .collect::<anyhow::Result<Vec<_>>>()
            .unwrap();
        let actual = calculate_winnings(calculate_ranks(parsed));
        assert_eq!(actual, 6440);
    }
}

mod data;

mod parse;
