use anyhow::Result;
use tanakh_solver::*;

#[argopt::subcmd]
fn solve(problem_id: i64) -> Result<()> {
    let problem = get_problem(problem_id)?;

    dbg!(problem);

    Ok(())
}

#[argopt::subcmd(name = "max-scores")]
fn max_scores() -> Result<()> {
    println!("Max scores:");

    for pid in 1..=59 {
        let problem = get_problem(pid)?;
        let max_score = 1000.0
            * ((problem.figure.vertices.len() * problem.figure.edges.len() * problem.hole.len())
                as f64
                / 6.0)
                .log2();

        println!("Problem {}: {}", pid, max_score.ceil() as i64);
    }

    Ok(())
}

#[argopt::cmd_group(commands = [solve, max_scores])]
fn main() -> Result<()> {}
