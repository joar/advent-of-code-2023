use std::cell::RefCell;
use std::collections::HashMap;

use anyhow::{anyhow, Context};
use tracing::{instrument, trace, trace_span};
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

use crate::models::{Redirect, Redirects, SowingContext};

#[derive(Debug)]
struct RedirectsMut {
    source: String,
    destination: String,
    redirects: RefCell<Vec<Redirect>>,
}

impl RedirectsMut {
    pub fn redirects(&self) -> Vec<Redirect> {
        self.redirects.borrow().clone()
    }

    pub fn to_redirects(self) -> Redirects {
        Redirects::new(
            self.source.as_str(),
            self.destination.as_str(),
            self.redirects().clone(),
        )
    }
}

impl Structable for RedirectsMut {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_static("RedirectsMut", Fields::Named(&[]))
    }
}

impl Valuable for RedirectsMut {
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
                self.redirects().as_value(),
            ],
        ));
    }
}

impl RedirectsMut {
    pub fn new(source: &str, destination: &str, redirects: Vec<Redirect>) -> Self {
        Self {
            source: source.to_string(),
            destination: destination.to_string(),
            redirects: RefCell::new(redirects),
        }
    }
    pub fn add(&self, redirect: Redirect) -> anyhow::Result<()> {
        let mut inner = self.redirects.borrow_mut();
        inner.push(redirect);
        Ok(())
    }
}

#[instrument(ret, skip(text))]
pub fn parse_input(text: &str) -> anyhow::Result<SowingContext> {
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
    let mut current_redirects: Option<RedirectsMut> = None;
    let mut seeds: Option<Vec<usize>> = None;

    for (line_number, line) in lines.iter().enumerate() {
        let span = trace_span!("line", line_number = line_number).entered();
        if let Some(captures) = seeds_re.captures(line.as_ref()) {
            seeds = Some(
                captures["numbers"]
                    .split(" ")
                    .map(|number_str| Ok(number_str.parse::<usize>()?))
                    .collect::<anyhow::Result<Vec<_>>>()?,
            );
        } else if let Some(captures) = map_re.captures(line.as_ref()) {
            if let Some(redirects_mut) = current_redirects {
                trace!(
                    redirects = redirects_mut.as_value(),
                    "closing redirects section"
                );
                if redirects_by_source.contains_key(&redirects_mut.source) {
                    return Err(anyhow!(
                        "Multiple redirects for the same source: {}",
                        &redirects_mut.source
                    ));
                }
                redirects_by_source
                    .insert(redirects_mut.source.clone(), redirects_mut.to_redirects());
            }
            trace!(line = line, "redirects section");
            current_redirects = Some(RedirectsMut::new(
                &captures["source"],
                &captures["destination"],
                vec![],
            ));
        } else if let Some(captures) = redirect_re.captures(line.as_ref()) {
            let redirect = Redirect::new(
                captures["destination_range_start"].parse()?,
                captures["source_range_start"].parse()?,
                captures["range_length"].parse()?,
            );
            current_redirects = match current_redirects {
                Some(redirects) => {
                    redirects.add(redirect)?;
                    Ok(Some(redirects))
                }
                None => Err(anyhow!("No current redirects")),
            }?;
        } else if !line.trim().is_empty() {
            return Err(anyhow!("Could not parse line #{}: {}", line_number, line));
        }
        span.exit();
    }
    if let Some(redirects) = current_redirects {
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
        redirects_by_source.insert(redirects.source.clone(), redirects.to_redirects());
    }
    Ok(SowingContext::new(
        seeds.context("No seeds")?,
        redirects_by_source.into_iter().collect(),
    ))
}
#[cfg(test)]
pub mod test {
    use std::collections::HashSet;

    use crate::models::Redirect;
    use crate::parse::parse_input;

    pub const TEST_INPUT: &'static str = "seeds: 79 14 55 13

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
        let actual = parse_input(TEST_INPUT).unwrap();
        assert_eq!(actual.seeds().clone(), vec![79, 14, 55, 13]);
        let sources: HashSet<String> = actual
            .redirects_by_source()
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
                .redirects_by_source()
                .get("humidity")
                .unwrap()
                .redirects(),
            vec![Redirect::new(60, 56, 37), Redirect::new(56, 93, 4)]
        );
    }
}
