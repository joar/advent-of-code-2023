use std::collections::HashMap;

use anyhow::Context;
use tracing::trace;
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Valuable)]
pub struct Redirect {
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

#[derive(Debug)]
pub struct Redirects {
    source: String,
    destination: String,
    redirects: Vec<Redirect>,
}

impl Redirects {
    pub fn new(source: &str, destination: &str, redirects: Vec<Redirect>) -> Self {
        Self {
            source: source.to_string(),
            destination: destination.to_string(),
            redirects,
        }
    }

    #[inline]
    pub fn resolve(&self, source_location: usize) -> anyhow::Result<usize> {
        Ok(
            if let Some(redirect) = self.redirects.iter().find(|r| r.contains(&source_location)) {
                trace!(
                    source_location = source_location,
                    redirect = redirect.as_value(),
                    "found redirect"
                );
                redirect.resolve(source_location)
            } else {
                source_location
            },
        )
    }

    pub fn redirects(&self) -> Vec<Redirect> {
        self.redirects.clone()
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
                self.redirects.as_value(),
            ],
        ));
    }
}

impl Structable for Redirects {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_static("Redirects", Fields::Named(&[]))
    }
}

#[derive(Debug)]
pub struct SowingContext {
    seeds: Vec<usize>,
    redirects_by_source: HashMap<String, Redirects>,
}

impl SowingContext {
    pub fn new(seeds: Vec<usize>, redirects_by_source: HashMap<String, Redirects>) -> Self {
        Self {
            seeds,
            redirects_by_source,
        }
    }
    #[inline]
    pub fn resolve_location(&self, seed_location: usize) -> anyhow::Result<usize> {
        let mut next: &str = "seed";
        let mut location: usize = seed_location;
        while next != "location" {
            trace!("{} {}", next, location);
            let redirects = self
                .redirects_by_source
                .get(next)
                .with_context(|| format!("No redirects with source {}", next))?;
            let prev_location = location;
            location = redirects.resolve(prev_location)?;
            next = redirects.destination.as_str();
        }
        Ok(location)
    }

    pub fn seeds(&self) -> &Vec<usize> {
        &self.seeds
    }

    pub fn redirects_by_source(&self) -> &HashMap<String, Redirects> {
        &self.redirects_by_source
    }
}
