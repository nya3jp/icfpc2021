#[macro_use]
extern crate prettytable;

mod sa;

use std::cmp::{max, min, Reverse};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use chrono::{Datelike, Timelike};
use easy_scraper::Pattern;
use geom::{
    point::Point,
    // polygon::ContainsResult,
    schema::{BonusType, Edge, Pose, Problem as P, UsedBonus},
};
use itertools::Itertools;
use rand::Rng;
use reqwest::blocking::ClientBuilder;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::header::HeaderValue;
use sa::*;
use scorer::{is_inside_hole, is_inside_hole_partial, is_valid_solution};
use tanakh_solver::{get_problem, ENDPOINT};

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

#[argopt::subcmd]
fn solve(
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
            * ((problem.figure.vertices.len()
                * problem.figure.edges.len()
                * problem.hole.polygon.vertices.len()) as f64
                / 6.0)
                .log2();

        println!("Problem {}: {}", pid, max_score.ceil() as i64);
    }

    Ok(())
}

fn load_cookie_store(session_file: impl AsRef<Path>, endpoint: &str) -> Result<Jar> {
    let url = endpoint.parse().unwrap();
    let jar = reqwest::cookie::Jar::default();
    let f = File::open(session_file);

    if f.is_err() {
        bail!("session.txt not found. Please login first.");
    }

    for line in BufReader::new(f.unwrap()).lines() {
        let v = line?
            .split("; ")
            .map(|s| HeaderValue::from_str(s).unwrap())
            .collect_vec();
        jar.set_cookies(&mut v.iter(), &url)
    }

    Ok(jar)
}

#[argopt::subcmd]
fn login() -> Result<()> {
    let cookie_store = Arc::new(Jar::default());

    let client = ClientBuilder::new()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let email: String = dialoguer::Input::new()
        .with_prompt("Email address")
        .interact()?;
    let passwd = dialoguer::Password::new()
        .with_prompt("Password")
        .interact()?;

    let _resp = client
        .post("https://poses.live/login")
        .form(&[("login.email", &email), ("login.password", &passwd)])
        .send()?
        .error_for_status()?
        .text()?;

    {
        let mut f = File::create("session.txt")?;
        for cookie in cookie_store.cookies(&ENDPOINT.parse().unwrap()) {
            writeln!(&mut f, "{}", cookie.to_str()?)?;
        }
    }

    println!("Ok");

    Ok(())
}

struct ProblemState {
    problem_id: i64,
    your_dislikes: i64,
    minimal_dislikes: i64,
    point_ratio: f64,
    max_score: i64,
    your_score: i64,
    remaining_score: i64,
}

fn get_problem_states() -> Result<Vec<ProblemState>> {
    let cookie_store = Arc::new(load_cookie_store("session.txt", ENDPOINT)?);

    let client = ClientBuilder::new()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let resp = client
        .get("https://poses.live/problems")
        .send()?
        .error_for_status()?
        .text()?;

    let pat = Pattern::new(
        r#"
        <table>
            <tr>
                <td><a href="/problems/{{problem-id}}"></a></td>
                <td>{{your-dislikes}}</td>
                <td>{{minimal-dislikes}}</td>
            </tr>
        </table>
        "#,
    )
    .unwrap();

    let ps = get_problems()?;
    let mut problems = vec![];

    println!("{} problems", ps.len());

    for m in pat.matches(&resp) {
        let problem_id: i64 = m["problem-id"].parse()?;
        let your_dislikes = m["your-dislikes"].parse();

        let your_dislikes = your_dislikes.unwrap_or(9999999);

        let minimal_dislikes: i64 = m["minimal-dislikes"].parse()?;

        let point_ratio = (((minimal_dislikes + 1) as f64) / ((your_dislikes + 1) as f64)).sqrt();

        let problem = ps.iter().find(|r| r.0 == problem_id);

        if problem.is_none() {
            continue;
        }
        let problem = &problem.unwrap().1;

        let max_score = (1000.0
            * ((problem.figure.vertices.len()
                * problem.figure.edges.len()
                * problem.hole.polygon.vertices.len()) as f64
                / 6.0)
                .log2()) as i64;

        let your_score = (max_score as f64 * point_ratio).ceil() as i64;
        let remaining_score = max_score - your_score;

        problems.push(ProblemState {
            problem_id,
            your_dislikes,
            minimal_dislikes,
            point_ratio,
            max_score,
            your_score,
            remaining_score,
        });
    }

    Ok(problems)
}

