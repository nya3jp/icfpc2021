use crate::schema::Pose;

pub fn dislike(hole: &Vec<(i64,i64)>, pose: &Pose) -> usize {
    let mut dislike = 0;
    for (hx, hy) in hole {
        dislike += pose.vertices.iter().map(|(vx, vy)| (hx - vx) * (hx - vx) + (hy - vy) * (hy - vy)).min().unwrap()
    }
    dislike as usize
}

enum ContainsResult {
    OUT, ON, IN
}

fn cross(a: &(i64, i64), b: &(i64, i64)) -> i64 {
    a.0 * b.1 - a.1 * b.0
}

fn dot(a: &(i64, i64), b: &(i64, i64)) -> i64 {
    a.0 * b.0 + a.1 * b.1
}

fn contains(polygon: &Vec<(i64, i64)>, p: &(i64, i64)) -> ContainsResult {
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
        if a.1 <= 0 && 0 < b.1 && c < 0 { inside = !inside; }
        else if c == 0 && dot(&a, &b) <= 0 { return ContainsResult::ON; }
    }
    if inside {
        ContainsResult::IN
    } else {
        ContainsResult::OUT
    }
}
