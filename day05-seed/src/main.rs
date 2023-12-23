use std::fs::read_to_string;

use anyhow::{Context as AnyhowContext, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use si_scale::helpers::number_;

use aoc2023lib::init_logging;

mod models;
mod parse;

fn main() -> Result<()> {
    init_logging();

    // Part 1
    {
        let seed_context = parse::parse_input(
            read_to_string("day05-seed/input")
                .context("Could not read string")?
                .as_str(),
        )
        .context("Could not parse input")?;
        let locations: Vec<usize> = seed_context
            .seeds()
            .iter()
            .map(|location| seed_context.resolve_location(*location))
            .collect::<Result<Vec<_>>>()?;

        let closest_location = locations
            .iter()
            .min()
            .context("Could not get closest location")?;
        println!("Closest location: {}", closest_location);
    };

    // Part 2
    {
        let seed_context = parse::parse_input(
            read_to_string("day05-seed/input")
                .context("Could not read string")?
                .as_str(),
        )
        .context("Could not parse input")?;
        let ranges: Vec<(usize, usize)> = seed_context
            .seeds()
            .chunks(2)
            .map(|x| match x {
                [l, r] => (*l, *r),
                other => panic!("Unexpected chunk: {:?}", other),
            })
            .collect::<Vec<(usize, usize)>>();

        let total_range_length: usize = ranges.iter().map(|(_, len)| len).sum();
        println!("Total range length: {}", number_(total_range_length as f64));

        let style = ProgressStyle::with_template(
            "[{elapsed_precise} ETA {eta_precise}]  {bar:40.cyan/blue} {pos:>7}/{len:7} {per_sec} {msg}",
        )?;

        let progress = ProgressBar::new(total_range_length as u64).with_style(style);

        let closest_location: usize = ranges
            .par_iter()
            .flat_map(|(range_start, range_length)| {
                *range_start..(range_start + range_length)
            })
            .map(|location| {
                progress.inc(1);
                seed_context
                    .resolve_location(location)
                    .with_context(|| format!("Could not resolve location {}", location))
                    .unwrap()
            })
            .min()
            .context("Could not get closest location")?;
        println!("Part 2 closest location: {}", closest_location);
    };

    Ok(())
}

#[cfg(test)]
mod test {
    use ctor::ctor;

    use aoc2023lib::init_logging;

    use crate::models::{Redirect, Redirects};
    use crate::parse::parse_input;
    use crate::parse::test::TEST_INPUT;

    #[ctor]
    fn init() {
        init_logging();
    }

    #[test]
    fn test_modulo() {
        assert_eq!(10 % 10, 0);
        assert_eq!(5, 5);
    }

    #[test]
    fn test_redirect_resolve() {
        let redirect = Redirect::new(50, 98, 2);

        assert_eq!(
            vec![
                redirect.resolve(97),
                redirect.resolve(98),
                redirect.resolve(99),
                redirect.resolve(100)
            ],
            vec![97, 50, 51, 100]
        );
    }

    #[test]
    fn test_redirects_resolve() {
        let redirects = Redirects::new(
            "source",
            "destination",
            vec![Redirect::new(50, 98, 2), Redirect::new(30, 2, 1)],
        );

        assert_eq!(
            vec![
                redirects.resolve(97).unwrap(),
                redirects.resolve(98).unwrap(),
                redirects.resolve(99).unwrap(),
                redirects.resolve(100).unwrap(),
                redirects.resolve(1).unwrap(),
                redirects.resolve(2).unwrap(),
                redirects.resolve(3).unwrap(),
            ],
            vec![97, 50, 51, 100, 1, 30, 3]
        );
    }

    #[test]
    fn test_seed_context_resolve() {
        let seed_context = parse_input(TEST_INPUT).unwrap();

        assert_eq!(
            vec![
                seed_context.resolve_location(79).unwrap(),
                seed_context.resolve_location(14).unwrap(),
                seed_context.resolve_location(55).unwrap(),
                seed_context.resolve_location(13).unwrap(),
            ],
            vec![82, 43, 86, 35]
        );
    }
}
