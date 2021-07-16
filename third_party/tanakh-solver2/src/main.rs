// Copyright 2021 Team Special Weekend
// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod sa;

use std::fs::File;
use std::os::linux::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use std::collections::BTreeMap;

use anyhow::Result;
use chrono::{Datelike, Timelike};
use itertools::Itertools;
use num_complex::Complex;
use sa::*;
use tanakh_solver::geom::is_inside_hole;
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

static mut HINT_POS: Option<BTreeMap<usize, usize>> = None;

fn read_hint<P: AsRef<Path>>(path: P) -> Result<BTreeMap<usize, usize>> {
    let v: Vec<(usize, usize)> = serde_json::from_reader(File::open(path)?)?;
    let mut m = BTreeMap::new();
    for (i1, i2) in &v {
        m.insert(*i1, *i2);
    }
    Ok(m)
}

fn check_hint(problem: &Problem, assignment: &BTreeMap<usize, usize>) -> bool {
    let eps = problem.epsilon as f64 / 1e6;
    for e in &problem.figure.edges {
        if !assignment.contains_key(&e.0) || !assignment.contains_key(&e.1) {
            continue;
        }
        let p1 = &problem.hole[*assignment.get(&e.0).unwrap()];
        let p2 = &problem.hole[*assignment.get(&e.1).unwrap()];
        let q1 = &problem.figure.vertices[e.0];
        let q2 = &problem.figure.vertices[e.1];
        let d1 = norm_sqr(&pt_sub(p1, p2));
        let d2 = norm_sqr(&pt_sub(q1, q2));
        let err = ((d1 as f64 / d2 as f64) - 1.0).abs();
        if err > eps {
            return false;
        }
    }
    true
}

fn find_hint_dfs(problem: &Problem, assignment: &mut BTreeMap<usize, usize>, result: &mut Vec<BTreeMap<usize, usize>>) {
    let level = assignment.len();
    if level == problem.hole.len() {
        result.push(assignment.clone());
        return;
    }

    for i in 0..problem.figure.vertices.len() {
        if assignment.contains_key(&i) {
            continue;
        }

        assignment.insert(i, level);
        if check_hint(problem, assignment) {
            find_hint_dfs(problem, assignment, result);
        }
        assignment.remove(&i);
    }
}

fn find_hint(problem: &Problem) -> Vec<BTreeMap<usize, usize>> {
    let mut result = Vec::new();
    let mut assignment = BTreeMap::new();
    find_hint_dfs(problem, &mut assignment, &mut result);
    result
}

impl Annealer for Problem {
    type State = Vec<(i64, i64)>;

    type Move = (usize, (i64, i64));

    fn init_state(&self, _rng: &mut impl rand::Rng) -> Self::State {
        let m = unsafe { HINT_POS.as_ref() };
        (0..self.figure.vertices.len())
            .map(|i| self.hole[m.map_or(0, |m| *m.get(&i).unwrap_or(&0))].clone())
            .collect_vec()
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        init_score / 10.0
    }

    fn eval(&self, state: &Self::State) -> f64 {
        let mut score = 0.0;

        let eps = self.epsilon as f64 / 1_000_000.0;

        for &(i, j) in self.figure.edges.iter() {
            let d1 = norm_sqr(&pt_sub(&state[i], &state[j]));
            let d2 = norm_sqr(&pt_sub(&self.figure.vertices[i], &self.figure.vertices[j]));
            let err = ((d1 as f64 / d2 as f64) - 1.0).abs();

            if err <= eps {
                continue;
            }

            score += err / eps * 1000.0;
        }

        for h in self.hole.iter() {
            score += state.iter().map(|v| norm_sqr(&pt_sub(v, h))).min().unwrap() as f64
        }

        score
    }

    fn neighbour(&self, state: &Self::State, rng: &mut impl rand::Rng) -> Self::Move {
        // FIXME: performance
        let mut state = state.clone();
        let m = unsafe { HINT_POS.as_ref() };

        loop {
            let i = rng.gen_range(0..state.len());
            if m.map_or(false, |m| m.contains_key(&i)) {
                continue;
            }
            let dx = rng.gen_range(-2..=2);
            let dy = rng.gen_range(-2..=2);
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

    #[opt(long)] submit: bool,
    problem_id: i64,

    #[opt(long)]
    hint: Option<PathBuf>,
) -> Result<()> {
    let problem = get_problem(problem_id)?;

    let hints = {
        if hint.is_some() {
            vec![read_hint(hint.unwrap())?]
        } else {
            find_hint(&problem)
        }
    };
    eprintln!("hints: {:?}", hints);

    let mut min_score = None;
    let mut min_solution = None;

    for i in 0..hints.len() {
        eprintln!("Trial {:?}/{:?}\n", i+1, hints.len());
        unsafe { HINT_POS = Some(hints[i].clone()) }
        let (score, solution) = annealing(
            &problem,
            &AnnealingOptions {
                time_limit,
                limit_temp: 0.1,
                restart: 1,
                threads,
                silent: false,
            },
            777,
        );
        if min_score.is_none() || min_score.unwrap() > score {
            min_score = Some(score);
            min_solution = Some(solution);
            if score == 0. {
                break;
            }
        }
    }
    let score = min_score.unwrap();
    let solution = min_solution.unwrap();

    let solution = Solution { vertices: solution };

    if score.is_infinite() || (score.round() - score).abs() > 1e-10 {
        eprintln!("Cannot find solution");
        eprintln!("Wrong solution: {}", serde_json::to_string(&solution)?);
        return Ok(());
    }

    eprintln!("Score: {}", score);

    println!("{}", serde_json::to_string(&solution)?);

    if !Path::new("results").exists() {
        std::fs::create_dir_all("results")?;
    }

    let now = chrono::Local::now();
    std::fs::write(
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
