use crate::schema::{Problem, Pose};

use geom::*;

pub fn dislike(hole: &Vec<(i64,i64)>, pose: &Pose) -> usize {
    let mut dislike = 0;
    for (hx, hy) in hole {
        dislike += pose.vertices.iter().map(|(vx, vy)| (hx - vx) * (hx - vx) + (hy - vy) * (hy - vy)).min().unwrap()
    }
    dislike as usize
}

pub fn is_valid_solution(problem: &Problem, pose: &Pose) -> bool {
    // The vertices in pose should match with figure.
    if problem.figure.vertices.len() != pose.vertices.len() {
        return false;
    }

    // All points should be contained in the hole.
    let hole_polygon = Polygon::new(problem.hole.iter().map(|v| Point::from(*v)).collect());
    for p in &pose.vertices {
        if hole_polygon.contains(&Point::from(*p)) == ContainsResult::OUT {
            return false;
        }
    }
    
    // All segments should not cross each other.
    for (e1, e2) in &problem.figure.edges {
        let p1 = Point::from(pose.vertices[*e1 as usize]);
        let p2 = Point::from(pose.vertices[*e2 as usize]);
        for i in 0..problem.hole.len() {
            let h1 = Point::from(problem.hole[i]);
            let h2 = Point::from(problem.hole[(i + 1) % problem.hole.len()]);
            if is_crossing(&p1, &p2, &h1, &h2) {
                return false;
            }
        }
        // All mid points should be contained in the hole.
        let mid = (p1 + p2) / 2.;
        if hole_polygon.contains(&mid) == ContainsResult::OUT {
            return false;
        }
    }
    
    // All edges should satisfy the strech restriction.
    for (e1, e2) in &problem.figure.edges {
        let p1 = problem.figure.vertices[*e1 as usize];
        let p2 = problem.figure.vertices[*e2 as usize];
        let d1 = (p1.0 - p2.0) * (p1.0 - p2.0) + (p1.1 - p2.1) * (p1.1 - p2.1);
        let q1 = pose.vertices[*e1 as usize];
        let q2 = pose.vertices[*e2 as usize];
        let d2 = (q1.0 - q2.0) * (q1.0 - q2.0) + (q1.1 - q2.1) * (q1.1 - q2.1);
        // if d1 < d2
        //   | d1/d2 - 1 | = 1 - d1/d2.
        //   <=> check d2 * 1000000 - d1 * 1000000 <= eps * d2
        // else
        //   | d1/d2 - 1 | = d1/d2 - 1
        //   <=>check d1 * 1000000 - d2 * 1000000 <= eps * d2
        let lhs =
            if d1 < d2 {
                d2 * 1000000 - d1 * 1000000
            } else {
                d1 * 1000000 - d2 * 1000000
            };
        let rhs = problem.epsilon * d2;
        if lhs > rhs { return false; }
    }

    true
}