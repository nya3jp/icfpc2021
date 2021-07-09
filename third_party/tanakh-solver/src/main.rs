mod sa;

use std::cmp::{max, min};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{Datelike, Timelike};
use itertools::Itertools;
use num_complex::Complex;
use rand::Rng;
use sa::*;
use tanakh_solver::geom::{contains, is_inside_hole, is_valid_solution, ContainsResult};
use tanakh_solver::{get_problem, Problem, Solution};

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

impl Annealer for Problem {
    type State = Vec<(i64, i64)>;

    type Move = (usize, (i64, i64));

    fn init_state(&self, rng: &mut impl rand::Rng) -> Self::State {
        // let ix = rng.gen_range(0..self.hole.len());

        // (0..self.figure.vertices.len())
        //     .map(|_| {
        //         /*self.hole[rng.gen_range(0..self.hole.len())].clone()*/
        //         self.hole[ix].clone()
        //     })
        //     .collect_vec()

        loop {
            let mut minx = i64::MAX;
            let mut maxx = i64::MIN;
            let mut miny = i64::MAX;
            let mut maxy = i64::MIN;

            for &(x, y) in self.hole.iter() {
                minx = min(minx, x);
                maxx = max(maxx, x);
                miny = min(miny, y);
                maxy = max(maxy, y);
            }

            let ret = (0..self.figure.vertices.len())
                .map(|_| loop {
                    let x = rng.gen_range(minx..=maxx);
                    let y = rng.gen_range(miny..=maxy);

                    if contains(&self.hole, &(x, y)) != ContainsResult::OUT {
                        break (x, y);
                    }
                })
                .collect_vec();

            if is_inside_hole(self, &ret) {
                break ret;
            }
        }
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        init_score / 10.0
    }

    fn is_done(&self, score: f64) -> bool {
        score < 1e-10
    }

    fn eval(&self, state: &Self::State) -> f64 {
        let mut score = 0.0;
        let mut pena = 0.0;

        let eps = self.epsilon as f64 / 1_000_000.0;

        for &(i, j) in self.figure.edges.iter() {
            let d1 = norm_sqr(&pt_sub(&state[i], &state[j]));
            let d2 = norm_sqr(&pt_sub(&self.figure.vertices[i], &self.figure.vertices[j]));
            let err = ((d1 as f64 / d2 as f64) - 1.0).abs();

            if err <= eps {
                continue;
            }

            // score += 500.0 * (err / eps);
            // score += 1000.0 * (err / eps).powi(2);
            pena += (err / eps).powf(1.0);
        }

        for h in self.hole.iter() {
            score += state.iter().map(|v| norm_sqr(&pt_sub(v, h))).min().unwrap() as f64
        }

        score * (1.0 + pena / 10.0) + pena * 500.0
        // score
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl rand::Rng,
        progress_ratio: f64,
    ) -> Self::Move {
        let w = max(2, (4.0 * (1.0 - progress_ratio)).round() as i64);

        loop {
            let i = rng.gen_range(0..state.len());

            if !self.exact {
                let dx = rng.gen_range(-w..=w);
                let dy = rng.gen_range(-w..=w);
                if (dx, dy) == (0, 0) {
                    continue;
                }

                state[i].0 += dx;
                state[i].1 += dy;

                let ok = is_inside_hole(self, &state);

                state[i].0 -= dx;
                state[i].1 -= dy;

                if !ok {
                    continue;
                }

                break (i, (dx, dy));
            } else {
                let j = rng.gen_range(0..self.hole.len());
                if state[i] == self.hole[j] {
                    continue;
                }

                let t = state[i];
                state[i] = self.hole[j];
                let ok = is_inside_hole(self, &state);
                state[i] = t;

                if !ok {
                    continue;
                }

                break (
                    i,
                    (self.hole[j].0 - state[i].0, self.hole[j].1 - state[i].1),
                );
            }
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        state[mov.0].0 += mov.1 .0;
        state[mov.0].1 += mov.1 .1;
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        state[mov.0].0 -= mov.1 .0;
        state[mov.0].1 -= mov.1 .1;
    }
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

    /// number of restart
    //
    #[opt(long, default_value = "1")]
    restart: usize,

    /// search only optimal solution
    //
    #[opt(long)]
    exact: bool,

    #[opt(long)] submit: bool,
    problem_id: i64,
) -> Result<()> {
    let seed = rand::thread_rng().gen();

    let mut problem = get_problem(problem_id)?;
    problem.exact = exact;

    let (score, solution) = annealing(
        &problem,
        &AnnealingOptions {
            time_limit,
            limit_temp: 1.0,
            restart,
            threads,
            silent: false,
        },
        seed,
    );

    let solution = Solution { vertices: solution };

    if score.is_infinite() || (score.round() - score).abs() > 1e-10 {
        eprintln!("Cannot find solution");
        eprintln!(
            "Wrong solution: score = {}, {}",
            score,
            serde_json::to_string(&solution)?
        );
        return Ok(());
    }

    if !is_valid_solution(&problem, &solution.vertices) {
        eprintln!("Validation failed");
        eprintln!(
            "Wrong solution: score = {}, {}",
            score,
            serde_json::to_string(&solution)?
        );
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

        let resp = tanakh_solver::submit(problem_id, &solution)?;
        eprintln!("Response: {:?}", resp);
    }

    Ok(())
}

#[argopt::subcmd]
fn submit(problem_id: i64, json_file: PathBuf) -> Result<()> {
    let solution = serde_json::from_reader(File::open(json_file)?)?;
    let resp = tanakh_solver::submit(problem_id, &solution)?;
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
