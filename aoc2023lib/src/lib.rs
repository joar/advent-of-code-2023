use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::sync::Once;

use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

static LOGGING_INIT: Once = Once::new();

pub fn maybe_init_logging() {
    LOGGING_INIT.call_once(|| {
        pretty_env_logger::init();

        let subscriber = create_tracing_subscriber();

        tracing::subscriber::set_global_default(subscriber).unwrap();
    });
}

fn create_tracing_subscriber() -> Layered<Layer<Registry>, Registry, Registry> {
    Registry::default().with(Layer::default())
}
