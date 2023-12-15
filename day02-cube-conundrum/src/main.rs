use anyhow::Result;
use pest::Parser;

use crate::data::Game;
use aoc2023lib::read_lines;

use crate::parser::{GamesParser, Rule};

mod parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "grammar.pest"]
    pub struct GamesParser;
}

mod data;

fn main() -> Result<()> {
    let lines = read_lines("day02-cube-conundrum/input")?;

    let mut games: Vec<Game> = Vec::new();

    for line in lines {
        let line_str = line?;
        games.push(Game::parse(line_str.as_str())?);
    }

    // 12 red cubes, 13 green cubes, and 14 blue cubes

    let max_red: u32 = 12;
    let max_green: u32 = 13;
    let max_blue: u32 = 14;

    let possible_games: Vec<&Game> = games
        .iter()
        .filter(|&game| {
            !game.sets.iter().any(|&stats| {
                stats.red > max_red || stats.green > max_green || stats.blue > max_blue
            })
        })
        .collect();
    let sum_of_possible_game_numbers: u32 = possible_games.iter().map(|&game| game.number).sum();
    for g in possible_games.clone() {
        eprintln!("{:?}", g);
    }
    println!(
        "Found {} possible games with sum: {}",
        possible_games.len(),
        sum_of_possible_game_numbers
    );
    Ok(())
}

#[cfg(test)]
mod test {
    use pest::Parser;

    use crate::parser::{GamesParser, Rule};

    #[test]
    fn parse_game() {
        let input = "Game 1: 7 blue, 6 green, 3 red; 3 red, 5 green, 1 blue; 1 red, 5 green, 8 blue; 3 red, 1 green, 5 blue";
        dbg!(GamesParser::parse(Rule::game, input).unwrap());
    }

    #[test]
    fn parse_games() {
        let input = "Game 1: 7 blue, 6 green, 3 red; 3 red, 5 green, 1 blue; 1 red, 5 green, 8 blue; 3 red, 1 green, 5 blue\n\
        Game 2: 9 green, 1 blue, 12 red; 1 blue, 18 green, 8 red; 2 blue, 6 green, 13 red; 3 blue, 13 red, 7 green; 5 blue, 4 red, 4 green; 6 blue, 7 green, 4 red";
        dbg!(GamesParser::parse(Rule::games, input).unwrap());
    }

    #[test]
    fn parse_cube_draw() {
        let input = "7 blue, 6 green, 3 red";
        dbg!(GamesParser::parse(Rule::cube_draw, input).unwrap());
    }

    #[test]
    fn parse_sets_of_cube_draws() {
        let input = "7 blue, 6 green, 3 red; 1 blue, 2 red, 0 green";
        dbg!(GamesParser::parse(Rule::sets_of_cube_draws, input).unwrap());
    }

    #[test]
    fn parse_cube_color() {
        for color in vec!["red", "green", "blue"] {
            dbg!(color);
            GamesParser::parse(Rule::cube_color, color).unwrap();
        }
    }
}
