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

use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::sa::*;
use anyhow::{anyhow, bail, Result};
use chrono::{Datelike, Timelike};
use geom::{
    point::Point,
    schema::{BonusType, Edge, Pose, Problem as P, UsedBonus},
};
use itertools::Itertools;
use rand::Rng;
use scorer::{is_inside_hole, is_inside_hole_partial, is_valid_solution};
use tanakh_solver::{get_problem_states, get_problems};

#[argopt::subcmd]
pub fn solve(
    /// time limit in seconds
    #[opt(long, default_value = "5.0")]
    time_limit: f64,

    /// number of threads
    #[opt(long, default_value = "1")]
    threads: usize,

    /// number of restart
    #[opt(long, default_value = "1")]
    restart: usize,

    /// seed
    #[opt(long)]
    seed: Option<u64>,

    /// search around optimal solution
    #[opt(long)]
    exact: bool,

    /// Find the hole->node mapping at the beginning.
    #[opt(long)]
    use_hint: bool,

    /// Use specified initial state
    #[opt(long)]
    init_state: Option<PathBuf>,

    #[opt(long)] start_temp: Option<f64>,
    #[opt(long, default_value = "0.25")] min_temp: f64,

    /// Penalty deflating ratio (0.0 - 1.0). Please set this value to around minimal dislike value (e.g. max(min_dislike/2, 10)).
    #[opt(long, default_value = "1000.0")]
    penalty_ratio: f64,

    /// Penalty deflating ratio (0.0 - 1.0). Larger value means more agressive (but unstable) search.
    #[opt(long, default_value = "0.25")]
    penalty_deflate: f64,

    #[opt(long)] no_submit: bool,
    #[opt(long)] submit_on_better: bool,

    /// Bonus to use (one of "GLOBALIST", "BREAK_A_LEG", "WALLHACK")
    #[opt(long)]
    use_bonus: Option<BonusType>,

    #[opt(long)] bonus_from: Option<i64>,

    /// Bonuses to get specified by problem id
    #[opt(long)]
    get_bonuses: Vec<i64>,

    /// search parallel moves
    #[opt(long)]
    parallel: bool,

    problem_id: i64,
) -> Result<()> {
    let ps = get_problems()?;

    let use_bonus: Option<UsedBonus> = match (use_bonus, bonus_from) {
        (None, None) => None,

        (Some(b), None) => {
            // bonus support check
            match &use_bonus {
                None => (),
                Some(BonusType::GLOBALIST) => (),
                Some(BonusType::SUPERFLEX) => (),
                Some(r) => {
                    bail!("Bonus {} is currently not supported", r);
                }
            }

            let problem = {
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

                avails[0].0
            };

            Some(UsedBonus {
                bonus: b,
                problem: problem as _,
                edge: None,
            })
        }

        (None, Some(pid)) => {
            let p = ps
                .iter()
                .find_map(|p| {
                    if p.0 == pid {
                        if let Some(bonus) =
                            p.1.bonuses
                                .iter()
                                .find(|bonus| bonus.problem as i64 == problem_id)
                        {
                            return Some((p.0, bonus));
                        }
                    }
                    None
                })
                .ok_or_else(|| {
                    anyhow!(
                        "Problem {} does not provide bonus for problem {}",
                        pid,
                        problem_id
                    )
                })?;

            Some(UsedBonus {
                bonus: p.1.bonus,
                problem: p.0 as _,
                edge: None,
            })
        }

        (Some(_), Some(_)) => bail!("Cannot specify both 'use_bonus' and 'bonus_from'"),
    };

    if let Some(b) = &use_bonus {
        eprintln!("Use bonus {} from problem {}", b.bonus, b.problem);
    }

    // let problem: P = get_problem(problem_id)?;
    let problem = &ps
        .iter()
        .find(|p| p.0 == problem_id)
        .ok_or_else(|| anyhow!("Problem {} does not exist", problem_id))?
        .1;
    let seed = seed.unwrap_or_else(|| rand::thread_rng().gen());

    for gb in get_bonuses.iter() {
        if !problem.bonuses.iter().any(|b| b.problem as i64 == *gb) {
            bail!(
                "Problem {} does not provide bonus for problem {}",
                problem_id,
                gb
            );
        }
    }

    let mut triangles = vec![];

    for i in 0..problem.figure.vertices.len() {
        for j in 0..problem.figure.vertices.len() {
            for k in 0..problem.figure.vertices.len() {
                if problem.figure.edges.contains(&Edge::new(i, j))
                    && problem.figure.edges.contains(&Edge::new(j, k))
                    && problem.figure.edges.contains(&Edge::new(k, i))
                {
                    triangles.push((i, j, k));
                }
            }
        }
    }

    eprintln!("Start annealing seed: {}", seed);
    eprintln!("Problem contains {} triangles", triangles.len());

    let mut hints = Vec::new();
    if use_hint {
        hints = find_hint(&problem);
        // eprintln!("Use hints: {:?}", hints);
    } else {
        hints.push(BTreeMap::new());
    }

    let init_state: Option<Pose> = init_state.map(|path| {
        serde_json::from_reader(
            File::open(&path).expect(&format!("{} is not found", path.display())),
        )
        .expect("invalid json file")
    });

    let mut min_score = None;
    let mut min_solution = None;
    for i in 0..hints.len() {
        eprintln!("Trial: {}/{}", i + 1, hints.len());
        // eprintln!("Trial: {}/{}: {:?}", i + 1, hints.len(), hints[i]);

        let hint = hints[i].clone();
        let problem = Problem {
            problem: problem.clone(),
            use_bonus: use_bonus.clone(),
            get_bonuses: get_bonuses.clone(),
            exact,
            parallel,
            penalty_ratio,
            penalty_deflate,
            triangles: triangles.clone(),
            fixed_points: hint.clone(),
            init_state: init_state.clone(),
            start_temp: start_temp.clone(),
            candidate_vertices: filter_vertices(&problem.figure.vertices, &hint),
            candidate_edges: filter_edges(&problem.figure.edges, &hint),
            candidate_triangles: filter_triangles(&triangles, &hint),
        };

        let res = annealing(
            &problem,
            &AnnealingOptions {
                time_limit,
                limit_temp: min_temp,
                restart,
                threads,
                silent: false,
                header: format!("Problem {}: ", problem_id),
            },
            seed,
        );

        if res.is_none() {
            eprintln!("Could not find solution");
            continue;
        }

        let (score, solution) = res.unwrap();

        let solution = solution.pose;

        if !is_valid_solution(&problem.problem, &solution) {
            eprintln!("Validation failed");
            eprintln!(
                "Wrong solution: score = {}, {}",
                score,
                serde_json::to_string(&solution)?
            );
        }

        if min_score.is_none() || min_score.unwrap() > score {
            min_score = Some(score);
            min_solution = Some(solution);
            if score == 0. {
                break;
            }
        }
    }

    if min_score.is_none() {
        return Ok(());
    }

    let score = min_score.unwrap();
    let solution = min_solution.unwrap();

    eprintln!("Score for problem {}: {}", problem_id, score);

    println!("{}", serde_json::to_string(&solution)?);

    if !Path::new("results").exists() {
        fs::create_dir_all("results")?;
    }

    let now = chrono::Local::now();
    let solution_filename = format!(
        "results/{}-{}-{:02}{:02}{:02}{:02}.json",
        problem_id,
        score.round() as i64,
        now.date().day(),
        now.time().hour(),
        now.time().minute(),
        now.time().second(),
    );

    fs::write(&solution_filename, serde_json::to_string(&solution)?)?;

    eprintln!("Wrote the solution to {}", solution_filename);

    // Submit to the internal dashboard.
    eprintln!("Submitting internal dashboard");
    tanakh_solver::submit_dashboard(problem_id, &solution_filename)?;

    if no_submit {
        return Ok(());
    }

    let problems = get_problem_states()?;
    let problem = problems.iter().find(|r| r.problem_id == problem_id);
    let mut better = false;

    if let Some(problem) = problem {
        eprintln!(
            "Dislike: {}, Your previous dislike: {}, Minimal dislike: {}",
            score as i64, problem.your_dislikes, problem.minimal_dislikes
        );
        if (score as i64) < problem.your_dislikes {
            better = true;
        }
    } else {
        eprintln!("No submission for problem {} found.", problem_id);
    }

    if (submit_on_better && better)
        || dialoguer::Confirm::new()
            .with_prompt("Submit?")
            .interact()?
    {
        eprintln!("Submitting");

        let resp = tanakh_solver::submit(problem_id, &solution)?;
        eprintln!("Response: {:?}", resp);
    }

    Ok(())
}

