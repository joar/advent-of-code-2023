use std::fs::read_to_string;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

use crate::data::Outcome;
use crate::parse::{parse_input, parse_input_part_two};

mod data {

    use std::time::Duration;

    use anyhow::{anyhow, Result};
    use tracing::instrument;

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub struct Race {
        time_allowed: Duration,
        best_distance_millimeters: usize,
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum Outcome {
        Win,
        Loss,
    }

    impl Race {
        pub fn time_allowed(&self) -> &Duration {
            &self.time_allowed
        }

        pub fn from_ms_and_mm(time_allowed_ms: usize, best_distance_millimeters: usize) -> Self {
            Self {
                time_allowed: Duration::from_millis(time_allowed_ms as u64),
                best_distance_millimeters,
            }
        }

        #[instrument(ret)]
        pub fn number_of_ways_to_beat(&self) -> Result<usize> {
            let attempts = (0..self.time_allowed.as_millis() as usize)
                .map(|i| self.compete(i))
                .collect::<Result<Vec<_>>>()?;
            let winning_attempts = attempts
                .iter()
                .filter_map(|outcome: &Outcome| match outcome {
                    Outcome::Win => Some(1),
                    Outcome::Loss => None,
                })
                .collect::<Vec<_>>();
            tracing::Span::current().record("winning_attempts", format!("{:?}", attempts));
            Ok(winning_attempts.len())
        }

        #[instrument(ret)]
        pub fn compete(&self, button_hold_time_ms: usize) -> Result<Outcome> {
            let distance = self.run(button_hold_time_ms)?;
            Ok(match distance > self.best_distance_millimeters {
                true => Outcome::Win,
                false => Outcome::Loss,
            })
        }

        #[instrument(ret)]
        pub fn run(&self, button_hold_time_ms: usize) -> Result<usize> {
            let hold_time = Duration::from_millis(button_hold_time_ms as u64);
            if hold_time > self.time_allowed {
                return Err(anyhow!(
                    "Can't hold button for more than {:?}, held for {:?}",
                    self.time_allowed,
                    hold_time
                ));
            }
            let run_time = self.time_allowed - hold_time;
            let mm_per_ms = button_hold_time_ms;

            let distance = mm_per_ms * run_time.as_millis() as usize;
            Ok(distance)
        }
    }

    #[cfg(test)]
    mod test {
        use anyhow::Result;
        use ctor::ctor;
        use uom::num_rational::Ratio;
        use uom::si::length::{millimeter, nanometer};
        use uom::si::rational::Length;

        use aoc2023lib::init_logging;

        use crate::data::Race;

        #[ctor]
        fn init() {
            init_logging();
        }

        #[test]
        fn test_uom_sanity() {
            let a = Length::new::<millimeter>(Ratio::new(1, 1));
            dbg!(a.value, a.dimension, a.units);
            assert_eq!(a.get::<nanometer>(), Ratio::new(1000000, 1));
        }

        #[test]
        fn test_attempt() {
            let race = Race::from_ms_and_mm(7, 9);
            let actual = (0..=7)
                .map(|i| race.run(i))
                .collect::<Result<Vec<_>>>()
                .unwrap();
            let expected = vec![0, 6, 10, 12, 12, 10, 6, 0];
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_number_of_ways_to_beat() {
            let races = [Race::from_ms_and_mm(7, 9),
                Race::from_ms_and_mm(15, 40),
                Race::from_ms_and_mm(30, 200)];
            let actual = races
                .iter()
                .map(|race| race.number_of_ways_to_beat())
                .collect::<Result<Vec<_>>>()
                .unwrap();
            let expected = vec![4, 8, 9];
            assert_eq!(actual, expected);
        }
    }
}

mod parse {
    use std::iter::zip;

    use anyhow::{anyhow, Result};

    use crate::data::Race;

    pub fn parse_input(input: &str) -> Result<Vec<Race>> {
        let (times, distances) = parse_times_and_distances(input)?;

        Ok(zip(times, distances)
            .map(|(time, distance)| Race::from_ms_and_mm(time, distance))
            .collect())
    }

    fn parse_times_and_distances(input: &str) -> Result<(Vec<usize>, Vec<usize>)> {
        let lines: Vec<&str> = input.lines().collect();
        let times: Vec<_> = match lines.first() {
            Some(&times_line) => {
                if times_line.starts_with("Time: ") {
                    let times: Result<Vec<_>> = times_line
                        .split(' ')
                        .filter_map(|s| match s {
                            "Time:" => None,
                            "" => None,
                            s => Some(s.parse::<usize>().map_err(|err| err.into())),
                        })
                        .collect::<Result<Vec<_>>>();
                    times
                } else {
                    Err(anyhow!(
                        "Expected line to start with 'Time: ': {:?}",
                        times_line
                    ))
                }
            }
            None => Err(anyhow!("Could not get first line from input {:?}", input)),
        }?;
        let distances: Vec<_> = match lines.get(1) {
            Some(&distances_line) => {
                if distances_line.starts_with("Distance: ") {
                    let times: Result<Vec<_>> = distances_line
                        .split(' ')
                        .filter_map(|s| match s {
                            "Distance:" => None,
                            "" => None,
                            s => Some(s.parse::<usize>().map_err(|err| err.into())),
                        })
                        .collect::<Result<Vec<_>>>();
                    times
                } else {
                    Err(anyhow!(
                        "Expected line to start with 'Distance: ': {:?}",
                        distances_line
                    ))
                }
            }
            None => Err(anyhow!("Could not get second line from input {:?}", input)),
        }?;
        Ok((times, distances))
    }

    pub fn parse_input_part_two(input: &str) -> Result<Race> {
        let (times, distances) = parse_times_and_distances(input)?;
        let time = times
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("")
            .parse::<usize>()?;
        let distance = distances
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("")
            .parse::<usize>()?;
        Ok(Race::from_ms_and_mm(time, distance))
    }

    #[cfg(test)]
    mod test {
        use crate::data::Race;
        use crate::parse::parse_input;

        #[test]
        fn test_parse_input() {
            let input = "Time:      7  15   30
Distance:  9  40  200";
            let actual = parse_input(input).unwrap();
            assert_eq!(
                actual,
                vec![
                    Race::from_ms_and_mm(7, 9),
                    Race::from_ms_and_mm(15, 40),
                    Race::from_ms_and_mm(30, 200)
                ]
            )
        }
    }
}

fn main() -> Result<()> {
    let input = read_to_string("day06-wait/input")?;

    // Part one
    {
        let races = parse_input(input.as_str())?;
        println!("{:?}", races);

        let ways_to_beat = races
            .iter()
            .map(|r| r.number_of_ways_to_beat())
            .collect::<Result<Vec<_>>>()?;
        println!("ways to beat: {:?}", ways_to_beat);
        let answer = ways_to_beat
            .iter()
            .fold(None, |acc, &x| match acc {
                None => Some(x),
                Some(y) => Some(y * x),
            })
            .unwrap();
        println!("answer: {}", answer);
    }

    // Part two
    {
        let race = parse_input_part_two(input.as_str())?;
        println!("race: {:?}", race);

        let time_allowed_ms = race.time_allowed().as_millis() as usize;
        let style = ProgressStyle::with_template(
            "[{elapsed_precise}]  {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;

        let mut first_win_idx: Option<usize> = None;
        let mut last_win_idx: Option<usize> = None;

        {
            let progress = ProgressBar::new(time_allowed_ms as u64)
                .with_style(style.clone())
                .with_message("find first win from start");
            for start_idx in (0..=time_allowed_ms).progress_with(progress) {
                match race.compete(start_idx)? {
                    Outcome::Win => {
                        first_win_idx = Some(start_idx);
                        break;
                    }
                    Outcome::Loss => {}
                }
            }

            println!("first win idx: {:?}", first_win_idx);
        }
        {
            let progress = ProgressBar::new(time_allowed_ms as u64)
                .with_style(style)
                .with_message("find last win from end");
            for end_idx in (0..=time_allowed_ms).rev().progress_with(progress) {
                match race.compete(end_idx)? {
                    Outcome::Win => {
                        last_win_idx = Some(end_idx);
                        break;
                    }
                    Outcome::Loss => {}
                }
            }
            println!("last win idx: {:?}", last_win_idx);
        }

        let num_wins = last_win_idx.context("Never won from the end").unwrap()
            - first_win_idx
                .context("Never won from the beginning")
                .unwrap()
            // Hmm, good old off-by-one
            //   first
            //   |   last
            //   |   |
            // 0 1 2 3
            // -------
            // 3 - 1     = 2 ⚠️
            // ...
            // 3 - 1 + 1 = 3 🎉
            + 1;
        println!("number of wins: {}", num_wins);
    }

    Ok(())
}
