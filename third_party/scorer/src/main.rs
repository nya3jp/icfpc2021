use geom::schema;
use scorer;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let problem = schema::parse_problem(&args[1])?;
    let pose = schema::parse_pose(&args[2])?;
    let score = scorer::dislike(&problem.hole, &pose);
    let is_valid = scorer::is_valid_solution(&problem, &pose);
    let bonus = scorer::bonus(&problem, &pose);
    if args.len() == 4 && args[3] == "json" {
        println!("{{\"dislike\": {:?}, \"is_valid\": {:?}, \"bonus\": {}}}",
                 score, is_valid, serde_json::to_string(&bonus)?);
    } else {
        eprintln!("Problem {:?}", problem);
        eprintln!("Solution {:?}", pose);
        eprintln!("dislike = {:?}", score);
        eprintln!("is_valid = {:?}", is_valid);
        eprintln!("bonus = {}", serde_json::to_string(&bonus)?);
    }
    Ok(())
}
