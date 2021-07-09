use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "(i64, i64)", into = "(i64, i64)")]
pub struct Point {
    x: i64,
    y: i64,
}

impl From<(i64, i64)> for Point {
    fn from(t: (i64, i64)) -> Point {
        Point { x: t.0, y: t.1 }
    }
}

impl From<Point> for (i64, i64) {
    fn from(t: Point) -> (i64, i64) {
        (t.x, t.y)
    }
}

// TODO
#[derive(Serialize, Deserialize, Debug)]
pub struct Hole {
    vertices: Vec<(i64, i64)>,
}

// TODO
#[derive(Debug)]
pub struct Edge {
    v1: usize,
    v2: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Figure {
    pub vertices: Vec<(i64, i64)>,
    pub edges: Vec<(i64, i64)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Problem {
    pub hole: Vec<(i64, i64)>,
    pub figure: Figure,
    pub epsilon: i64,
}

pub fn parse_problem<P: AsRef<Path>>(path: P) -> Result<Problem, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pose {
    pub vertices: Vec<(i64, i64)>,
}

pub fn parse_pose<P: AsRef<Path>>(path: P) -> Result<Pose, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}
