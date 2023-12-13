use std::collections::HashMap;
use std::ops::{RangeInclusive};

use anyhow::Context;
use tracing::{instrument, trace, trace_span};
use valuable::Valuable;

use crate::digit_word::DigitWord;
use crate::utils::format_text_span;

#[instrument(ret)]
pub fn find_numbers(text: &str) -> anyhow::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    let mut match_start_positions: HashMap<usize, Vec<DigitWord>> =
        create_start_positions(text.len());
    let mut next_start_positions: HashMap<usize, Vec<DigitWord>> =
        create_start_positions(text.len());

    let mut recognized_ranges: Vec<RangeInclusive<usize>> = Vec::new();

    for (cursor_pos, char) in text.chars().enumerate() {
        let span_match = trace_span!(
            "match",
            cursor_pos = cursor_pos,
            current = format_text_span(text, cursor_pos..=cursor_pos)
        )
        .entered();
        match char.is_numeric() {
            true => {
                next_start_positions =
                    drop_overlapping(text, &mut next_start_positions, cursor_pos);
                trace!(char = char.to_string(), "DIGIT");
                recognized_ranges.push(cursor_pos..=cursor_pos);
                buf.push(char.to_string().parse::<u8>()?);
            }
            false => {
                // Match first letter of all digit-words
                for dw in DigitWord::all() {
                    if dw.str_value().starts_with(char) {
                        trace!(
                            candidate = dw.as_value(),
                            range = format_text_span(text, cursor_pos..=cursor_pos),
                            "START"
                        );
                        next_start_positions
                            .get_mut(&cursor_pos)
                            .with_context(|| format!("Index {:?} not found", cursor_pos))?
                            .push(dw);
                    }
                }

                // Check candidate matches for continued match
                for (candidate_start_pos, candidate) in match_start_positions
                    .iter()
                    .flat_map(|(k, vs)| vs.iter().map(|&v| (*k, v)))
                {
                    let candidate_position_to_check = cursor_pos - candidate_start_pos;
                    let candidate_word = candidate.char_vec();
                    let current_range =
                        candidate_start_pos..=candidate_start_pos + candidate_position_to_check;

                    let is_past_end_of_word = candidate_position_to_check >= candidate_word.len();
                    let still_matches = match is_past_end_of_word {
                        true => false,
                        false => char == candidate_word[candidate_position_to_check],
                    };

                    let is_complete_match = candidate_position_to_check == candidate_word.len() - 1;

                    match still_matches {
                        false => {
                            trace!(
                                candidate = candidate.as_value(),
                                range = format_text_span(text, current_range),
                                "DISCARD"
                            );
                        }
                        true => match is_complete_match {
                            true => {
                                // Found a digit-word
                                let range = candidate_start_pos..=cursor_pos;

                                trace!(
                                    candidate = candidate.as_value(),
                                    range = format_text_span(text, range.clone()),
                                    "COMPLETE"
                                );
                                recognized_ranges.push(range);

                                next_start_positions =
                                    drop_overlapping(text, &mut next_start_positions, cursor_pos);

                                buf.push(candidate.int_value())
                            }
                            false => {
                                trace!(
                                    range = format_text_span(text, current_range),
                                    candidate = candidate.as_value(),
                                    "KEEP",
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
                .map(|(k, v)| (*k, v.clone())),
        );
        span_match.exit();
    }

    trace!(
        recognized = format!(
            "{}",
            recognized_ranges
                .iter()
                .map(|r| format_text_span(text, r.clone()))
                .collect::<Vec<String>>()
                .join(", ")
        ),
        "FOUND"
    );
    Ok(buf)
}

fn drop_overlapping(
    text: &str,
    next_start_positions: &mut HashMap<usize, Vec<DigitWord>>,
    cursor_pos: usize,
) -> HashMap<usize, Vec<DigitWord>> {
    let filtered_next: HashMap<usize, Vec<DigitWord>> = next_start_positions
        .clone()
        .iter()
        .map(|(k, vs)| (*k, vs.clone()))
        .map(|(k, vs)| match k > cursor_pos {
            false => {
                for v in vs {
                    trace!(
                        start_pos = k,
                        range = format_text_span(text, k..cursor_pos),
                        candidate = v.as_value(),
                        "DROP OVERLAP",
                    );
                }
                (k, Vec::new())
            }
            true => (k, vs),
        })
        .collect();
    filtered_next
}

fn create_start_positions(len: usize) -> HashMap<usize, Vec<DigitWord>> {
    let mut match_start_positions: HashMap<usize, Vec<DigitWord>> = HashMap::with_capacity(len);
    for i in 0..len {
        match_start_positions.insert(i, Vec::new());
    }
    match_start_positions
}

#[cfg(test)]
mod test {

    use crate::find_numbers::find_numbers;
    use crate::utils::maybe_init_logging;
    use ctor::ctor;
    use paste::paste;

    #[ctor]
    fn init() {
        maybe_init_logging();
    }

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
        h: ("7nineight", vec![7, 8]),
    }
}
