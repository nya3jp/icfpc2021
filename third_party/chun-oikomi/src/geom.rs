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

use std::vec::Vec;

use crate::Problem;

// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(from = "(i64, i64)", into = "(i64, i64)")]
// pub struct Point {
//     x: i64,
//     y: i64,
// }

// impl From<(i64, i64)> for Point {
//     fn from(t: (i64, i64)) -> Point {
//         Point { x: t.0, y: t.1 }
//     }
// }

// impl From<Point> for (i64, i64) {
//     fn from(t: Point) -> (i64, i64) {
//         (t.x, t.y)
//     }
// }

// // TODO
// #[derive(Serialize, Deserialize, Debug)]
// pub struct Hole {
//     vertices: Vec<(i64, i64)>,
// }

// // TODO
// #[derive(Debug)]
// pub struct Edge {
//     v1: usize,
//     v2: usize,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Figure {
//     pub vertices: Vec<(i64, i64)>,
//     pub edges: Vec<(i64, i64)>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Problem {
//     pub hole: Vec<(i64, i64)>,
//     pub figure: Figure,
//     pub epsilon: i64,
// }

// pub fn parse_problem<P: AsRef<Path>>(path: P) -> Result<Problem, Box<dyn Error>> {
//     let file = File::open(path)?;
//     let reader = BufReader::new(file);
//     Ok(serde_json::from_reader(reader)?)
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Pose {
//     pub vertices: Vec<(i64, i64)>,
// }

// pub fn parse_pose<P: AsRef<Path>>(path: P) -> Result<Pose, Box<dyn Error>> {
//     let file = File::open(path)?;
//     let reader = BufReader::new(file);
//     Ok(serde_json::from_reader(reader)?)
// }

type Pose = Vec<(i64, i64)>;

pub fn dislike(hole: &Vec<(i64, i64)>, pose: &Pose) -> usize {
    let mut dislike = 0;
    for (hx, hy) in hole {
        dislike += pose
            .iter()
            .map(|(vx, vy)| (hx - vx) * (hx - vx) + (hy - vy) * (hy - vy))
            .min()
            .unwrap()
    }
    dislike as usize
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContainsResult {
    OUT,
    ON,
    IN,
}

fn cross(a: &(i64, i64), b: &(i64, i64)) -> i64 {
    a.0 * b.1 - a.1 * b.0
}

fn dot(a: &(i64, i64), b: &(i64, i64)) -> i64 {
    a.0 * b.0 + a.1 * b.1
}

pub fn contains(polygon: &Vec<(i64, i64)>, p: &(i64, i64)) -> ContainsResult {
    let mut inside = false;
    for i in 0..polygon.len() {
        let curr = &polygon[i];
        let next = &polygon[(i + 1) % polygon.len()];
        let mut a = (curr.0 - p.0, curr.1 - p.1);
        let mut b = (next.0 - p.0, next.1 - p.1);
        if a.1 > b.1 {
            let tmp = a;
            a = b;
            b = tmp;
        }
        let c = cross(&a, &b);
        if a.1 <= 0 && 0 < b.1 && c < 0 {
            inside = !inside;
        } else if c == 0 && dot(&a, &b) <= 0 {
            return ContainsResult::ON;
        }
    }
    if inside {
        ContainsResult::IN
    } else {
        ContainsResult::OUT
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CCWResult {
    Clockwise,
    CounterClockwise,
    OnLine,
}

fn ccw(p1: &(i64, i64), p2: &(i64, i64), p3: &(i64, i64)) -> CCWResult {
    let b = (p2.0 - p1.0, p2.1 - p1.1);
    let c = (p3.0 - p1.0, p3.1 - p1.1);
    let cr = cross(&b, &c);
    if cr < 0 {
        CCWResult::CounterClockwise
    } else if cr > 0 {
        CCWResult::Clockwise
    } else {
        CCWResult::OnLine
    }
}

// Returns true iff p1-p2 crosses p3-p4. Returns false if
fn is_crossing(p1: &(i64, i64), p2: &(i64, i64), p3: &(i64, i64), p4: &(i64, i64)) -> bool {
    let ccw1 = ccw(p1, p2, p3);
    let ccw2 = ccw(p1, p2, p4);
    let ccw3 = ccw(p3, p4, p1);
    let ccw4 = ccw(p3, p4, p2);
    if ccw1 == CCWResult::OnLine
        || ccw2 == CCWResult::OnLine
        || ccw3 == CCWResult::OnLine
        || ccw4 == CCWResult::OnLine
    {
        return false;
    }

    ccw1 != ccw2 && ccw3 != ccw4
}

pub fn is_inside_hole(problem: &Problem, pose: &Pose) -> bool {
    // The vertices in pose should match with figure.
    if problem.figure.vertices.len() != pose.len() {
        return false;
    }

    // All points should be contained in the hole.
    for p in pose.iter() {
        if contains(&problem.hole, &p) == ContainsResult::OUT {
            return false;
        }
    }

    // All segments should not cross each other.
    for (e1, e2) in &problem.figure.edges {
        let p1 = &pose[*e1 as usize];
        let p2 = &pose[*e2 as usize];
        for i in 0..problem.hole.len() {
            let h1 = &problem.hole[i];
            let h2 = &problem.hole[(i + 1) % problem.hole.len()];
            if is_crossing(p1, p2, h1, h2) {
                return false;
            }
        }
    }

    // All mid points should be contained in the hole.
    let scaled_hole = problem
        .hole
        .iter()
        .map(|v| (v.0 * 2, v.1 * 2))
        .collect::<Vec<(i64, i64)>>();
    for (e1, e2) in &problem.figure.edges {
        let p1 = &pose[*e1 as usize];
        let p2 = &pose[*e2 as usize];
        let mid = (p1.0 + p2.0, p1.1 + p2.1);
        if contains(&scaled_hole, &mid) == ContainsResult::OUT {
            return false;
        }
    }

    true
}

pub fn is_valid_solution(problem: &Problem, pose: &Pose) -> bool {
    if !is_inside_hole(problem, pose) {
        return false;
    }

    // All edges should satisfy the strech restriction.
    for (e1, e2) in &problem.figure.edges {
        let p1 = problem.figure.vertices[*e1 as usize];
        let p2 = problem.figure.vertices[*e2 as usize];
        let d1 = (p1.0 - p2.0) * (p1.0 - p2.0) + (p1.1 - p2.1) * (p1.1 - p2.1);
        let q1 = pose[*e1 as usize];
        let q2 = pose[*e2 as usize];
        let d2 = (q1.0 - q2.0) * (q1.0 - q2.0) + (q1.1 - q2.1) * (q1.1 - q2.1);
        // if d1 < d2
        //   | d1/d2 - 1 | = 1 - d1/d2.
        //   <=> check d2 * 1000000 - d1 * 1000000 <= eps * d2
        // else
        //   | d1/d2 - 1 | = d1/d2 - 1
        //   <=>check d1 * 1000000 - d2 * 1000000 <= eps * d2
        let lhs = if d1 < d2 {
            d2 * 1000000 - d1 * 1000000
        } else {
            d1 * 1000000 - d2 * 1000000
        };
        let rhs = problem.epsilon * d2;
        if lhs > rhs {
            return false;
        }
    }

    true
}
