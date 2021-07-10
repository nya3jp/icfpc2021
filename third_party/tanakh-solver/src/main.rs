#[macro_use]
extern crate prettytable;

mod sa;

use std::cmp::{max, Reverse};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Result};
use chrono::{Datelike, Timelike};
use easy_scraper::Pattern;
use geom::{
    point::Point,
    // polygon::ContainsResult,
    schema::{Edge, Pose, Problem as P},
};
use itertools::Itertools;
use rand::Rng;
use reqwest::blocking::ClientBuilder;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::header::HeaderValue;
use sa::*;
use scorer::{is_inside_hole, is_inside_hole_partial, is_valid_solution};
use tanakh_solver::{get_problem, ENDPOINT};

fn read_hint<P: AsRef<Path>>(path: P) -> Result<BTreeMap<usize, usize>> {
    let v: Vec<(usize, usize)> = serde_json::from_reader(File::open(path)?)?;
    let mut m = BTreeMap::new();
    for (i1, i2) in &v {
        m.insert(*i1, *i2);
    }
    Ok(m)
}

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
    penalty_ratio: f64,
    exact: bool,
    triangles: Vec<(usize, usize, usize)>,
    init_state: Option<Pose>,
    start_temp: Option<f64>,
    fixed_points: BTreeMap<usize, Point>, // From |node| to |point|.
    candidate_vertices: Vec<usize>,
    candidate_edges: Vec<usize>,
    candidate_triangles: Vec<usize>,
}

impl Annealer for Problem {
    type State = Pose;

    type Move = Vec<(usize, Point)>;

    fn init_state(&self, rng: &mut impl rand::Rng) -> Self::State {
        if let Some(init_state) = &self.init_state {
            if !is_inside_hole(&self.problem, init_state) {
                panic!("init state is not inside the hole");
            }

            return init_state.clone();
        }

        let ix = rng.gen_range(0..self.problem.hole.len());

        let default_point = self.problem.hole[ix].clone();

        let ret = (0..self.problem.figure.vertices.len())
            .map(|i| {
                *self.fixed_points.get(&i).unwrap_or(&default_point.clone())
                /*self.hole[rng.gen_range(0..self.hole.len())].clone()*/
            })
            .collect_vec();

        let init_state = Pose { vertices: ret };
        if !is_inside_hole(&self.problem, &init_state) {
            eprintln!("Wrong Answer!!");
        }
        init_state

        // loop {
        //     let mut minx = i64::MAX;
        //     let mut maxx = i64::MIN;
        //     let mut miny = i64::MAX;
        //     let mut maxy = i64::MIN;

        //     for p in self.problem.hole.iter() {
        //         minx = min(minx, p.x as i64);
        //         maxx = max(maxx, p.x as i64);
        //         miny = min(miny, p.y as i64);
        //         maxy = max(maxy, p.y as i64);
        //     }

        //     let ret = (0..self.problem.figure.vertices.len())
        //         .map(|_| loop {
        //             let x = rng.gen_range(minx..=maxx);
        //             let y = rng.gen_range(miny..=maxy);

        //             if self
        //                 .problem
        //                 .contains(&Point::new(x as _, y as _))
        //                 != ContainsResult::OUT
        //             {
        //                 break Point::new(x as _, y as _);
        //             }
        //         })
        //         .collect_vec();

        //     let ret = Pose { vertices: ret };

        //     if is_inside_hole(&self.problem, &ret) {
        //         break ret;
        //     }
        // }
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        self.start_temp
            .unwrap_or_else(|| (init_score / 100.0).max(self.penalty_ratio))
    }

    fn is_done(&self, score: f64) -> bool {
        score < 1e-10
    }