fn find_hint_dfs(
    problem: &P,
    assignment: &mut BTreeMap<usize, Point>,
    result: &mut Vec<BTreeMap<usize, Point>>,
) {
    let level = assignment.len();
    if level == problem.hole.len() {
        result.push(assignment.clone());
        return;
    }

    for i in 0..problem.figure.vertices.len() {
        if assignment.contains_key(&i) {
            continue;
        }

        assignment.insert(i, problem.hole[level]);
        if check_hint(problem, assignment) {
            find_hint_dfs(problem, assignment, result);
        }
        assignment.remove(&i);
    }
}

fn find_hint(problem: &P) -> Vec<BTreeMap<usize, Point>> {
    let mut result = Vec::new();
    let mut assignment = BTreeMap::new();
    find_hint_dfs(problem, &mut assignment, &mut result);
    result
}

#[derive(Clone)]
struct Problem {
    problem: P,
    use_bonus: Option<UsedBonus>,
    get_bonuses: Vec<i64>,
    penalty_ratio: f64,
    penalty_deflate: f64,
    exact: bool,
    parallel: bool,
    triangles: Vec<(usize, usize, usize)>,
    init_state: Option<Pose>,
    start_temp: Option<f64>,
    fixed_points: BTreeMap<usize, Point>, // From |node| to |point|.
    candidate_vertices: Vec<usize>,
    candidate_edges: Vec<usize>,
    candidate_triangles: Vec<usize>,
}

