// Copyright 2021 Team Special Weekend
// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};
use std::ops;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(from = "(i64, i64)", into = "(i64, i64)")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn cross(a: Point, b: Point) -> f64 {
        a.x * b.y - a.y * b.x
    }

    pub fn dot(a: Point, b: Point) -> f64 {
        a.x * b.x + a.y * b.y
    }

    pub fn norm_sqr(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
}

impl From<(i64, i64)> for Point {
    fn from(t: (i64, i64)) -> Point {
        Point {
            x: t.0 as f64,
            y: t.1 as f64,
        }
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

impl ops::Add<&Point> for &Point {
    type Output = Point;
    fn add(self, other: &Point) -> Self::Output {
        Self::Output::new(self.x + other.x, self.y + other.y)
    }
}

impl ops::AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::AddAssign<&Point> for Point {
    fn add_assign(&mut self, rhs: &Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::Sub<Point> for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Self::Output {
        Self::Output::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::Sub<&Point> for &Point {
    type Output = Point;
    fn sub(self, other: &Point) -> Self::Output {
        Self::Output::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl ops::SubAssign<&Point> for Point {
    fn sub_assign(&mut self, rhs: &Point) {
        self.x -= rhs.x;
        self.y -= rhs.y;
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

impl ops::Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point::new(-self.x, -self.y)
    }
}
