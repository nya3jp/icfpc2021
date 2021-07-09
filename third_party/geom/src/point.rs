use std::ops;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(from = "(i64, i64)", into = "(i64, i64)")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self{x, y}
    }

    pub fn cross(a: Point, b: Point) -> f64 {
        a.x * b.y - a.y * b.x
    }
    
    pub fn dot(a: Point, b: Point) -> f64 {
        a.x * b.x + a.y * b.y
    }    
}

impl From<(i64, i64)> for Point {
    fn from(t: (i64, i64)) -> Point {
        Point { x: t.0 as f64, y: t.1 as f64}
    }
}

impl From<Point> for (i64, i64) {
    fn from(t: Point) -> (i64, i64) {
        (t.x as i64, t.y as i64)
    }
}

impl ops::Add<Point> for Point {
    type Output = Point;
    fn add(self, other: Point) -> Self::Output {
        Self::Output::new(self.x + other.x, self.y + other.y)
    }
}

impl ops::Sub<Point> for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Self::Output {
        Self::Output::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::Mul<Point> for f64 {
    type Output = Point;
    fn mul(self, rhs: Point) -> Self::Output {
        Self::Output::new(self * rhs.x, self * rhs.y)
    }
}

impl ops::Mul<f64> for Point {
    type Output = Point;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output::new(self.x * rhs, self.y * rhs)
    }
}

impl ops::Div<f64> for Point {
    type Output = Point;
    fn div(self, rhs: f64) -> Self::Output {
        Self::Output::new(self.x / rhs, self.y / rhs)
    }
}