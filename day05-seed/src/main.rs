use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;

use anyhow::{anyhow, Context as AnyhowContext, Result};
use tracing::{instrument, trace, trace_span};
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

use aoc2023lib::{init_logging, read_lines};

fn main() -> Result<()> {
    init_logging();
    let seed_context = parse_input(
        read_to_string("day05-seed/input")
            .context("Could not read string")?
            .as_str(),
    )
    .context("Could not parse input")?;

    let locations: Vec<usize> = seed_context
        .seeds
        .iter()
        .map(|location| Ok(seed_context.resolve_location(location.clone())?))
        .collect::<Result<Vec<_>>>()?;

    let closest_location = locations
        .iter()
        .min()
        .context("Could not get closest location")?;
    println!("Closest location: {}", closest_location);
    Ok(())
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Valuable)]
struct Redirect {
    destination_range_start: usize,
    source_range_start: usize,
    range_length: usize,
}

impl Redirect {
    pub fn new(
        destination_range_start: usize,
        source_range_start: usize,
        range_length: usize,
    ) -> Self {
        Self {
            destination_range_start,
            source_range_start,
            range_length,
        }
    }

    pub fn contains(&self, source_location: &usize) -> bool {
        (self.source_range_start..(self.source_range_start + self.range_length))
            .contains(source_location)
    }