fn get_problems() -> Result<Vec<(i64, P)>> {
    let mut ret = vec![];
    for rd in fs::read_dir("./problems")? {
        let rd = rd?;

        let path = rd.path();
        if !matches!(path.extension(), Some(ext) if ext == "problem") {
            continue;
        }

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
fn list() -> Result<()> {
    let mut problems = get_problem_states()?;

    problems.sort_by_key(|r| Reverse(r.remaining_score));

    let mut table = prettytable::Table::new();

    table.add_row(row![
        "pid",
        "your",
        "best",
        "point ratio",
        "max score",
        "your score",
        "remaining",
    ]);

    for p in problems.iter() {
        table.add_row(row![
            p.problem_id,
            p.your_dislikes,
            p.minimal_dislikes,
            format!("{:.2}%", p.point_ratio * 100.0),
            p.max_score,
            p.your_score,
            p.remaining_score
        ]);
    }

    table.printstd();

    Ok(())
}

#[argopt::subcmd]
fn info(problem_id: i64) -> Result<()> {
    let problem = get_problem(problem_id)?;

    println!("Problem {}:", problem_id);
    println!("  * hole vertices:   {}", problem.hole.len());
    println!("  * figure vertices: {}", problem.figure.vertices.len());
    println!("  * figure edges:    {}", problem.figure.edges.len());
    println!(
        "  * epsilon:         {:.2}%",
        problem.epsilon as f64 / 1_000_000.0 * 100.0
    );
    println!("  * bonuses:");

    for bonus in problem.bonuses.iter() {
        println!("    * {:?}", bonus);
    }

    Ok(())
}

#[argopt::subcmd]
fn commands() -> Result<()> {
    let mut ps = get_problems()?;
    let status = get_problem_states()?;

    ps.sort_by_cached_key(|p| p.0);

    fn is_usable(bonus: BonusType) -> bool {
        bonus == BonusType::GLOBALIST || bonus == BonusType::SUPERFLEX
    }

    for &(pid, ref p) in ps.iter() {
        let stat = status
            .iter()
            .find(|r| r.problem_id == pid)
            .ok_or_else(|| anyhow!("Problem {}'s status not found", pid))?;

        let minimal_dislike = stat.minimal_dislikes;

        let mut use_bonus = vec![None];

        for pid in ps.iter().filter_map(|p| {
            p.1.bonuses
                .iter()
                .find(|bonus| is_usable(bonus.bonus) && bonus.problem as i64 == pid)
                .map(|_| p.0)
        }) {
            use_bonus.push(Some(pid));
        }

        for use_bonus in use_bonus {
            'outer: for b in 0..(1 << p.bonuses.len()) {
                let mut get_bonuses = vec![];

                for i in 0..p.bonuses.len() {
                    if (b >> i) & 1 != 0 {
                        if !is_usable(p.bonuses[i].bonus) {
                            continue 'outer;
                        }

                        get_bonuses.push(p.bonuses[i].problem);
                    }
                }

                println!(
                    "cargo run --release -- solve \
                        --time-limit=600 \
                        --threads=8 \
                        {}\
                        {}\
                        --penalty-ratio {} \
                        {}",
                    if let Some(b) = use_bonus {
                        format!("--bonus-from {} ", b)
                    } else {
                        "".to_string()
                    },
                    if !get_bonuses.is_empty() {
                        format!(
                            "--get-bonuses {} ",
                            get_bonuses
                                .iter()
                                .map(|r| r.to_string())
                                .collect_vec()
                                .join(" ")
                        )
                    } else {
                        "".to_string()
                    },
                    max(10, minimal_dislike / 2),
                    pid
                );
            }
        }
    }

    Ok(())
}

#[argopt::cmd_group(commands = [solve, max_scores, submit, login, list, info, commands])]
fn main() -> Result<()> {}
