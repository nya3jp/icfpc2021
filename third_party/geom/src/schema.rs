use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

use super::point::Point;
use super::polygon::{ContainsResult, Polygon};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "Vec<(i64, i64)>", into = "Vec<(i64, i64)>")]
pub struct Hole {
    pub polygon: Polygon,
    minx: i64,
    miny: i64,
    maxx: i64,
    maxy: i64,
    contains_set: Vec<Vec<bool>>,
}

fn create_contains_set(
    t: &Vec<(i64, i64)>,
    polygon: &Polygon,
) -> (i64, i64, i64, i64, Vec<Vec<bool>>) {
    let mut ret = Vec::new();

    let minx = t.iter().fold(1 << 20, |acc, x| std::cmp::min(acc, x.0)) * 2;
    let maxx = t.iter().fold(-1 << 20, |acc, x| std::cmp::max(acc, x.0)) * 2;
    let miny = t.iter().fold(1 << 20, |acc, x| std::cmp::min(acc, x.1)) * 2;
    let maxy = t.iter().fold(-1 << 20, |acc, x| std::cmp::max(acc, x.1)) * 2;
    for x in minx..=maxx {
        let mut line = Vec::new();
        for y in miny..=maxy {
            match polygon.contains(&Point::new(x as f64 / 2., y as f64 / 2.)) {
                ContainsResult::ON | ContainsResult::IN => line.push(true),
                ContainsResult::OUT => line.push(false),
            }
        }
        ret.push(line)
    }
    (minx, miny, maxx, maxy, ret)
}

impl Hole {
    pub fn len(&self) -> usize {
        self.polygon.vertices.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Point> {
        self.polygon.vertices.iter()
    }

    // Point coord must be integers.
    pub fn contains(&self, p: &Point) -> bool {
        let x = (p.x * 2.) as i64;
        let y = (p.y * 2.) as i64;
        if x < self.minx || self.maxx < x || y < self.miny || self.maxy < y {
            return false;
        }
        self.contains_set[(x - self.minx) as usize][(y - self.miny) as usize]
    }
}

impl std::ops::Index<usize> for Hole {
    type Output = Point;

    fn index(&self, index: usize) -> &Self::Output {
        &self.polygon.vertices[index]
    }
}

impl std::ops::IndexMut<usize> for Hole {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.polygon.vertices[index]
    }
}

impl From<Vec<(i64, i64)>> for Hole {
    fn from(t: Vec<(i64, i64)>) -> Self {
        let polygon = Polygon::from(t.clone());
        let (minx, miny, maxx, maxy, contains_set) = create_contains_set(&t, &polygon);
        Hole {
            polygon,
            minx,
            miny,
            maxx,
            maxy,
            contains_set,
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

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum BonusType {
    GLOBALIST,
    BREAK_A_LEG,
    WALLHACK,
    SUPERFLEX,
}

impl FromStr for BonusType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "GLOBALIST" {
            BonusType::GLOBALIST
        } else if s == "BREAK_A_LEG" {
            BonusType::BREAK_A_LEG
        } else if s == "WALLHACK" {
            BonusType::WALLHACK
        } else if s == "SUPERFLEX" {
            BonusType::SUPERFLEX
        } else {
            Err(format!("Unknown Bonus Type; {}", s))?
        })
    }
}

impl Display for BonusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BonusType::GLOBALIST => write!(f, "GLOBALIST"),
            BonusType::BREAK_A_LEG => write!(f, "BREAK_A_LEG"),
            BonusType::WALLHACK => write!(f, "WALLHACK"),
            BonusType::SUPERFLEX => write!(f, "SUPERFLEX"),
        }
    }
}

impl From<String> for BonusType {
    fn from(t: String) -> BonusType {
        t.parse().unwrap()
    }
}

impl From<BonusType> for String {
    fn from(t: BonusType) -> String {
        t.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bonus {
    pub position: Point,
    pub bonus: BonusType,
    pub problem: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Problem {
    pub hole: Hole,
    pub figure: Figure,
    pub epsilon: i64,

    pub bonuses: Vec<Bonus>,
}

pub fn parse_problem<P: AsRef<Path>>(path: P) -> Result<Problem, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsedBonus {
    pub bonus: BonusType,
    pub problem: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge: Option<Edge>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pose {
    pub vertices: Vec<Point>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonuses: Option<Vec<UsedBonus>>,
}

impl Pose {
    pub fn has_globalist(&self) -> bool {
        match &self.bonuses {
            None => false,
            Some(bonuses) => bonuses.iter().any(|b| b.bonus == BonusType::GLOBALIST),
        }
    }

    pub fn has_wallhack(&self) -> bool {
        match &self.bonuses {
            None => false,
            Some(bonuses) => bonuses.iter().any(|b| b.bonus == BonusType::WALLHACK),
        }
    }

    pub fn has_superflex(&self) -> bool {
        match &self.bonuses {
            None => false,
            Some(bonuses) => bonuses.iter().any(|b| b.bonus == BonusType::SUPERFLEX),
        }
    }
}

pub fn parse_pose<P: AsRef<Path>>(path: P) -> Result<Pose, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}
