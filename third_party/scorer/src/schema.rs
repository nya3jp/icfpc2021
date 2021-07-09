use std::fs::File;
use std::io::BufReader;
use std::vec::Vec;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};
use std::error::Error;

// TODO
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct Point {
    x: i64,
    y: i64,
}

// TODO
#[derive(Serialize, Deserialize, Debug)]
pub struct Hole {
    vertices: Vec<(i64, i64)>,
}

// TODO
#[derive(Serialize_tuple, Deserialize_tuple, Debug)]
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