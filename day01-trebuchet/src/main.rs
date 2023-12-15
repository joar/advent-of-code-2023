extern crate core;

use std::iter::Iterator;
use std::string::ToString;

use anyhow::Result;
use find_numbers::find_numbers;
use tracing::instrument;

pub mod calibration_digit;
pub mod digit_word;
mod find_numbers;
pub mod utils;

use aoc2023lib::{maybe_init_logging, read_lines};

fn main() -> Result<()> {
    color_backtrace::install();
    maybe_init_logging();
    let lines: Vec<String> = read_lines("../input")
        .expect("Error reading file")
        .collect::<Result<_, _>>()
        .expect("Error reading lines");
    let x = extract_calibration_value(lines);

    let sum: u64 = x
        .iter()
        .map(|l| l.parse::<u64>().expect("Could not parse int"))
        .sum();
    println!("sum: {}", sum);
    Ok(())
}

#[instrument(skip(lines))]
pub fn extract_calibration_value(lines: Vec<String>) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| {
            let numbers = find_numbers(line).unwrap();
            extract_two_digit_number(numbers)
        })
        .collect()
}

#[instrument(ret)]
fn extract_two_digit_number(numbers: Vec<u8>) -> Option<String> {
    match *numbers.as_slice() {
        [a] => Some([a, a]),
        [a, .., b] => Some([a, b]),
        _ => None,
    }
    .map(|pair| String::from_iter(pair.map(|c| c.to_string())))
}
