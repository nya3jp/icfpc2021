use crate::schema::Pose;

pub fn dislike(hole: &Vec<(i64,i64)>, pose: &Pose) -> usize {
    let mut dislike = 0;
    for (hx, hy) in hole {
        dislike += pose.vertices.iter().map(|(vx, vy)| (hx - vx) * (hx - vx) + (hy - vy) * (hy - vy)).min().unwrap()
    }
    dislike as usize
}