    fn eval(&self, state: &Self::State) -> f64 {
        let mut score = 0.0;
        let mut pena = 0.0;

        let eps = self.problem.epsilon as f64 / 1_000_000.0;

        for edge in self.problem.figure.edges.iter() {
            let i = edge.v1;
            let j = edge.v2;

            let d1 = (state.vertices[i] - state.vertices[j]).norm_sqr();
            let d2 = (self.problem.figure.vertices[i] - self.problem.figure.vertices[j]).norm_sqr();
            let err = ((d1 as f64 / d2 as f64) - 1.0).abs();

            if err <= eps {
                continue;
            }

            // score += 500.0 * (err / eps);
            // score += 1000.0 * (err / eps).powi(2);
            pena += err / eps;
        }

        for h in self.problem.hole.iter() {
            score += state
                .vertices
                .iter()
                .map(|v| (*v - *h).norm_sqr())
                .fold(0.0 / 0.0, f64::min);
        }

        score * (1.0 + pena / 10.0) + pena * self.penalty_ratio
        // score
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl rand::Rng,
        progress_ratio: f64,
    ) -> Self::Move {
        let w = max(1, (4.0 * (1.0 - progress_ratio)).round() as i64);

        loop {
            match rng.gen_range(0..if self.exact { 21 } else { 20 }) {
                0..=9 => {
                    let i = rng.gen_range(0..self.candidate_vertices.len());
                    let i = self.candidate_vertices[i];

                    let dx = rng.gen_range(-w..=w);
                    let dy = rng.gen_range(-w..=w);
                    if (dx, dy) == (0, 0) {
                        continue;
                    }

                    let d = Point::new(dx as _, dy as _);

                    state.vertices[i] += d;

                    let ok = is_inside_hole_partial(&self.problem, &state, &[i]);

                    state.vertices[i] -= d;

                    if ok {
                        return vec![(i, d)];
                    }
                }
                10..=16 => loop {
                    let i = rng.gen_range(0..self.candidate_edges.len());
                    let e = &self.problem.figure.edges[self.candidate_edges[i]];
                    let i = e.v1;
                    let j = e.v2;

                    // let i = rng.gen_range(0..state.vertices.len());
                    // let j = rng.gen_range(0..state.vertices.len());
                    // if !self.problem.figure.edges.contains(&Edge::new(i, j)) {
                    //     continue;
                    // }

                    let dx = rng.gen_range(-w..=w);
                    let dy = rng.gen_range(-w..=w);
                    if (dx, dy) == (0, 0) {
                        continue;
                    }

                    let d1 = Point::new(dx as _, dy as _);
                    let d2 = d1;

                    state.vertices[i] += d1;
                    state.vertices[j] += d2;

                    let ok = is_inside_hole_partial(&self.problem, &state, &[i, j]);

                    state.vertices[i] -= d1;
                    state.vertices[j] -= d2;

                    if ok {
                        return vec![(i, d1), (j, d2)];
                    }
                },
                17..=19 => {
                    if self.candidate_triangles.is_empty() {
                        continue;
                    }
                    let i = rng.gen_range(0..self.candidate_triangles.len());
                    let (i, j, k) = self.triangles[self.candidate_triangles[i]];

                    let dx = rng.gen_range(-w..=w);
                    let dy = rng.gen_range(-w..=w);
                    if (dx, dy) == (0, 0) {
                        continue;
                    }

                    let d1 = Point::new(dx as _, dy as _);
                    let d2 = d1;
                    let d3 = d1;

                    state.vertices[i] += d1;
                    state.vertices[j] += d2;
                    state.vertices[k] += d3;

                    let ok = is_inside_hole_partial(&self.problem, &state, &[i, j, k]);

                    state.vertices[i] -= d1;
                    state.vertices[j] -= d2;
                    state.vertices[k] -= d3;

                    if ok {
                        return vec![(i, d1), (j, d2), (k, d3)];
                    }
                }

                _ => {
                    for _ in 0..10 {
                        let i = rng.gen_range(0..self.candidate_vertices.len());
                        let i = self.candidate_vertices[i];

                        let j = rng.gen_range(0..self.problem.hole.polygon.vertices.len());
                        if state.vertices[i] == self.problem.hole.polygon.vertices[j] {
                            continue;
                        }

                        let t = state.vertices[i];
                        state.vertices[i] = self.problem.hole.polygon.vertices[j];
                        let ok = is_inside_hole_partial(&self.problem, &state, &[i]);
                        state.vertices[i] = t;

                        if ok {
                            return vec![(
                                i,
                                self.problem.hole.polygon.vertices[j] - state.vertices[i],
                            )];
                        }
                    }
                }
            }
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        for (i, v) in mov.iter() {
            state.vertices[*i] += *v;
        }
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        for (i, v) in mov.iter() {
            state.vertices[*i] -= *v;
        }
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

    /// seed
    //
    #[opt(long)]
    seed: Option<u64>,

    /// search around optimal solution
    //
    #[opt(long)]
    exact: bool,

    // Find the hole->node mapping at the beginning.
    //
    #[opt(long)] use_hint: bool,

    /// Use specified initial state
    #[opt(long)]
    init_state: Option<PathBuf>,

    /// Use specified initial state
    #[opt(long)]
    start_temp: Option<f64>,

    #[opt(long, default_value = "100.0")] penalty_ratio: f64,
    #[opt(long, default_value = "1.0")] min_temp: f64,

    #[opt(long)] no_submit: bool,

    problem_id: i64,
) -> Result<()> {
    let seed = seed.unwrap_or_else(|| rand::thread_rng().gen());

    let problem: P = get_problem(problem_id)?;

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
            exact,
            penalty_ratio,
            triangles: triangles.clone(),
            fixed_points: hint.clone(),
            init_state: init_state.clone(),
            start_temp: start_temp.clone(),
            candidate_vertices: filter_vertices(&problem.figure.vertices, &hint),
            candidate_edges: filter_edges(&problem.figure.edges, &hint),
            candidate_triangles: filter_triangles(&triangles, &hint),
        };

        let (score, solution) = annealing(
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

        if score.is_infinite() || (score.round() - score).abs() > 1e-10 {
            eprintln!("Cannot find solution");
            eprintln!(
                "Wrong solution: score = {}, {}",
                score,
                serde_json::to_string(&solution)?
            );
            continue;
        }

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

    let score = min_score.unwrap();
    let solution = min_solution.unwrap();

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

    if no_submit {
        return Ok(());
    }

    let problems = get_problem_states()?;
    let problem = problems.iter().find(|r| r.problem_id == problem_id);

    if let Some(problem) = problem {
        eprintln!(
            "Dislike: {}, Your previous dislike: {}, Minimal dislike: {}",
            score as i64, problem.your_dislikes, problem.minimal_dislikes
        );
    } else {
        eprintln!("No submission for problem {} found.", problem_id);
    }

    if dialoguer::Confirm::new()
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
        // eprintln!("Session file not found. start new session.");
        // return Ok(jar);
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

    let mut problems = vec![];

    for m in pat.matches(&resp) {
        let problem_id: i64 = m["problem-id"].parse()?;
        let your_dislikes = m["your-dislikes"].parse();

        if your_dislikes.is_err() {
            continue;
        }

        let your_dislikes: i64 = your_dislikes.unwrap();

        let minimal_dislikes: i64 = m["minimal-dislikes"].parse()?;

        let point_ratio = (((minimal_dislikes + 1) as f64) / ((your_dislikes + 1) as f64)).sqrt();

        let problem: P =
            serde_json::from_reader(File::open(format!("../problems/{}.problem", problem_id))?)?;

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

    Ok(())
}

#[argopt::cmd_group(commands = [solve, max_scores, submit, login, list, info])]
fn main() -> Result<()> {}
