#[cfg(feature = "draw")]
pub mod draw;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::sync::Once;

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

static LOGGING_INIT: Once = Once::new();

pub fn init_logging() {
    LOGGING_INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();
    });
}
