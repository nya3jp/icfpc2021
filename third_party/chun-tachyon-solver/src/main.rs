mod brute;

use std::fs::{self, File};
use std::path::{Path, PathBuf};


use anyhow::{anyhow, bail, Result};
use chrono::{Datelike, Timelike};
use itertools::Itertools;
use num_complex::Complex;
use geom::{
    point::Point,
    // polygon::ContainsResult,
    schema::{BonusType, Edge, Pose, Problem as P, UsedBonus},
};
//use sa::*;

use brute::*;
//use chun_bruteforce_solver::brute::*;
use chun_tachyon_solver::{get_problem, SubmitResult};

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


    /// Bonus to use (one of "GLOBALIST", "BREAK_A_LEG", "WALLHACK")
    #[opt(long)]
    use_bonus: Option<BonusType>,

    #[opt(long)] bonus_from: Option<i64>,

    #[opt(long)]
    problem: Option<PathBuf>,

    #[opt(long)]
    search_vertices: Option<Vec<usize>>,

    #[opt(long)]
    base_solution: Option<PathBuf>,

    /// Bonuses to get (one of "GLOBALIST", "BREAK_A_LEG", "WALLHACK")
    #[opt(long)]
    get_bonuses_unimplemented: Vec<BonusType>,


    #[opt(long)] submit: bool,
    problem_id: i64,
) -> Result<()> {
    eprintln!("Initializing solver");
    match &use_bonus {
        None => (),
        Some(BonusType::GLOBALIST) => (),
        Some(r) => {
            bail!("Bonus {} is currently not supported", r);
        }
    }
    let ps = get_problems()?;
    eprintln!("Fetch problem completed");

    let use_bonus: Option<UsedBonus> = use_bonus
    .map(|b| -> Result<UsedBonus> {
        let problem = if let Some(pid) = &bonus_from {
            let p = ps.iter().find(|p| p.0 == *pid).ok_or_else(|| {
                anyhow!(
                    "Problem {} does not provide bonus {} for problem {}",
                    pid,
                    b,
                    problem_id
                )
            })?;

            p.0
        } else {
            let avails = ps
                .iter()
                .filter(|(_, p)| {
                    p.bonuses
                        .iter()
                        .any(|bonus| bonus.bonus == b && bonus.problem as i64 == problem_id)
                })
                .collect_vec();

            if avails.is_empty() {
                bail!("{} for problem {} is not available", b, problem_id);
            }

            eprintln!(
                "Problem {:?} provide {} for problem {}",
                avails
                    .iter()
                    .map(|r| r.0.to_string())
                    .collect_vec()
                    .join(", "),
                b,
                problem_id
            );

            eprintln!("Use bonus from Problem {}", avails[0].0);

            avails[0].0
        };

        Ok(UsedBonus {
            bonus: b,
            problem: problem as _,
        })
    })
    .transpose()?;

    let problem = &ps
    .iter()
    .find(|p| p.0 == problem_id)
    .ok_or_else(|| anyhow!("Problem {} does not exist", problem_id))?
    .1;

    for gb in get_bonuses_unimplemented.iter() {
        if !problem.bonuses.iter().any(|b| b.bonus == *gb) {
            bail!("Problem {} does not provide bonus {}", problem_id, gb);
        }
    }

    let init_state: Option<Pose> = base_solution.map(|frompath| {
        let solution: Pose =
            serde_json::from_reader(
                File::open(&frompath).expect(&format!("{} is not found", frompath.display())),
            )
            .expect("invalid json file");
        solution
    });
    eprintln!("Starting solver");
    let mut fixed_vertices = Vec::new();
    match search_vertices {
        Some(vs) => {
            fixed_vertices = {0usize..problem.figure.vertices.len()}.into_iter().collect();
            fixed_vertices.retain(|i| !vs.contains(i))
        },
        None => ()
    };
    eprintln!("Starting solver");
    let (score, solution) = brute(
        &brute::Problem {
            problem: problem.clone(),
            init_state,
            use_bonus,
            get_bonuses: get_bonuses_unimplemented,
            fixed_vertices,
        });

    //let solution = Solution { vertices: solution };

    if score >= 99999999usize {
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
    let solution_filename = format!(
        "results/{}-{:02}{:02}{:02}{:02}.json",
        problem_id,
        now.date().day(),
        now.time().hour(),
        now.time().minute(),
        now.time().second(),
    );

    fs::write(&solution_filename, serde_json::to_string(&solution)?)?;

    eprintln!("Submitting internal dashboard");
    chun_tachyon_solver::submit_dashboard(problem_id, &solution_filename)?;

    if submit {
        eprintln!("Submitting");

        let resp = chun_tachyon_solver::submit(problem_id, &solution)?;
        eprintln!("Response: {:?}", resp);
    }

    Ok(())
}

fn get_problems() -> Result<Vec<(i64, P)>> {
    let mut ret = vec![];
    for rd in fs::read_dir("./problems")? {
        let rd = rd?;

        let path = rd.path();
        if !matches!(path.extension(), Some(ext) if ext == "problem") {
            continue;
        }

        eprintln!("Parsing {}", path.display());
        let problem: P = serde_json::from_reader(File::open(&path)?)?;
        let problem_id = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<i64>()
            .map_err(|_| anyhow!("{} is not valid problem filename", path.display()))?;

        ret.push((problem_id, problem));
    }

    Ok(ret)
}

#[argopt::subcmd]
fn submit(problem_id: i64, json_file: PathBuf) -> Result<()> {
    let solution = serde_json::from_reader(File::open(json_file)?)?;
    let resp = chun_tachyon_solver::submit(problem_id, &solution)?;
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
