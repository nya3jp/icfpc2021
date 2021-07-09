use std::error::Error;
use std::env;

mod schema;
mod scorer;

fn main() -> Result<(), Box<dyn Error>> {
    let args : Vec<String> = env::args().collect();

    let problem = schema::parse_problem(&args[1])?;
    let pose = schema::parse_pose(&args[2])?;
    eprintln!("Problem {:?}", problem);
    eprintln!("Solution {:?}", pose);
    let score = scorer::dislike(&problem.hole, &pose);
    eprintln!("dislike = {:?}", score);
    Ok(())
}
