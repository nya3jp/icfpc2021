use anyhow::Result;
use tanakh_solver::*;

fn main() -> Result<()> {
    // dbg!(get_problem(1)?);

    dbg!(submit(59, &Solution { vertices: vec![] })?);

    Ok(())
}
