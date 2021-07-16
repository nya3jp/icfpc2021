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

#[macro_use]
extern crate prettytable;

mod sa;

use geom::schema::BonusType;
use geom::schema::UsedBonus;
use std::cmp::{max, min, Reverse};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use chrono::{Datelike, Timelike};
use easy_scraper::Pattern;
use itertools::Itertools;
use rand::Rng;
use reqwest::blocking::ClientBuilder;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::header::HeaderValue;
use sa::*;

use symbolic_rudolf::{get_solutions, get_problem, SolutionDashboard, ProblemDashboard, ProblemDashboardData};

#[derive(Clone, Debug, Copy)]
struct BonusGive {
    bonus: BonusType,
    problemid: usize
}

/*
#[derive(Clone, Debug)]
struct BonusUse {
    bonus: BonusType
}*/

#[derive(Clone, Debug)]
struct Problem {
    solution_ids: Vec<Vec<usize>>,
    scores: Vec<Vec<i64>>,
    give_bonuses: Vec<Vec<Vec<BonusGive>>>,
    req_bonuses: Vec<Vec<Option<BonusType>>>,
    start_temp: Option<f64>,
}

impl Problem {
    const VIOLATION_PENALTY: i64 = 1000000;
}

#[derive(Clone, Debug)]
struct State {
    problem: Problem,
    solutionix_of_problemid: Vec<usize>
}

impl State {

    fn new(problem: &Problem) -> Self {
        let mut solution_chosen = Vec::new();
        solution_chosen.push(0usize);
        for probid in 1usize..problem.solution_ids.len() {
            let mut nonreqbonus = 0usize;
            let mut nonreqbest = 0;
            for (i, req) in problem.req_bonuses[probid].iter().enumerate() {
                if req.is_none() {
                    if problem.scores[probid][i] > nonreqbest {
                        nonreqbonus = i;
                        nonreqbest = problem.scores[probid][i]
                    }

                }
            }
            solution_chosen.push(nonreqbonus);
        }
        State { problem: problem.clone(), solutionix_of_problemid: solution_chosen }
    }

    fn score(&self) -> i64 {
        self.solutionix_of_problemid.iter().enumerate().fold(0, |acc, (probid, solix)| if probid == 0 { acc } else { acc + self.problem.scores[probid][*solix] })
    }
    
    fn violation(&self) -> i64 {
        // get obtained bonuses
        let mut obtained_bonuses: Vec<Vec<BonusType>> = Vec::new();
        let problimit = self.solutionix_of_problemid.len();
        obtained_bonuses.resize(problimit, Vec::new());
        for (probid, solix) in self.solutionix_of_problemid.iter().enumerate() {
            if probid == 0 { continue; }
            let bgive = &self.problem.give_bonuses[probid][*solix];
            for b in bgive.iter() {
                if b.problemid >= problimit { continue; }
                obtained_bonuses[b.problemid].push(b.bonus); 
            }
        }
        let mut violations = 0;
        for (probid, solix) in self.solutionix_of_problemid.iter().enumerate() {
            if probid == 0 { continue; }
            let breq = &self.problem.req_bonuses[probid][*solix];
            if breq.is_none() {
                continue;
            }
            if !obtained_bonuses[probid].contains(&breq.as_ref().unwrap()) {
                violations += 1
            }
        }
        violations
    }
}

#[derive(Clone, Debug)]
struct Move {
    probid: usize,
    from: usize,
    to: usize
}

impl Annealer for Problem {

    type State = State;

    type Move = Move;

    fn init_state(&self, rng: &mut impl rand::Rng) -> Self::State {
        State::new(&self)
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        // self.start_temp
        //     .unwrap_or_else(|| (init_score / 100.0).max(self.penalty_ratio))

        self.start_temp
            .unwrap_or_else(|| (-init_score / 10.0).max(100.0))
    }

    fn is_done(&self, score: f64) -> bool {
        score < 1e-10
    }

