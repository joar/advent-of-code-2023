extern crate piston_window;

use std::collections::HashSet;
use std::fs::File;
use std::ops::{Add, Index};
use std::sync::atomic::{AtomicUsize, Ordering};

use ::grid::Grid;
use anyhow::{anyhow, Context as AnyhowContext, Result};
use cairo;
use cairo::{Context, Format, ImageSurface};

use aoc2023lib::draw::{draw_text_in_center_of_square, Color, Draw, Point, Rectangle};
use aoc2023lib::{init_logging, read_lines};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Value {
    Blank,
    Symbol(char),
    Digit(u8),
}

#[derive(Copy, Clone)]
struct RGB {
    r: f64,
    g: f64,
    b: f64,
}

impl RGB {
    fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    fn set_source_color(&self, context: &Context) {
        context.set_source_rgb(self.r, self.g, self.b)
    }
}

struct Evaluator {
    grid: Grid<Value>,
    square_size: f64,
    surface: ImageSurface,
    context: Context,
    frame_counter: AtomicUsize,
}

impl<'a> Evaluator {
    pub fn new(grid: Grid<Value>) -> Result<Self> {
        let square_size: f64 = 20.0;
        let width = (grid.cols() * square_size.round() as usize) as i32;
        let height = (grid.rows() * square_size.round() as usize) as i32;
        let surface = ImageSurface::create(Format::ARgb32, width, height)?;

        let context = Context::new(&surface)?;

        context.rectangle(0., 0., width as f64, height as f64);
        context.set_source_rgb(1., 1., 1.);
        context.fill()?;

        Ok(Self {
            grid,
            square_size,
            surface,
            context,
            frame_counter: AtomicUsize::new(0),
        })
    }

    fn run(&self) -> Result<()> {
        self.draw_grid()?;
        self.write_frame()?;

        let mut part_numbers: Vec<PartNumber> = Vec::new();

        for symbol_position in self.find_symbols() {
            self.fill_square(symbol_position, Color::rgba(0., 0., 1., 0.1))?;
            self.write_frame()?;
            for part_number_positions in self.find_part_numbers(symbol_position) {
                for pos in part_number_positions.clone() {
                    self.fill_square(pos, Color::rgba(1., 0., 0., 0.1))?;
                }

                self.write_frame()?;
                let part_number =
                    PartNumber::from_grid_positions(&self.grid, part_number_positions)?;
                part_numbers.push(part_number);
            }
        }

        let sum: i32 = part_numbers.iter().map(|pn| pn.number).sum();
        println!("Sum is {}", sum);
        Ok(())
    }

    fn find_part_numbers(&self, symbol_position: Position) -> impl Iterator<Item = Vec<Position>> {
        let neighbor_positions = get_neighbor_positions(&self.grid, symbol_position);
        let mut visited_positions: HashSet<Position> = HashSet::new();
        neighbor_positions
            .iter()
            .filter_map(|pos| match pos.grid_value(&self.grid) {
                Some(Value::Digit(_)) => {
                    if !visited_positions.contains(pos) {
                        visited_positions.insert(pos.clone());
                        let connected_numbers = complete_part_number(&self.grid, pos.clone());
                        for cp in connected_numbers.clone() {
                            visited_positions.insert(cp.clone());
                        }
                        Some(connected_numbers)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<Vec<Position>>>()
            .into_iter()
    }

    fn fill_square(&self, position: Position, color: Color) -> Result<()> {
        let top_left = Point::new(
            self.square_size * position.x() as f64,
            self.square_size * position.y() as f64,
        );
        Rectangle::create(top_left, self.square_size, self.square_size)
            .fill(color)
            .draw(&self.context)
    }

    fn draw_grid(&self) -> Result<()> {
        let grid = &self.grid;
        for x_int in 0..grid.cols() {
            for y_int in 0..grid.rows() {
                let (x, y) = (x_int as f64, y_int as f64);
                let top_left = Point::new(self.square_size * x, self.square_size * y);
                let center = top_left + Point::new(self.square_size / 2., self.square_size / 2.);
                // let bottom_left = top_left.add(Point::new(0.0, square_size));
                match grid.get(y_int, x_int) {
                    Some(a) => match a {
                        Value::Blank => {
                            draw_text_in_center_of_square(
                                &self.context,
                                Color::rgba(0.0, 0.0, 0.0, 1.0),
                                ".",
                                &center,
                                &self.square_size,
                            )?;
                        }
                        Value::Symbol(c) => {
                            // Rectangle::create(top_left, square_size, square_size)
                            //     .fill(Color::rgba(0.0, 0.0, 0.0, 0.1))
                            //     .draw(&context)?;

                            let string = String::from(*c);
                            let text = string.as_str();
                            draw_text_in_center_of_square(
                                &self.context,
                                Color::rgba(0.0, 0.0, 0.0, 1.0),
                                text,
                                &center,
                                &self.square_size,
                            )?;
                        }
                        Value::Digit(value) => {
                            // Rectangle::create(top_left, square_size, square_size)
                            //     .fill(Color::rgba(0.0, 0.0, 1.0, 0.1))
                            //     .draw(&context)?;

                            let str = format!("{}", value);
                            let digit = str.as_str();
                            draw_text_in_center_of_square(
                                &self.context,
                                Color::rgb(0., 0., 0.),
                                digit,
                                &center,
                                &self.square_size,
                            )?;
                        }
                    },
                    None => {}
                }
            }
        }
        Ok(())
    }

    fn find_symbols(&self) -> impl Iterator<Item = Position> + '_ {
        self.grid.iter_rows().enumerate().flat_map(|(y, row)| {
            row.enumerate().filter_map(move |(x, value)| match value {
                Value::Blank => None,
                Value::Symbol(_) => Some(Position {
                    x: x.clone(),
                    y: y.clone(),
                }),
                Value::Digit(_) => None,
            })
        })
    }

    fn write_frame(&self) -> Result<()> {
        let idx = self.frame_counter.fetch_add(1, Ordering::SeqCst);
        let filename = format!("frame-{:05}.png", idx).to_string();

        eprintln!("Writing frame {:?}", filename);
        let mut file =
            File::create(filename.as_str()).context("Could not create frame output file")?;
        self.surface
            .write_to_png(&mut file)
            .with_context(|| format!("Could not write frame to {}", filename))?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn add_x(&self, value: usize) -> Self {
        Self {
            x: self.x + value,
            y: self.y,
        }
    }

    pub fn sub_x(&mut self, value: usize) -> Self {
        Self {
            x: self.x - value,
            y: self.y,
        }
    }

    pub fn grid_value<'a, T>(&self, grid: &'a Grid<T>) -> Option<&'a T> {
        grid.get(self.y, self.x)
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

trait SizedPosIterator: Iterator<Item = Position> + Sized {}

struct PartNumber {
    number: i32,
    positions: Vec<Position>,
}

impl PartNumber {
    pub fn from_grid_positions(grid: &Grid<Value>, positions: Vec<Position>) -> Result<Self> {
        let mut sorted_positions = positions.clone();
        sorted_positions.sort_by_key(|pos| pos.x());

        let mut numbers: Vec<u8> = Vec::new();

        for pos in sorted_positions.clone() {
            match pos.grid_value(grid) {
                Some(Value::Digit(value)) => {
                    numbers.push(value.clone());
                }
                other => Err(anyhow!(
                    "Expected number at position {:?}, got {:?}",
                    pos,
                    other
                ))?,
            }
        }

        let str: String = numbers
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join("");
        let number = str.parse::<i32>()?;
        Ok(Self {
            number,
            positions: sorted_positions,
        })
    }
}

fn get_neighbor_positions(grid: &Grid<Value>, position: Position) -> Vec<Position> {
    let mut neighbors: Vec<Position> = Vec::new();
    for x_offset in -1i8..=1 {
        for y_offset in -1i8..=1 {
            if (x_offset, y_offset) != (0, 0) {
                let neighbor: Position = Position::new(
                    (position.x() as isize + x_offset as isize) as usize,
                    (position.y() as isize + y_offset as isize) as usize,
                );
                if (0..=grid.cols()).contains(&neighbor.x())
                    && (0..=grid.rows()).contains(&neighbor.y())
                {
                    neighbors.push(neighbor);
                }
            }
        }
    }
    neighbors
}

fn complete_part_number(grid: &Grid<Value>, symbol_position: Position) -> Vec<Position> {
    let mut pos = symbol_position;
    let mut positions: HashSet<Position> = HashSet::new();
    while let Some(Value::Digit(_)) = pos.grid_value(grid) {
        positions.insert(pos);
        if pos.x == 0 {
            break;
        }
        pos = Position::new(pos.x - 1, pos.y);
    }
    pos = symbol_position;
    while let Some(Value::Digit(_)) = pos.grid_value(grid) {
        positions.insert(pos);
        pos = Position::new(pos.x + 1, pos.y);
    }
    positions.into_iter().collect()
}

fn grid_from_lines<'a, I>(lines: I) -> Result<Grid<Value>>
where
    I: IntoIterator<Item = &'a str> + Index<usize, Output = &'a str> + Clone,
{
    let cols = lines[0].len();
    for line in lines.clone() {
        if line.len() != cols {
            panic!(
                "Expected lines to be of length {}, but {} was of length {}",
                cols,
                line,
                line.len()
            )
        }
    }

    let char_vec = lines
        .into_iter()
        .flat_map(|line| {
            line.chars()
                .map(|c| match c.is_ascii_digit() {
                    true => Value::Digit(c.to_string().parse::<u8>().unwrap()),
                    false => match c {
                        '.' => Value::Blank,
                        c => Value::Symbol(c),
                    },
                })
                .collect::<Vec<Value>>()
        })
        .collect();

    Ok(Grid::from_vec(char_vec, cols))
}

fn main() -> Result<()> {
    init_logging();
    let lines: Vec<String> = read_lines("day03-gear/input")
        .context("Could not read input")?
        .collect::<Result<_, _>>()
        .context("Could not read line")?;
    let grid = grid_from_lines(lines.iter().map(String::as_str).collect::<Vec<&str>>())?;

    let evaluator = Evaluator::new(grid)?;
    evaluator.run()?;
    println!("Hello, world!");
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{grid_from_lines, Value};

    #[test]
    fn test_grid_from_lines() {
        let lines = vec!["abc123", "456..."];
        let grid = grid_from_lines(lines).unwrap();

        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.cols(), 6);
        assert_eq!(grid.get(0, 3), Some(&Value::Digit(1)));
        assert_eq!(grid.get(1, 0), Some(&Value::Digit(4)));
    }
}
