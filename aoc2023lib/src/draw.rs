use std::ops::Add;

use anyhow::Result;
use cairo;
use cairo::{Content, Context, Operator};
use pango::{Alignment, FontDescription};

#[derive(Copy, Clone)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl Color {
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self::rgba(r, g, b, 1.0)
    }
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    pub fn set_source_color(&self, context: &Context) {
        context.set_source_rgba(self.r, self.g, self.b, self.a);
    }

    pub fn fill(&self, context: &Context) -> Result<()> {
        context.set_source_rgba(self.r, self.g, self.b, self.a);
        context.fill()?;
        Ok(())
    }

    pub fn stroke(&self, context: &Context) -> Result<()> {
        context.set_source_rgba(self.r, self.g, self.b, self.a);
        context.set_line_width(2.);
        context.stroke()?;
        Ok(())
    }
}

pub trait Draw {
    fn draw(&self, context: &Context) -> Result<()>;
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

pub struct Rectangle {
    top_left: Point,
    width: f64,
    height: f64,
    fill_color: Option<Color>,
    stroke_color: Option<Color>,
}

impl Rectangle {
    pub fn create(top_left: Point, width: f64, height: f64) -> Self {
        Self {
            top_left,
            width,
            height,
            fill_color: Default::default(),
            stroke_color: Default::default(),
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }

    fn set_path(&self, context: &Context) {
        context.rectangle(
            self.top_left.x(),
            self.top_left.y(),
            self.width,
            self.height,
        );
    }
}

impl Draw for Rectangle {
    fn draw(&self, context: &Context) -> Result<()> {
        self.set_path(context);
        if let Some(fill_color) = self.fill_color {
            fill_color.fill(context)?;
        }
        if let Some(stroke_color) = self.stroke_color {
            stroke_color.stroke(context)?;
            // stroke_inside(context, stroke_color);
        }
        Ok(())
    }
}

#[allow(dead_code)]
fn stroke_inside(context: &Context, stroke_color: Color) -> Result<()> {
    let line_width = context.line_width();
    let operator = context.operator();
    context.push_group_with_content(Content::Alpha);
    context.set_line_width(2.);
    stroke_color.set_source_color(context);
    context.set_operator(Operator::Source);
    context.stroke_preserve()?;
    context.set_operator(Operator::Clear);
    context.fill_preserve()?;
    let mask = context.pop_group()?;
    context.mask(&mask)?;

    context.set_line_width(line_width);
    context.set_operator(operator);
    Ok(())
}

pub fn draw_text_in_center_of_square(
    context: &Context,
    text_color: Color,
    text: &str,
    center: &Point,
    square_size: &f64,
) -> Result<()> {
    let font_size: i32 = (square_size * (13.0 / square_size)).round() as i32;

    let layout = pangocairo::create_layout(context);
    let mut font_description = FontDescription::from_string("Inconsolata,Medium");
    font_description.set_size(font_size * pango::SCALE);
    layout.set_font_description(Some(&font_description));
    layout.set_width(*square_size as i32);
    layout.set_height(*square_size as i32);
    layout.set_alignment(Alignment::Center);
    layout.set_text(text);

    let origin = Point::new(center.x(), center.y() - ((*square_size) / 2.0));

    context.save()?;
    context.move_to(origin.x(), origin.y());
    text_color.set_source_color(context);
    // context.show_text(text)?;
    pangocairo::show_layout(context, &layout);
    context.restore()?;
    Ok(())
}