#[derive(Clone)]
struct State {
    pose: Pose,
    dislikes: Vec<i64>,
}

impl State {
    fn new(pose: Pose, problem: &P) -> Self {
        let dislikes = problem
            .hole
            .iter()
            .map(|h| {
                pose.vertices
                    .iter()
                    .map(|v| (v - h).norm_sqr().round() as i64)
                    .min()
                    .unwrap()
            })
            .collect_vec();

        Self {
            pose: pose,
            dislikes,
        }
    }

    fn dislike(&self) -> i64 {
        self.dislikes.iter().sum()
    }

    fn move_vertex(&mut self, i: usize, v: &Point) {
        self.pose.vertices[i] += v;
    }

    fn move_vertex_and_update_dislike(&mut self, i: usize, v: &Point, problem: &P) {
        let old_pos = self.pose.vertices[i];

        self.move_vertex(i, v);

        for (j, h) in problem.hole.iter().enumerate() {
            let prev_dislike = self.dislikes[j];
            let new_dislike = (&self.pose.vertices[i] - h).norm_sqr().round() as i64;
            let old_dislike = (&old_pos - h).norm_sqr().round() as i64;

            if new_dislike <= prev_dislike {
                self.dislikes[j] = new_dislike;
            } else if prev_dislike == old_dislike {
                self.dislikes[j] = self
                    .pose
                    .vertices
                    .iter()
                    .map(|v| (v - h).norm_sqr().round() as i64)
                    .min()
                    .unwrap();
            }
        }
    }
}

// fn calc_dislike(problem: &P, pose: &Pose) -> i64 {
//     let mut ret = 0.0;
//     for h in problem.hole.iter() {
//         ret += pose
//             .vertices
//             .iter()
//             .map(|v| (*v - *h).norm_sqr())
//             .fold(0.0 / 0.0, f64::min);
//     }
//     ret.round() as i64
// }

impl Annealer for Problem {
    type State = State;

