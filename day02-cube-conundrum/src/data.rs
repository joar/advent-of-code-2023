use std::cell::OnceCell;

use anyhow::{anyhow, Context, Result};
use pest::iterators::Pair;
use pest::Parser;

use crate::parser::{GamesParser, Rule};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Game {
    pub number: u32,
    pub sets: Vec<Stats>,
}

impl Game {
    pub fn parse(input: &str) -> Result<Self> {
        let mut pairs = GamesParser::parse(Rule::game, input)?;

        let game = pairs.next().context("Unable to parse first token")?;

        let number: OnceCell<u32> = OnceCell::new();
        let mut sets: Vec<Stats> = Vec::new();

        match game.as_rule() {
            Rule::game => {
                let mut inner = game.into_inner();
                let game_number_token = inner.next().context("Unable to find game_number")?;
                match game_number_token.as_rule() {
                    Rule::game_number => number
                        .set(game_number_token.as_str().parse::<u32>()?)
                        .unwrap(),
                    _ => unreachable!(),
                };

                let sets_of_cube_draws_token = inner.next().context("No sets_of_cube_draws")?;
                match sets_of_cube_draws_token.as_rule() {
                    Rule::sets_of_cube_draws => {
                        let inner = sets_of_cube_draws_token.into_inner();

                        for cube_draw in inner {
                            match cube_draw.as_rule() {
                                Rule::cube_draw => {
                                    sets.push(Stats::parse(cube_draw)?);
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    _ => unreachable!(),
                };
            }
            _ => unreachable!(),
        }

        Ok(Self {
            number: *number.get().unwrap(),
            sets,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Stats {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
}

impl Stats {
    /// Parse a [Rule::cube_draw]
    fn parse(cube_draw: Pair<Rule>) -> Result<Self> {
        let red: OnceCell<u32> = OnceCell::new();
        let green: OnceCell<u32> = OnceCell::new();
        let blue: OnceCell<u32> = OnceCell::new();
        let x: Result<()> = match cube_draw.as_rule() {
            Rule::cube_draw => {
                for pair in cube_draw.into_inner() {
                    match pair.as_rule() {
                        Rule::amount_and_color => {
                            let mut amount_and_color_inner = pair.into_inner();
                            let cube_amount = amount_and_color_inner
                                .find(|r| match r.as_rule() {
                                    Rule::cube_amount => true,
                                    _ => false,
                                })
                                .context("No cube_amount found")?
                                .as_str()
                                .parse::<u32>()?;
                            let color = amount_and_color_inner
                                .find(|r| match r.as_rule() {
                                    Rule::cube_color => true,
                                    _ => false,
                                })
                                .context("No cube_color token found")?;
                            let specific_color_token = color
                                .into_inner()
                                .find(|r| match r.as_rule() {
                                    Rule::red => true,
                                    Rule::green => true,
                                    Rule::blue => true,
                                    _ => false,
                                })
                                .context("No red/green/blue token found")?;

                            match specific_color_token.as_rule() {
                                Rule::red => {
                                    red.set(cube_amount).unwrap();
                                    Ok(())
                                }
                                Rule::green => {
                                    green.set(cube_amount).unwrap();
                                    Ok(())
                                }
                                Rule::blue => {
                                    blue.set(cube_amount).unwrap();
                                    Ok(())
                                }
                                _ => Err(anyhow!("Unable to match specific color")),
                            }
                        }
                        _ => Err(anyhow!("No match for amount_and_color")),
                    }?
                }
                Ok(())
            }
            _ => Err(anyhow!(
                "Expected a cube_draw, got {:?}",
                cube_draw.as_rule()
            )),
        };

        x?;

        Ok(Self {
            red: *red.get().unwrap_or(&0),
            green: *green.get().unwrap_or(&0),
            blue: *blue.get().unwrap_or(&0),
        })
    }
}

#[cfg(test)]
mod test {
    use pest::Parser;

    use crate::data::{Game, Stats};
    use crate::parser::{GamesParser, Rule};

    #[test]
    fn parse_stats() {
        let input = "7 blue, 6 green, 3 red";
        let actual = Stats::parse(
            GamesParser::parse(Rule::cube_draw, input)
                .unwrap()
                .find(|r| r.as_rule() == Rule::cube_draw)
                .unwrap(),
        )
        .unwrap();

        assert_eq!(
            actual,
            Stats {
                red: 3,
                green: 6,
                blue: 7,
            }
        )
    }

    #[test]
    fn parse_game() {
        let input = "Game 1: 7 blue, 6 green, 3 red; 3 red, 5 green, 1 blue; 1 red, 5 green, 8 blue; 3 red, 1 green, 5 blue";
        let actual = Game::parse(input).unwrap();
        assert_eq!(
            actual,
            Game {
                number: 1,
                sets: vec![
                    Stats {
                        red: 3,
                        green: 6,
                        blue: 7,
                    },
                    Stats {
                        red: 3,
                        green: 5,
                        blue: 1
                    },
                    Stats {
                        red: 1,
                        green: 5,
                        blue: 8
                    },
                    Stats {
                        red: 3,
                        green: 1,
                        blue: 5
                    }
                ],
            }
        )
    }
}
