use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::point::Point;
use super::polygon::Polygon;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "Vec<(i64, i64)>", into = "Vec<(i64, i64)>")]
pub struct Hole {
    pub polygon: Polygon,
}

impl From<Vec<(i64, i64)>> for Hole {
    fn from(t: Vec<(i64, i64)>) -> Self {
        Hole {
            polygon: Polygon::from(t),
        }
    }
}

impl From<Hole> for Vec<(i64, i64)> {
    fn from(t: Hole) -> Self {
        Vec::from(t.polygon)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(from = "(i64, i64)", into = "(i64, i64)")]
pub struct Edge {
    pub v1: usize,
    pub v2: usize,
}

impl Edge {
    pub fn new(v1: usize, v2: usize) -> Self {
        Self { v1, v2 }
    }
}

impl From<(i64, i64)> for Edge {
    fn from(t: (i64, i64)) -> Self {
        Self {
            v1: t.0 as usize,
            v2: t.1 as usize,
        }
    }
}

impl From<Edge> for (i64, i64) {
    fn from(t: Edge) -> Self {
        (t.v1 as i64, t.v2 as i64)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Figure {
    pub vertices: Vec<Point>,
    pub edges: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Problem {
    pub hole: Hole,
    pub figure: Figure,
    pub epsilon: i64,
}

pub fn parse_problem<P: AsRef<Path>>(path: P) -> Result<Problem, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pose {
    pub vertices: Vec<Point>,
}

pub fn parse_pose<P: AsRef<Path>>(path: P) -> Result<Pose, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}
