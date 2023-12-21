use anyhow::{Context, Error, Result};
use aoc2023lib::{init_logging, read_lines};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{trace, trace_span};
use valuable::Valuable;

#[derive(Clone, Eq, PartialEq, Debug, Valuable)]
struct Card {
    card_number: i32,
    numbers: Vec<i32>,
    winning_numbers: HashSet<i32>,
}

impl Card {
    fn parse(text: &str) -> Result<Self> {
        let (card_text, all_numbers) = text
            .split_once(":")
            .with_context(|| format!("Could not parse line {:?}", text))?;
        let (_, card_number_str) = card_text
            .split_once(" ")
            .with_context(|| format!("Could not split card number from {:?}", card_text))?;
        let card_number = card_number_str
            .trim()
            .parse::<i32>()
            .with_context(|| format!("Invalid number {:?}", card_number_str))?;

        let (left_numbers, right_numbers) = all_numbers
            .split_once(" | ")
            .with_context(|| format!("Could not split card numbers: {:?}", all_numbers))?;

        let numbers =
            parse_space_delimited_numbers(left_numbers).context("Unable to parse numbers")?;
        let winning_numbers = HashSet::from_iter(
            parse_space_delimited_numbers(right_numbers)
                .context("Unable to parse winning numbers")?
                .iter()
                .cloned(),
        );

        Ok(Self {
            card_number,
            numbers,
            winning_numbers,
        })
    }
}

fn parse_space_delimited_numbers(text: &str) -> Result<Vec<i32>> {
    let vec =
        text.split(" ")
            .filter(|&x| !x.is_empty())
            .try_fold(Vec::<i32>::new(), |mut acc, x| {
                acc.push(
                    x.trim()
                        .parse::<i32>()
                        .with_context(|| format!("Could not parse number from {:?}", x))?,
                );
                Ok::<Vec<_>, Error>(acc)
            })?;
    Ok(vec)
}

fn main() -> Result<()> {
    init_logging();
    let lines = read_lines("day04-scratchcards/input")?;

    let mut cards: Vec<Card> = vec![];

    for line_maybe in lines {
        let line_str = line_maybe?;
        let card = Card::parse(line_str.as_str())?;
        cards.push(card);
    }

    let scores: Vec<i32> = cards
        .clone()
        .iter()
        .map(|card| {
            card.numbers.iter().fold(0, |acc, number| {
                if card.winning_numbers.contains(number) {
                    if acc == 0 {
                        1
                    } else {
                        acc * 2
                    }
                } else {
                    acc
                }
            })
        })
        .collect();

    let cards_won = calculate_cards_won(cards.clone());

    let sum: i32 = scores.iter().sum();
    println!("Scratchcard score: {}", sum);

    println!("Cards won: {}", cards_won);

    Ok(())
}

fn calculate_cards_won(cards: Vec<Card>) -> i32 {
    let winnings_by_card_number: HashMap<i32, i32> = cards
        .clone()
        .iter()
        .map(|card| {
            (
                card.card_number,
                card.numbers.iter().fold(0, |acc, number| {
                    if card.winning_numbers.contains(number) {
                        acc + 1
                    } else {
                        acc
                    }
                }),
            )
        })
        .collect();

    let cards_by_number: HashMap<i32, Card> = cards
        .iter()
        .map(|card| (card.card_number, card.clone()))
        .collect();

    let mut queue: VecDeque<Card> = cards.iter().cloned().collect();

    let mut cards_won: i32 = 0;

    while let Some(card) = queue.pop_back() {
        let span = trace_span!(
            "loop",
            queue_length = queue.len(),
            card_number = card.card_number,
            cards_won = cards_won,
        )
        .entered();
        if let Some(winnings) = winnings_by_card_number.get(&card.card_number) {
            let mut new_cards: Vec<Card> = vec![];
            for i in 0..winnings.clone() {
                let next_card_idx = card.card_number + 1 + i;
                if let Some(won_card) = cards_by_number.get(&next_card_idx) {
                    new_cards.push(won_card.clone());
                }
            }
            // trace!(
            //     winnings = winnings,
            //     new_cards = new_cards.len(),
            //     first_new_card_number = new_cards.first().map(|card| card.card_number),
            //     "WON"
            // );
            for new_card in new_cards {
                queue.push_front(new_card);
            }
        }
        span.exit();
        cards_won += 1;
    }
    cards_won
}

#[cfg(test)]
mod tests {
    use crate::{parse_space_delimited_numbers, Card};
    use std::collections::HashSet;

    #[test]
    fn test_parse_space_delimited_numbers() {
        let text = "83 86  6 31 17  9 48 53";
        assert_eq!(
            parse_space_delimited_numbers(text).unwrap(),
            vec![83, 86, 6, 31, 17, 9, 48, 53]
        )
    }

    #[test]
    fn test_parse_line() {
        let line = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53";
        assert_eq!(
            Card::parse(line).unwrap(),
            Card {
                card_number: 1,
                numbers: vec![41, 48, 83, 86, 17],
                winning_numbers: HashSet::from_iter(vec![83, 86, 6, 31, 17, 9, 48, 53]),
            }
        )
    }
}
