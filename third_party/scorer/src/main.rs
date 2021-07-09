use geom::schema;
use scorer;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let problem = schema::parse_problem(&args[1])?;
    let pose = schema::parse_pose(&args[2])?;
    eprintln!("Problem {:?}", problem);
    eprintln!("Solution {:?}", pose);
    let score = scorer::dislike(&problem.hole, &pose);
    eprintln!("dislike = {:?}", score);
    eprintln!(
        "is_valid = {:?}",
        scorer::is_valid_solution(&problem, &pose)
    );
    Ok(())
}