    pub fn resolve(&self, source_location: usize) -> usize {
        if self.contains(&source_location) {
            self.destination_range_start + source_location - self.source_range_start
        } else {
            source_location
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Redirects {
    source: String,
    destination: String,
    redirects: RefCell<Vec<Redirect>>,
}

impl Redirects {
    pub fn new(source: &str, destination: &str, redirects: Vec<Redirect>) -> Self {
        Self {
            source: source.to_string(),
            destination: destination.to_string(),
            redirects: RefCell::new(redirects),
        }
    }

    #[instrument(ret, skip(self))]
    pub fn resolve(&self, source_location: usize) -> usize {
        if let Some(redirect) = self
            .redirects
            .borrow()
            .iter()
            .find(|r| r.contains(&source_location))
        {
            trace!(
                source_location = source_location,
                redirect = redirect.as_value(),
                "found redirect"
            );
            redirect.resolve(source_location)
        } else {
            source_location
        }
    }
}

impl Valuable for Redirects {
    fn as_value(&self) -> Value<'_> {
        Value::Structable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        visit.visit_named_fields(&NamedValues::new(
            &[
                NamedField::new("source"),
                NamedField::new("destination"),
                NamedField::new("redirects"),
            ],
            &[
                self.source.as_value(),
                self.destination.as_value(),
                self.redirects.borrow().as_value(),
            ],
        ));
    }
}

impl Structable for Redirects {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_static("Redirects", Fields::Named(&[]))
    }
}

impl Redirects {
    #[instrument]
    fn add(&self, redirect: Redirect) {
        self.redirects.borrow_mut().push(redirect)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Context {
    seeds: Vec<usize>,
    redirects_by_source: HashMap<String, Redirects>,
}

impl Context {
    #[instrument(ret, skip(self))]
    pub fn resolve_location(&self, seed_location: usize) -> Result<usize> {
        let mut next: &str = "seed";
        let mut location: usize = seed_location;
        while next != "location" {
            trace!("{} {}", next, location);
            let redirects = self
                .redirects_by_source
                .get(next)
                .with_context(|| format!("No redirects with source {}", next))?;
            let prev_location = location.clone();
            location = redirects.resolve(prev_location);
            // trace!(
            //     prev_location = prev_location,
            //     next_location = location,
            //     source = next,
            //     destination = redirects.destination,
            //     "{} {}"
            // );
            next = redirects.destination.as_str();
        }
        Ok(location)
    }
}

#[instrument(ret, skip(text))]
fn parse_input(text: &str) -> Result<Context> {
    let lines: Vec<String> = text
        .lines()
        .into_iter()
        .map(str::to_string)
        .to_owned()
        .collect::<Vec<_>>();

    let seeds_re = regex::Regex::new(r#"^seeds: (?<numbers>(\d+ )*\d+)$"#)?;
    let map_re = regex::Regex::new(r#"^(?<source>[^-]+)-to-(?<destination>[^-]+) map:$"#)?;
    let redirect_re = regex::Regex::new(
        r#"^(?<destination_range_start>\d+) (?<source_range_start>\d+) (?<range_length>\d+)$"#,
    )?;

    let mut redirects_by_source: HashMap<String, Redirects> = HashMap::new();
    let mut current_redirects: Option<Redirects> = None;
    let mut seeds: Option<Vec<usize>> = None;

    for (line_number, line) in lines.iter().enumerate() {
        let span = trace_span!("line", line_number = line_number).entered();
        if let Some(captures) = seeds_re.captures(line.as_ref()) {
            seeds = Some(
                captures["numbers"]
                    .split(" ")
                    .map(|number_str| Ok(number_str.parse::<usize>()?))
                    .collect::<Result<Vec<_>>>()?,
            );
        } else if let Some(captures) = map_re.captures(line.as_ref()) {
            if let Some(mut redirects) = current_redirects {
                current_redirects = None;
                trace!(
                    redirects = redirects.as_value(),
                    "closing redirects section"
                );
                if redirects_by_source.contains_key(&redirects.source) {
                    return Err(anyhow!(
                        "Multiple redirects for the same source: {}",
                        &redirects.source
                    ));
                }
                redirects_by_source.insert(redirects.source.clone(), redirects);
            }
            trace!(line = line, "redirects section");
            current_redirects = Some(Redirects {
                source: captures["source"].to_string(),
                destination: captures["destination"].to_string(),
                redirects: RefCell::new(vec![]),
            });
        } else if let Some(captures) = redirect_re.captures(line.as_ref()) {
            let redirect = Redirect {
                destination_range_start: captures["destination_range_start"].parse()?,
                source_range_start: captures["source_range_start"].parse()?,
                range_length: captures["range_length"].parse()?,
            };
            current_redirects = match current_redirects {
                Some(redirects) => {
                    redirects.add(redirect);
                    Ok(Some(redirects))
                }
                None => Err(anyhow!("No current redirects")),
            }?;
        } else if !line.trim().is_empty() {
            return Err(anyhow!("Could not parse line #{}: {}", line_number, line));
        }
        span.exit();
    }
    if let Some(mut redirects) = current_redirects {
        current_redirects = None;
        trace!(
            redirects = redirects.as_value(),
            "closing redirects section"
        );
        if redirects_by_source.contains_key(&redirects.source) {
            return Err(anyhow!(
                "Multiple redirects for the same source: {}",
                &redirects.source
            ));
        }
        redirects_by_source.insert(redirects.source.clone(), redirects);
    }
    Ok(Context {
        seeds: seeds.context("No seeds")?,
        redirects_by_source: redirects_by_source.into_iter().collect(),
    })
}

#[cfg(test)]
mod test {
    use ctor::ctor;
    use std::cell::RefCell;
    use std::collections::HashSet;

    use aoc2023lib::init_logging;

    use crate::{parse_input, Redirect, Redirects};

    #[ctor]
    fn init() {
        init_logging();
    }

    const test_input: &'static str = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

    #[test]
    fn test_parse_input() {
        let actual = parse_input(test_input).unwrap();
        assert_eq!(actual.seeds, vec![79, 14, 55, 13]);
        let sources: HashSet<String> = actual
            .redirects_by_source
            .keys()
            .map(|x| x.clone())
            .collect();
        assert_eq!(
            sources,
            HashSet::from_iter(
                vec![
                    "seed",
                    "soil",
                    "fertilizer",
                    "water",
                    "light",
                    "temperature",
                    "humidity"
                ]
                .into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
            )
        );
        assert_eq!(
            actual
                .redirects_by_source
                .get("humidity")
                .unwrap()
                .redirects,
            RefCell::new(vec![Redirect::new(60, 56, 37), Redirect::new(56, 93, 4)])
        );
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
                redirects.resolve(97),
                redirects.resolve(98),
                redirects.resolve(99),
                redirects.resolve(100),
                redirects.resolve(1),
                redirects.resolve(2),
                redirects.resolve(3),
            ],
            vec![97, 50, 51, 100, 1, 30, 3]
        );
    }

    #[test]
    fn test_seed_context_resolve() {
        let seed_context = parse_input(test_input).unwrap();

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
