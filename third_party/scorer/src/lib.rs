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

use geom::schema::{Hole, Pose, Problem, UsedBonus};

use geom::*;

pub fn distance(p1: &Point, p2: &Point) -> usize {
    let d = *p1 - *p2;
    Point::dot(d, d) as usize
}

pub fn dislike(hole: &Hole, pose: &Pose) -> usize {
    let mut dislike = 0;
    for p1 in &hole.polygon.vertices {
        dislike += pose
            .vertices
            .iter()
            .map(|p2| distance(p1, p2))
            .min()
            .unwrap()
    }
    dislike as usize
}

pub fn bonus(problem: &Problem, pose: &Pose) -> Vec<UsedBonus> {
    let mut bonuses = vec![];
    for b in &problem.bonuses {
        for v in &pose.vertices {
            if v == &b.position {
                bonuses.push(UsedBonus {
                    bonus: b.bonus,
                    problem: b.problem,
                    edge: None
                });
            }
            eprintln!("b, v: {:?}, {:?}", v, &b.position);
        }
    }
    bonuses
}

fn is_inside_hole_internal(
    problem: &Problem,
    pose: &Pose,
    changed: Option<&[usize]>,
    verbose: bool,
) -> bool {
    // The vertices in pose should match with figure.
    if problem.figure.vertices.len() != pose.vertices.len() {
        if verbose {
            eprintln!(
                "Edge mismatch: {:?}, {:?}",
                problem.figure.vertices.len(),
                pose.vertices.len()
            );
        }
        return false;
    }

    let mut outside_point: Option<usize> = None;
    // All points should be contained in the hole.
    if changed.is_none() {
        for i in 0..pose.vertices.len() {
            let p = &pose.vertices[i];
            if !problem.hole.contains(p) {
                if verbose {
                    eprintln!("Point {:?} is not contained in hole:", Point::from(*p));
                }
                if outside_point.is_some() || !pose.has_wallhack() {
                    return false;
                }
                outside_point = Some(i)
            }
        }
    } else {
        // TODO: Wallhack check for partial update?
        for i in changed.unwrap() {
            let p = &pose.vertices[*i];
            if !problem.hole.contains(p) {
                if verbose {
                    eprintln!("Point {:?} is not contained in hole:", Point::from(*p));
                }
                return false;
            }
        }
    }

    // All segments should not cross each other.
    for edge in &problem.figure.edges {
        if !changed.is_none()
            && !(changed.unwrap().contains(&edge.v1) || changed.unwrap().contains(&edge.v2))
        {
            continue;
        }

        if outside_point.is_some()
            && (outside_point.unwrap() == edge.v1 || outside_point.unwrap() == edge.v2)
            && pose.has_wallhack()
        {
            continue;
        }

        let p1 = &pose.vertices[edge.v1];
        let p2 = &pose.vertices[edge.v2];
        for i in 0..problem.hole.polygon.vertices.len() {
            let h1 = &problem.hole.polygon.vertices[i];
            let h2 = &problem.hole.polygon.vertices[(i + 1) % problem.hole.polygon.vertices.len()];
            if is_crossing(p1, p2, h1, h2) {
                if verbose {
                    eprintln!("Point {:?} {:?} is crossing with {:?} {:?}", p1, p2, h1, h2);
                }
                return false;
            }
        }
        // All mid points should be contained in the hole.
        let mid = (*p1 + *p2) / 2.;
        if !problem.hole.contains(&mid) {
            if verbose {
                eprintln!("Mid point {:?} is not contained in the hole", mid);
            }
            return false;
        }
    }

    true
}

pub fn is_inside_hole_partial(problem: &Problem, pose: &Pose, changed: &[usize]) -> bool {
    is_inside_hole_internal(problem, pose, Some(changed), false)
}

pub fn is_inside_hole(problem: &Problem, pose: &Pose) -> bool {
    is_inside_hole_internal(problem, pose, None, false)
}

pub fn is_inside_hole_verbose(problem: &Problem, pose: &Pose) -> bool {
    is_inside_hole_internal(problem, pose, None, true)
}

pub fn is_valid_solution(problem: &Problem, pose: &Pose) -> bool {
    if !is_inside_hole(problem, pose) {
        return false;
    }

    // TODO: Use integer?
    let mut error_sum: f64 = 0.0;
    let mut error_count = 0;

    // All edges should satisfy the strech restriction.
    for edge in &problem.figure.edges {
        let p1 = problem.figure.vertices[edge.v1];
        let p2 = problem.figure.vertices[edge.v2];
        let d1 = distance(&p1, &p2);
        let q1 = pose.vertices[edge.v1];
        let q2 = pose.vertices[edge.v2];
        let d2 = distance(&q1, &q2);
        // if d1 < d2
        //   | d2/d1 - 1 | = d2/d1 - 1.
        //   <=> check d2 * 1000000 - d1 * 1000000 <= eps * d1
        // else
        //   | d2/d1 - 1 | = 1 - d2/d1
        //   <=>check d1 * 1000000 - d2 * 1000000 <= eps * d1
        let lhs = if d1 < d2 {
            d2 * 1000000 - d1 * 1000000
        } else {
            d1 * 1000000 - d2 * 1000000
        };
        let rhs = problem.epsilon as usize * d1;
        if lhs > rhs {
            eprintln!(
                "Invalid stretch: {:?}/{:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                d1, d2, p1, p2, q1, q2, lhs, rhs
            );

            if pose.has_superflex() {
                error_count += 1;
                if error_count >= 2 {
                    return false;
                }
                continue;
            }

            if !pose.has_globalist() {
                return false;
            }

            error_sum += ((d2 as f64) / (d1 as f64) - 1.0).abs();
        }
    }

    1000000.0 * error_sum <= (problem.figure.edges.len() * problem.epsilon as usize) as f64
}
