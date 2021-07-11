mod brute;

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{Datelike, Timelike};
use itertools::Itertools;
use num_complex::Complex;
//use sa::*;

use brute::*;
//use chun_bruteforce_solver::brute::*;
use chun_triangle_solver::geom::is_inside_hole;
use chun_triangle_solver::{get_problem, Problem, Solution};

type Pt = Complex<f64>;

pub fn cross(a: &Pt, b: &Pt) -> f64 {
    (a.conj() * b).im
}

/// Signed area of triangle
pub fn triangle_area_signed(a: &Pt, b: &Pt, c: &Pt) -> f64 {
    cross(&(b - a), &(c - a)) / 2.0
}

fn pt_sub(p1: &(i64, i64), p2: &(i64, i64)) -> (i64, i64) {
    (p1.0 - p2.0, p1.1 - p2.1)
}

fn norm_sqr(p: &(i64, i64)) -> i64 {
    p.0 * p.0 + p.1 * p.1
}

#[argopt::subcmd]
fn solve(
    /// time limit in seconds
    //
    #[opt(long, default_value = "5.0")]
    time_limit: f64,

    /// number of threads
    //
    #[opt(long, default_value = "1")]
    threads: usize,

    #[opt(long)]
    search_vertices: Option<Vec<usize>>,

    #[opt(long)]
    base_solution: Option<PathBuf>,

    #[opt(long)] submit: bool,
    problem_id: i64,
) -> Result<()> {
    let problem = get_problem(problem_id)?;

    let init_state: Option<Vec<(i64, i64)>> = base_solution.map(|frompath| {
        let solution: Solution =
            serde_json::from_reader(
                File::open(&frompath).expect(&format!("{} is not found", frompath.display())),
            )
            .expect("invalid json file");
        solution.vertices
    });
    let (score, solution) = brute(
        &problem, &init_state, &search_vertices
    );

    //let solution = Solution { vertices: solution };

    if score.is_infinite() || (score.round() - score).abs() > 1e-10 {
        eprintln!("Cannot find solution");
        eprintln!("Wrong solution: {}", serde_json::to_string(&solution)?);
        return Ok(());
    }

    eprintln!("Score for problem {}: {}", problem_id, score);

    println!("{}", serde_json::to_string(&solution)?);

    if !Path::new("results").exists() {
        fs::create_dir_all("results")?;
    }

    let now = chrono::Local::now();
    fs::write(
        format!(
            "results/{}-{}-{:02}{:02}{:02}{:02}.json",
            problem_id,
            score.round() as i64,
            now.date().day(),
            now.time().hour(),
            now.time().minute(),
            now.time().second(),
        ),
        serde_json::to_string(&solution)?,
    )?;

    if submit {
        eprintln!("Submitting");

        let resp = chun_triangle_solver::submit(problem_id, &solution)?;
        eprintln!("Response: {:?}", resp);
    }

    Ok(())
}

#[argopt::subcmd]
fn submit(problem_id: i64, json_file: PathBuf) -> Result<()> {
    let solution = serde_json::from_reader(File::open(json_file)?)?;
    let resp = chun_triangle_solver::submit(problem_id, &solution)?;
    println!("{:?}", resp);
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

#[argopt::cmd_group(commands = [solve, max_scores, submit])]
fn main() -> Result<()> {}
