use std::collections::HashMap;

use anyhow::Context;
use tracing::{instrument, trace, trace_span};
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

use crate::calibration_digit::CalibrationDigit;
use crate::digit_word::DigitWord;
use crate::utils::format_text_span;

#[instrument(ret, level = "info")]
pub fn find_numbers(text: &str) -> anyhow::Result<Vec<u8>> {
    let mut match_start_positions: HashMap<usize, Vec<DigitWord>> =
        create_start_positions(text.len());
    let mut next_start_positions: HashMap<usize, Vec<DigitWord>> =
        create_start_positions(text.len());

    let mut calibration_digits: Vec<CalibrationDigit> = Vec::new();

    for (cursor_pos, char_at_cursor) in text.chars().enumerate() {
        let span_match = trace_span!(
            "match",
            cursor_pos = cursor_pos,
            current = format_text_span(text, cursor_pos..=cursor_pos)
        )
        .entered();
        match char_at_cursor.is_numeric() {
            true => {
                trace!(char = char_at_cursor.to_string(), "DIGIT");
                calibration_digits.push(CalibrationDigit::AsDigit {
                    value: char_at_cursor.to_string().parse::<u8>()?,
                    range: cursor_pos..=cursor_pos,
                });
            }
            false => {
                // Match first letter of all digit-words
                for dw in DigitWord::all() {
                    if dw.str_value().starts_with(char_at_cursor) {
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
                    let match_candidate =
                        MatchCandidate::from_digit_word(candidate_start_pos.clone(), &candidate);

                    match check_match(cursor_pos, &char_at_cursor, &match_candidate) {
                        MatchResult::Discard => {
                            trace!(
                                candidate = candidate.as_value(),
                                range = format_text_span(
                                    text,
                                    candidate_start_pos.clone()..=cursor_pos
                                ),
                                "DISCARD"
                            );
                        }
                        MatchResult::Complete(value) => {
                            let range = candidate_start_pos..=cursor_pos;
                            trace!(
                                candidate = candidate.as_value(),
                                range = format_text_span(text, range.clone()),
                                "COMPLETE"
                            );
                            calibration_digits.push(CalibrationDigit::AsWord {
                                value,
                                range: range.clone(),
                            });
                        }
                        MatchResult::Continue => {
                            trace!(
                                candidate = match_candidate.as_value(),
                                range = format_text_span(text, candidate_start_pos..=cursor_pos),
                                "CONTINUE",
                            );
                            next_start_positions
                                .get_mut(&match_candidate.start_pos)
                                .with_context(|| format!("Index {:?} not found", cursor_pos))?
                                .push(candidate);
                        }
                    }
                }
            }
        }
        match_start_positions.clear();
        match_start_positions.extend(next_start_positions.iter().map(|(k, v)| (*k, v.clone())));
        next_start_positions = create_start_positions(text.len());
        span_match.exit();
    }

    let calibration_digit_str = calibration_digits
        .iter()
        .map(|r| {
            let range = match r {
                CalibrationDigit::AsDigit { range, .. } => range,
                CalibrationDigit::AsWord { range, .. } => range,
            };

            format_text_span(text, range.clone())
        })
        .collect::<Vec<String>>()
        .join(", ");

    trace!(calibration_digits = calibration_digit_str, "FOUND");
    Ok(calibration_digits
        .iter()
        .map(|cd| match cd {
            CalibrationDigit::AsDigit { value, .. } => value.clone(),
            CalibrationDigit::AsWord { value, .. } => value.clone(),
        })
        .collect())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MatchCandidate {
    start_pos: usize,
    word: Vec<char>,
    value: u8,
}

impl MatchCandidate {
    pub fn new(start_pos: usize, word: &str, value: u8) -> Self {
        Self {
            start_pos,
            word: word.chars().collect(),
            value,
        }
    }
    pub fn from_digit_word(start_pos: usize, digit_word: &DigitWord) -> Self {
        Self {
            start_pos,
            word: digit_word.char_vec().clone(),
            value: digit_word.int_value().clone(),
        }
    }

    pub fn word_str(&self) -> String {
        String::from_iter(&self.word.clone())
    }
}

impl Valuable for MatchCandidate {
    fn as_value(&self) -> Value<'_> {
        Value::Structable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        visit.visit_named_fields(&NamedValues::new(
            &[
                NamedField::new("start_pos"),
                NamedField::new("word"),
                NamedField::new("value"),
            ],
            &[
                self.start_pos.as_value(),
                String::from_iter(&self.word).as_value(),
                self.value.as_value(),
            ],
        ))
    }
}

impl Structable for MatchCandidate {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_static("MatchCandidate", Fields::Named(&[]))
    }
}

