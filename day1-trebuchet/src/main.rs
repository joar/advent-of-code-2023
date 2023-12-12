extern crate core;

use std::iter::Iterator;
use std::string::ToString;

use anyhow::Result;
use find_numbers::find_numbers;
use tracing::{debug, instrument};

pub mod digit_word;
mod find_numbers;
pub mod utils;

use crate::utils::read_lines;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let lines: Vec<String> = read_lines("input")
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

#[instrument]
pub fn extract_calibration_value(lines: Vec<String>) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| {
            let numbers = find_numbers(line).unwrap();

            debug!(line = line, "{:?}", numbers.clone());

            match numbers.as_slice() {
                &[a] => {
                    let res = Some([a, a]);
                    res
                }
                &[a, .., b] => {
                    let res = Some([a, b]);
                    res
                }
                _ => None,
            }
            .map(|pair| String::from_iter(pair.map(|c| c.to_string())))
        })
        .collect()
}