    type Move = Vec<(usize, Point)>;

    fn init_state(&self, rng: &mut impl rand::Rng) -> Self::State {
        if let Some(init_state) = &self.init_state {
            if !is_inside_hole(&self.problem, init_state) {
                panic!("init state is not inside the hole");
            }

            return State::new(init_state.clone(), &self.problem);
        }

        let bonuses = self.use_bonus.as_ref().map(|b| vec![b.clone()]);

        let init_state = (|| {
            for _ in 0..100 {
                let mut minx = i64::MAX;
                let mut maxx = i64::MIN;
                let mut miny = i64::MAX;
                let mut maxy = i64::MIN;

                for p in self.problem.hole.iter() {
                    minx = min(minx, p.x as i64);
                    maxx = max(maxx, p.x as i64);
                    miny = min(miny, p.y as i64);
                    maxy = max(maxy, p.y as i64);
                }

                let ret = (0..self.problem.figure.vertices.len())
                    .map(|_| loop {
                        let x = rng.gen_range(minx..=maxx);
                        let y = rng.gen_range(miny..=maxy);

                        if self.problem.hole.contains(&Point::new(x as _, y as _)) {
                            break Point::new(x as _, y as _);
                        }
                    })
                    .collect_vec();

                let ret = Pose {
                    vertices: ret,
                    bonuses: bonuses.clone(),
                };

                if is_inside_hole(&self.problem, &ret) {
                    return ret;
                }
            }

            // fallback

            let ix = rng.gen_range(0..self.problem.hole.len());

            let default_point = self.problem.hole[ix].clone();

            let ret = (0..self.problem.figure.vertices.len())
                .map(|i| {
                    *self.fixed_points.get(&i).unwrap_or(&default_point.clone())
                    /*self.hole[rng.gen_range(0..self.hole.len())].clone()*/
                })
                .collect_vec();

            let init_state = Pose {
                vertices: ret,
                bonuses,
            };

            if !is_inside_hole(&self.problem, &init_state) {
                eprintln!("Wrong Answer!!");
            }

            init_state
        })();

        State::new(init_state, &self.problem)
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        // self.start_temp
        //     .unwrap_or_else(|| (init_score / 100.0).max(self.penalty_ratio))

        self.start_temp
            .unwrap_or_else(|| (init_score / 10.0).max(100.0))
    }

    fn is_done(&self, score: f64) -> bool {
        score < 1e-10
    }

