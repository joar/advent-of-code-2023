use std::collections::HashMap;
use std::ops::Range;

use anyhow::Context;
use tracing::{instrument, trace, trace_span};
use valuable::Valuable;

use crate::digit_word::DigitWord;
use crate::utils::format_text_span;

#[instrument]
pub fn find_numbers(text: &str) -> anyhow::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    let mut match_start_positions: HashMap<usize, Vec<DigitWord>> =
        create_start_positions(text.len());
    let mut next_start_positions: HashMap<usize, Vec<DigitWord>> =
        create_start_positions(text.len());

    let mut recognized_ranges: Vec<Range<usize>> = Vec::new();

    for (cursor_pos, char) in text.chars().enumerate() {
        let span = trace_span!("match", cursor_pos = cursor_pos).entered();
        match char.is_numeric() {
            true => {
                buf.push(char.to_string().parse::<u8>()?);
            }
            false => {
                // Match first letter of all digit-words
                for dw in DigitWord::all() {
                    if dw.str_value().starts_with(char) {
                        trace!(
                            event = "START",
                            candidate = dw.as_value(),
                            "\n{}",
                            format_text_span(text, cursor_pos..cursor_pos + 1)
                        );
                        next_start_positions
                            .get_mut(&cursor_pos)
                            .with_context(|| format!("Index {:?} not found", cursor_pos,))?
                            .push(dw.clone());
                    }
                }

                // Check candidate matches for continued match
                for (candidate_start_pos, candidate) in match_start_positions
                    .iter()
                    .flat_map(|(k, vs)| vs.iter().map(|&v| (k.clone(), v.clone())))
                {
                    let candidate_position_to_check = cursor_pos - candidate_start_pos;
                    let candidate_word = candidate.char_vec();
                    let current_range =
                        candidate_start_pos..candidate_start_pos + candidate_position_to_check;

                    let is_past_end_of_word = candidate_position_to_check >= candidate_word.len();
                    let still_matches = match is_past_end_of_word {
                        true => false,
                        false => char == candidate_word[candidate_position_to_check],
                    };

                    let is_complete_match = candidate_position_to_check == candidate_word.len() - 1;

                    let value = candidate.as_value();

                    match still_matches {
                        false => {
                            trace!(
                                candidate = value,
                                action = "DISCARD",
                                "{}",
                                format_text_span(text, current_range),
                            );
                        }
                        true => match is_complete_match {
                            true => {
                                // Found a digit-word

                                trace!(candidate = candidate.as_value(), "COMPLETE");
                                recognized_ranges.push(candidate_start_pos..cursor_pos);

                                let filtered_next: HashMap<usize, Vec<DigitWord>> =
                                    next_start_positions
                                        .clone()
                                        .iter()
                                        .map(|(k, vs)| (k.clone(), vs.clone()))
                                        .map(|(k, vs)| match k > cursor_pos {
                                            false => {
                                                for v in vs {
                                                    trace!(
                                                        "- OVERLAP {}: {}\n{}",
                                                        k,
                                                        v,
                                                        format_text_span(text, k..cursor_pos)
                                                    );
                                                }
                                                (k, Vec::new())
                                            }
                                            true => (k, vs),
                                        })
                                        .collect();

                                next_start_positions = filtered_next;

                                buf.push(candidate.int_value())
                            }
                            false => {
                                trace!(
                                    "- KEEP {}\n{}",
                                    candidate,
                                    format_text_span(text, current_range)
                                );
                            }
                        },
                    }
                }
            }
        }
        match_start_positions.clear();
        match_start_positions.extend(
            next_start_positions
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        span.exit();
    }
    Ok(buf)
}

fn create_start_positions(len: usize) -> HashMap<usize, Vec<DigitWord>> {
    let mut match_start_positions: HashMap<usize, Vec<DigitWord>> = HashMap::with_capacity(len);
    for i in 0..len {
        match_start_positions.insert(i, Vec::new());
    }
    match_start_positions
}

mod test {

    use crate::find_numbers::find_numbers;
    use paste::paste;

    macro_rules! test_find_numbers {
        ($($name:ident: $value:expr,)*) => {
            $(
                paste! {
                    #[test]
                    fn [<test_find_numbers_ $name>]() {
                        let (input, expected) = $value;
                        assert_eq!(expected, find_numbers(input).unwrap());
                    }
                }
            )*
        }
    }

    test_find_numbers! {
        a: ("two1nine", vec![2, 1, 9]),
        b: ("eightwothree", vec![8, 3]),
        c: ("abcone2threexyz", vec![1, 2, 3]),
        d: ("xtwone3four", vec![2, 3, 4]),
        e: ("4nineeightseven2", vec![4, 9, 8, 7, 2]),
        f: ("zoneight234", vec![1, 2, 3, 4]),
        g: ("7pqrstsixteen", vec![7, 6]),
    }
}
