use anyhow::Result;
use tanakh_solver::*;

#[argopt::subcmd]
fn solve(problem_id: i64) -> Result<()> {
    let problem = get_problem(problem_id)?;

    dbg!(problem);

    Ok(())
}

#[argopt::cmd_group(commands = [solve])]
fn main() -> Result<()> {}
