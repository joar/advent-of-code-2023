use anyhow::{Context, Result};
use aoc2023lib::read_lines;
use day03_gear::run;

fn main() -> Result<()> {
    let lines: Vec<String> = read_lines("day03-gear/input")
        .context("Could not read input")?
        .collect::<Result<_, _>>()
        .context("Could not read line")?;
    pollster::block_on(run());
    println!("Hello, world!");
    Ok(())
}
