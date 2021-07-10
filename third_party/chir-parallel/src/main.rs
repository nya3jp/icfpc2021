use geom::schema;
use geom::point::Point;
use log::{error, info};
use std::env;
use std::error::Error;
use scorer;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let problem = schema::parse_problem(&args[1])?;
    let mut pose = schema::parse_pose(&args[2])?;

    let initial_score = scorer::dislike(&problem.hole, &pose);
    info!("initial_score = {:?}", initial_score);
    info!("initial_is_valid = {:?}", scorer::is_valid_solution(&problem, &pose));

    for dx in -300..=300 {
        for dy in -300..=300 {
            let p = Point { x: dx as _, y: dy as _ };
            for i in 0..pose.vertices.len() {
                pose.vertices[i] += p
            }

            let score = scorer::dislike(&problem.hole, &pose);
            let is_valid = scorer::is_valid_solution(&problem, &pose);
            if is_valid {
                info!("found new solution = {:?} (shift {:?})", score, p);
            }
            if is_valid && score < initial_score {
                error!("Found a new better solution = {:?}", score);
                error!("{}", serde_json::to_string(&pose)?);
            }

            for i in 0..pose.vertices.len() {
                pose.vertices[i] -= p
            }
        }
    }

    Ok(())
}
