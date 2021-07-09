use serde::{Serialize, Deserialize};

use super::point::Point;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "Vec<(i64, i64)>", into = "Vec<(i64, i64)>")]
pub struct Polygon {
    pub vertices: Vec<Point>,
}

impl From<Vec<(i64, i64)>> for Polygon {
    fn from(t: Vec<(i64, i64)>) -> Self {
        Polygon::new(t.into_iter().map(Point::from).collect())
    }
}

impl From<Polygon> for Vec<(i64, i64)> {
    fn from(t: Polygon) -> Self {
        t.vertices.into_iter().map(<(i64, i64)>::from).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainsResult {
    OUT, ON, IN,
}

impl Polygon {
    pub fn new(vertices: Vec<Point>) -> Self {
        Self{vertices}
    }

    pub fn contains(&self, p: &Point) -> ContainsResult {
        let mut inside = false;
        for i in 0..self.vertices.len() {
            let curr = &self.vertices[i];
            let next = &self.vertices[(i + 1) % self.vertices.len()];
            let mut a = *curr - *p;
            let mut b = *next - *p;
            if a.y > b.y {
                let tmp = a;
                a = b;
                b = tmp;
            }
            let c = Point::cross(a, b);
            if a.y <= 0. && 0. < b.y && c < 0. { inside = !inside; }
            else if c == 0. && Point::dot(a, b) <= 0. { return ContainsResult::ON; }
        }
        if inside {
            ContainsResult::IN
        } else {
            ContainsResult::OUT
        }
    }
}