    fn eval(&self, state: &Self::State, _best_score: f64, _valid_best_score: f64) -> (f64, bool) {
        let violation = state.violation();
        let score = -state.score() + Problem::VIOLATION_PENALTY * state.violation(); // score: higher the better, annealer: lower the better
        let score = score as f64;
        (score, violation == 0)
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl rand::Rng,
        progress_ratio: f64,
    ) -> Self::Move {
        loop {
            let rpos = rng.gen_range(1usize..state.solutionix_of_problemid.len());
            let qpos = rng.gen_range(0usize..state.problem.scores[rpos].len());
            if qpos == state.solutionix_of_problemid[rpos] { continue; }
            return Move { probid: rpos, from: state.solutionix_of_problemid[rpos], to: qpos };
        }

    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        state.solutionix_of_problemid[mov.probid] = mov.to;
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        state.solutionix_of_problemid[mov.probid] = mov.from;
    }
}

fn calc_score(probdash: &ProblemDashboard, ourdislike: usize) -> i64 {
    let bestdislike = probdash.minimal_dislike as f64;
    let ourdislike = ourdislike as f64;
    let nvertices = probdash.data.figure.vertices.len() as f64;
    let nedges = probdash.data.figure.edges.len() as f64;
    let nholes = probdash.data.hole.polygon.vertices.len() as f64;
    return (1000. * (nvertices * nedges * nholes / 6.).log2() * ((bestdislike + 1.) / (ourdislike + 1.)).sqrt()).ceil() as i64
}

#[argopt::subcmd]
fn solve(
    /// time limit in seconds
    #[opt(long, default_value = "30.0")]
    time_limit: f64,

    /// seed
    #[opt(long)]
    seed: Option<u64>,

    #[opt(long)] start_temp: Option<f64>,
    #[opt(long, default_value = "0.25")] min_temp: f64,

    #[opt(long, default_value = "132")]
    problem_id_upto: usize,
) -> Result<()> {
    let problems: Vec<usize> = (0usize..=problem_id_upto).collect();
    let mut solution_ids: Vec<Vec<usize>> = Vec::new();
    let mut scores: Vec<Vec<i64>> = Vec::new();
    let mut give_bonuses: Vec<Vec<Vec<BonusGive>>> = Vec::new();
    let mut req_bonuses: Vec<Vec<Option<BonusType>>> = Vec::new();

    // rust wakaran node hidoi code
    for probid in 0usize..=problem_id_upto {
        solution_ids.push(Vec::new());
        scores.push(Vec::new());
        give_bonuses.push(Vec::new());
        req_bonuses.push(Vec::new());
    }

    for probid in 1usize..=problem_id_upto {
        let prob: ProblemDashboard = get_problem(probid as i64).unwrap();
        let sol: Vec<SolutionDashboard> = get_solutions(probid as i64).unwrap();
        for s in sol {
            let score = calc_score(&prob, s.dislike);
            let s_req_bonus : Option<BonusType> = 
                match s.data.bonuses {
                    None => None,
                    Some (v) if v.len() == 0 => None,
                    Some (v) =>
                        Some(v[0].bonus.clone())
                };
            let s_give_bonuses: Vec<BonusGive> = {
                prob.data.bonuses.iter().map(|b| 
                    BonusGive { bonus: b.bonus, problemid: b.problem }
                ).collect()
            };
            solution_ids[probid].push(s.solution_id);
            scores[probid].push(score);
            req_bonuses[probid].push(s_req_bonus);
            give_bonuses[probid].push(s_give_bonuses);
        }
        //eprintln!("{:?}", prob);
        //eprintln!("{:?}", sol);
        ()
    }
    let problem: Problem = Problem {
        solution_ids,
        scores,
        give_bonuses,
        req_bonuses,
        start_temp
    };
    let seed = seed.unwrap_or_else(|| rand::thread_rng().gen());
    let res = annealing(
        &problem,
        &AnnealingOptions {
            time_limit,
            limit_temp: min_temp,
            restart: 0,
            threads: 1,
            silent: false,
            header: "Final opt".to_string(),
        },
        seed,
    );

    if res.is_none() {
        eprintln!("Could not find solution");
        return Ok(());
    }

    let (score, solution) = res.unwrap();

    let finalset = solution.solutionix_of_problemid;
    for (probid, probidix) in finalset.iter().enumerate() {
        if probid == 0 { continue; }
        print!("{},", problem.solution_ids[probid][*probidix]);
    }
    println!("");

    Ok(())
}

#[argopt::cmd_group(commands = [solve])]
fn main() -> Result<()> {}