#[derive(Debug, Eq, PartialEq)]
enum MatchResult {
    Discard,
    Complete(u8),
    Continue,
}

#[instrument(level = "trace", ret, skip(match_candidate), fields(word, word_pos))]
fn check_match(
    cursor_pos: usize,
    char_at_cursor: &char,
    match_candidate: &MatchCandidate,
) -> MatchResult {
    tracing::Span::current().record("word", match_candidate.word_str());
    let candidate_word = match_candidate.word.clone();
    let candidate_position_to_check = cursor_pos - match_candidate.start_pos;
    tracing::Span::current().record(
        "word_pos",
        format_text_span(
            match_candidate.word_str().as_str(),
            candidate_position_to_check..=candidate_position_to_check,
        ),
    );
    let is_past_end_of_word = candidate_position_to_check >= candidate_word.len();
    let still_matches = match is_past_end_of_word {
        true => false,
        false => *char_at_cursor == candidate_word[candidate_position_to_check],
    };

    let is_complete_match = candidate_position_to_check == candidate_word.len() - 1;

    match still_matches {
        false => MatchResult::Discard,
        true => match is_complete_match {
            true => {
                // Found a digit-word
                MatchResult::Complete(match_candidate.value)
            }
            false => MatchResult::Continue,
        },
    }
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
    use aoc2023lib::init_logging;
    use ctor::ctor;
    use paste::paste;

    use crate::find_numbers::{check_match, find_numbers, MatchCandidate, MatchResult};

    #[ctor]
    fn init() {
        init_logging();
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
        b: ("eightwothree", vec![8, 2, 3]),
        c: ("abcone2threexyz", vec![1, 2, 3]),
        d: ("xtwone3four", vec![2, 1, 3, 4]),
        e: ("4nineeightseven2", vec![4, 9, 8, 7, 2]),
        f: ("zoneight234", vec![1, 8, 2, 3, 4]),
        g: ("7pqrstsixteen", vec![7, 6]),
        h: ("7nineight", vec![7, 9, 8]),
    }

    macro_rules! test_check_match {
        ($($name:ident: $value:expr,)*) => {
            $(
                paste! {
                    #[test]
                    fn [<test_check_match_ $name>]() {
                        let (text, cursor_pos, match_candidate, expected) = $value;
                        let text_chars: Vec<char> = text.chars().collect();
                        let char_at_cursor = text_chars[cursor_pos];
                        assert_eq!(expected, check_match(cursor_pos, &char_at_cursor, &match_candidate));
                    }
                }
            )*
        }
    }

    test_check_match! {
        five_1: ("fiveight", 1, MatchCandidate::new(0, "five", 5), MatchResult::Continue),
        five_2: ("fiveight", 2, MatchCandidate::new(0, "five", 5), MatchResult::Continue),
        five_3: ("fiveight", 3, MatchCandidate::new(0, "five", 5), MatchResult::Complete(5)),
        eight_1: ("fiveight", 4, MatchCandidate::new(3, "eight", 8), MatchResult::Continue),
        eight_2: ("fiveight", 5, MatchCandidate::new(3, "eight", 8), MatchResult::Continue),
        eight_3: ("fiveight", 6, MatchCandidate::new(3, "eight", 8), MatchResult::Continue),
        eight_4: ("fiveight", 7, MatchCandidate::new(3, "eight", 8), MatchResult::Complete(8)),
    }

    #[test]
    fn test_find_numbers_fourzqlhcjksixthreejrl9() {
        assert_eq!(
            vec![4, 6, 3, 9],
            find_numbers("fourzqlhcjksixthreejrl9").unwrap()
        )
    }
}