    fn eval(&self, state: &Self::State, _best_score: f64, _valid_best_score: f64) -> (f64, bool) {
        let mut pena = 0.0;
        let mut invalid_edge_count = 0;
        let mut max_invalid_err = 0.0_f64;
        let mut total_err = 0.0;

        // let penalty_ratio = if valid_best_score.is_finite() {
        //     valid_best_score / 2.0
        // } else if best_score.is_finite() {
        //     best_score / 2.0
        // } else {
        //     self.penalty_ratio
        // }
        // .clamp(4.0, 10000.0);

        let penalty_ratio = self.penalty_ratio;

        let eps = self.problem.epsilon as f64 / 1_000_000.0;

        for edge in self.problem.figure.edges.iter() {
            let i = edge.v1;
            let j = edge.v2;

            let d1 = (state.pose.vertices[i] - state.pose.vertices[j]).norm_sqr();
            let d2 = (self.problem.figure.vertices[i] - self.problem.figure.vertices[j]).norm_sqr();
            let err = ((d1 as f64 / d2 as f64) - 1.0).abs();

            total_err += err;

            if err <= eps {
                continue;
            }

            invalid_edge_count += 1;

            // score += 500.0 * (err / eps);
            // score += 1000.0 * (err / eps).powi(2);
            let e = (err / eps - self.penalty_deflate).abs();

            // pena += (err / eps - 1.0).powi(2);
            // pena += (err / eps - 0.90).abs().powf(0.75);

            pena += e;
            max_invalid_err = max_invalid_err.max(e);
        }

        let dislike = state.dislike() as f64;

        let mut bonus_err: i64 = 0;

        for bonus in self.get_bonuses.iter() {
            let bonus_pos = self
                .problem
                .bonuses
                .iter()
                .find(|b| b.problem as i64 == *bonus)
                .unwrap()
                .position;

            let min_dist = state
                .pose
                .vertices
                .iter()
                .map(|p| (p - &bonus_pos).norm_sqr().round() as i64)
                .min()
                .unwrap();

            bonus_err += min_dist;
        }
        let mut is_valid = bonus_err == 0;

        let globalist = matches!(
            &self.use_bonus,
            Some(UsedBonus {
                bonus: geom::schema::BonusType::GLOBALIST,
                ..
            })
        );

        let superflex = matches!(
            &self.use_bonus,
            Some(UsedBonus {
                bonus: geom::schema::BonusType::SUPERFLEX,
                ..
            })
        );

        if globalist {
            let total_eps = eps * self.problem.figure.edges.len() as f64;
            is_valid = is_valid && total_err <= total_eps;
            let pena = (total_err - total_eps).max(0.0) + bonus_err as f64;
            let score = dislike + pena * penalty_ratio;
            (score, is_valid)
        } else {
            let pena = if !superflex {
                pena
            } else {
                pena - max_invalid_err
            } + bonus_err as f64;

            let score = dislike * (1.0 + pena / 8.0) + pena * self.penalty_ratio;
            // let score = dislike + pena * penalty_ratio;
            // let score = dislike * (1.0 + pena / 8.0) + pena * penalty_ratio;

            is_valid = is_valid && invalid_edge_count <= if superflex { 1 } else { 0 };

            (score, is_valid)
        }
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl rand::Rng,
        progress_ratio: f64,
    ) -> Self::Move {
        let w = max(1, (4.0 * (1.0 - progress_ratio)).round() as i64);

        fn delta(rng: &mut impl rand::Rng, w: i64) -> (i64, i64) {
            loop {
                let dx = rng.gen_range(-w..=w);
                let dy = rng.gen_range(-w..=w);
                if (dx, dy) != (0, 0) {
                    break (dx, dy);
                }
            }
        }

        loop {
            match rng.gen_range(0..if self.exact { 22 } else { 21 }) {
                0..=9 => {
                    let i = rng.gen_range(0..self.candidate_vertices.len());
                    let i = self.candidate_vertices[i];

                    let (dx, dy) = delta(rng, w);

                    let d = Point::new(dx as _, dy as _);

                    state.move_vertex(i, &d);

                    let ok = is_inside_hole_partial(&self.problem, &state.pose, &[i]);

                    state.move_vertex(i, &-d);

                    if ok {
                        return vec![(i, d)];
                    }
                }
                10..=16 => {
                    let i = rng.gen_range(0..self.candidate_edges.len());
                    let e = &self.problem.figure.edges[self.candidate_edges[i]];
                    let i = e.v1;
                    let j = e.v2;

                    // let i = rng.gen_range(0..state.vertices.len());
                    // let j = rng.gen_range(0..state.vertices.len());
                    // if !self.problem.figure.edges.contains(&Edge::new(i, j)) {
                    //     continue;
                    // }

                    let (dx, dy) = delta(rng, w);

                    let d = Point::new(dx as _, dy as _);

                    state.move_vertex(i, &d);
                    state.move_vertex(j, &d);

                    let ok = is_inside_hole_partial(&self.problem, &state.pose, &[i, j]);

                    state.move_vertex(i, &-d);
                    state.move_vertex(j, &-d);

                    if ok {
                        return vec![(i, d), (j, d)];
                    }
                }
                17..=19 => {
                    if self.candidate_triangles.is_empty() {
                        continue;
                    }
                    let i = rng.gen_range(0..self.candidate_triangles.len());
                    let (i, j, k) = self.triangles[self.candidate_triangles[i]];

                    let (dx, dy) = delta(rng, w);

                    let d = Point::new(dx as _, dy as _);

                    state.move_vertex(i, &d);
                    state.move_vertex(j, &d);
                    state.move_vertex(k, &d);

                    let ok = is_inside_hole_partial(&self.problem, &state.pose, &[i, j, k]);

                    state.move_vertex(i, &-d);
                    state.move_vertex(j, &-d);
                    state.move_vertex(k, &-d);

                    if ok {
                        return vec![(i, d), (j, d), (k, d)];
                    }
                }

                20 => {
                    if !self.parallel {
                        continue;
                    }

                    // Parallel move
                    let d = rng.gen_range(-w..=w);
                    if d == 0 {
                        continue;
                    }

                    let pd = Point::new(d as _, d as _);

                    for i in 0..state.pose.vertices.len() {
                        state.move_vertex(i, &pd);
                    }

                    let ok = is_inside_hole(&self.problem, &state.pose);

                    for i in 0..state.pose.vertices.len() {
                        state.move_vertex(i, &-pd);
                    }

                    if ok {
                        return (0..state.pose.vertices.len()).map(|v| (v, pd)).collect();
                    }
                }

                _ => {
                    for _ in 0..10 {
                        let i = rng.gen_range(0..self.candidate_vertices.len());
                        let i = self.candidate_vertices[i];

                        let j = rng.gen_range(0..self.problem.hole.polygon.vertices.len());
                        if state.pose.vertices[i] == self.problem.hole.polygon.vertices[j] {
                            continue;
                        }

                        let d = self.problem.hole.polygon.vertices[j] - state.pose.vertices[i];
                        state.move_vertex(i, &d);
                        let ok = is_inside_hole_partial(&self.problem, &state.pose, &[i]);
                        state.move_vertex(i, &d);

                        if ok {
                            return vec![(i, d)];
                        }
                    }
                }
            }
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        for (i, v) in mov.iter() {
            state.move_vertex_and_update_dislike(*i, v, &self.problem);
        }
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        for (i, v) in mov.iter() {
            state.move_vertex_and_update_dislike(*i, &-*v, &self.problem);
        }
    }
}

fn filter_vertices(vertices: &Vec<Point>, hint: &BTreeMap<usize, Point>) -> Vec<usize> {
    (0..vertices.len())
        .filter(|i| !hint.contains_key(&i))
        .collect_vec()
}

fn filter_edges(edges: &Vec<Edge>, hint: &BTreeMap<usize, Point>) -> Vec<usize> {
    (0..edges.len())
        .filter(|i| !hint.contains_key(&edges[*i].v1) && !hint.contains_key(&edges[*i].v2))
        .collect_vec()
}

fn filter_triangles(
    triangles: &Vec<(usize, usize, usize)>,
    hint: &BTreeMap<usize, Point>,
) -> Vec<usize> {
    (0..triangles.len())
        .filter(|i| {
            !hint.contains_key(&triangles[*i].0)
                && !hint.contains_key(&triangles[*i].1)
                && !hint.contains_key(&triangles[*i].2)
        })
        .collect_vec()
}

// fn read_hint<P: AsRef<Path>>(path: P) -> Result<BTreeMap<usize, usize>> {
//     let v: Vec<(usize, usize)> = serde_json::from_reader(File::open(path)?)?;
//     let mut m = BTreeMap::new();
//     for (i1, i2) in &v {
//         m.insert(*i1, *i2);
//     }
//     Ok(m)
// }

fn check_hint(problem: &P, assignment: &BTreeMap<usize, Point>) -> bool {
    let eps = problem.epsilon as f64 / 1e6;
    for e in &problem.figure.edges {
        if !assignment.contains_key(&e.v1) || !assignment.contains_key(&e.v2) {
            continue;
        }
        let p1 = assignment.get(&e.v1).unwrap();
        let p2 = assignment.get(&e.v2).unwrap();
        let q1 = &problem.figure.vertices[e.v1];
        let q2 = &problem.figure.vertices[e.v2];
        let d1 = (*p1 - *p2).norm_sqr();
        let d2 = (*q1 - *q2).norm_sqr();
        let err = ((d1 as f64 / d2 as f64) - 1.0).abs();
        if err > eps {
            return false;
        }
    }
    true
}
