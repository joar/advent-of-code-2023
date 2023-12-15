extern crate piston_window;

use anyhow::{Context, Result};
use aoc2023lib::read_lines;
use day03_gear::run;
use cairo;
use cairo::Format;
use grid::Grid;
use piston_window::*;

fn piston_run() {
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();
    while let Some(event) = window.next() {
        match event {
            Button::Keyboard(Spa)
        }
        window.draw_2d(&event, |context, graphics, _device| {
            clear([1.0; 4], graphics);
            rectangle(
                [1.0, 0.0, 0.0, 1.0], // red
                [0.0, 0.0, 100.0, 100.0],
                context.transform,
                graphics,
            );
        });
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Value {
    Blank,
    Symbol,
    Digit(u8)
}

fn draw_grid() -> Result<()> {
    let surface = cairo::ImageSurface::create(Format::ARgb32, 800, 800)?;
    let context = cairo::Context::new(&surface)?;
    grid
    context.move_to(0, 0)
    surface.
}

fn grid_from_text() -> Result<Grid<Value>> {
    let lines: Vec<String> = read_lines("day03-gear/input")
        .context("Could not read input")?
        .collect::<Result<_, _>>()
        .context("Could not read line")?;

    let cols = lines[0].len();
    for line in lines {
        if line.len() != cols {
            panic!("Expected lines to be of length {}, but {} was of length {}", cols, line, line.len())
        }
    }

    let char_vec = lines.iter().flat_map(|line| line.chars().map(|c| match c.is_ascii_digit() {
        true => Value::Digit(c.to_string().parse::<u8>().unwrap()),
        false => match c {
            '.' => Value::Blank,
            _ => Value::Symbol
        }
    }).collect::<Vec<Value>>()).collect();

    Grid::from_vec(char_vec, cols)
}

fn main() -> Result<()> {

    piston_run();
    // pollster::block_on(run());
    println!("Hello, world!");
    Ok(())
}
