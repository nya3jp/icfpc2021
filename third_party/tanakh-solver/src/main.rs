#[macro_use]
extern crate prettytable;

mod sa;

use std::cmp::{max, min, Reverse};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use chrono::{Datelike, Timelike};
use easy_scraper::Pattern;
use geom::{
    point::Point,
    polygon::ContainsResult,
    schema::{Pose, Problem as P},
};
use itertools::Itertools;
use rand::Rng;
use reqwest::blocking::ClientBuilder;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::header::HeaderValue;
use sa::*;
use scorer::{is_inside_hole, is_valid_solution};
use tanakh_solver::{get_problem, ENDPOINT};

#[derive(Clone)]
struct Problem {
    problem: P,
    exact: bool,
}

impl Annealer for Problem {
    type State = Pose;

    type Move = (usize, Point);

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

            for p in self.problem.hole.polygon.vertices.iter() {
                minx = min(minx, p.x as i64);
                maxx = max(maxx, p.x as i64);
                miny = min(miny, p.y as i64);
                maxy = max(maxy, p.y as i64);
            }

            let ret = (0..self.problem.figure.vertices.len())
                .map(|_| loop {
                    let x = rng.gen_range(minx..=maxx);
                    let y = rng.gen_range(miny..=maxy);

                    if self
                        .problem
                        .hole
                        .polygon
                        .contains(&Point::new(x as _, y as _))
                        != ContainsResult::OUT
                    {
                        break Point::new(x as _, y as _);
                    }
                })
                .collect_vec();

            let ret = Pose { vertices: ret };

            if is_inside_hole(&self.problem, &ret) {
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
            pena += (err / eps).powf(1.0);
        }

        for h in self.problem.hole.polygon.vertices.iter() {
            score += state
                .vertices
                .iter()
                .map(|v| (*v - *h).norm_sqr())
                .fold(0.0 / 0.0, f64::min);
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
            let i = rng.gen_range(0..state.vertices.len());

            if !self.exact {
                let dx = rng.gen_range(-w..=w);
                let dy = rng.gen_range(-w..=w);
                if (dx, dy) == (0, 0) {
                    continue;
                }

                state.vertices[i].x += dx as f64;
                state.vertices[i].y += dy as f64;

                let ok = is_inside_hole(&self.problem, &state);

                state.vertices[i].x -= dx as f64;
                state.vertices[i].y -= dy as f64;

                if !ok {
                    continue;
                }

                break (i, Point::new(dx as _, dy as _));
            } else {
                let j = rng.gen_range(0..self.problem.hole.polygon.vertices.len());
                if state.vertices[i] == self.problem.hole.polygon.vertices[j] {
                    continue;
                }

                let t = state.vertices[i];
                state.vertices[i] = self.problem.hole.polygon.vertices[j];
                let ok = is_inside_hole(&self.problem, &state);
                state.vertices[i] = t;

                if !ok {
                    continue;
                }

                break (i, self.problem.hole.polygon.vertices[j] - state.vertices[i]);
            }
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        state.vertices[mov.0] += mov.1;
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        state.vertices[mov.0] -= mov.1;
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

    #[opt(long)] no_submit: bool,

    problem_id: i64,
) -> Result<()> {
    let seed = rand::thread_rng().gen();

    let problem: P = get_problem(problem_id)?;
    let problem = Problem { problem, exact };

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

    if score.is_infinite() || (score.round() - score).abs() > 1e-10 {
        eprintln!("Cannot find solution");
        eprintln!(
            "Wrong solution: score = {}, {}",
            score,
            serde_json::to_string(&solution)?
        );
        return Ok(());
    }

    if !is_valid_solution(&problem.problem, &solution) {
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

    if !no_submit
        && dialoguer::Confirm::new()
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
        return Ok(jar);
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

#[argopt::subcmd]
fn list() -> Result<()> {
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

    struct ProblemState {
        problem_id: i64,
        your_dislikes: i64,
        minimal_dislikes: i64,
        point_ratio: f64,
        max_score: i64,
        your_score: i64,
        remaining_score: i64,
    }

    let mut problems = vec![];

    for m in pat.matches(&resp) {
        let problem_id: i64 = m["problem-id"].parse()?;
        let your_dislikes: i64 = m["your-dislikes"].parse()?;
        let minimal_dislikes: i64 = m["minimal-dislikes"].parse()?;

        let point_ratio = (((minimal_dislikes + 1) as f64) / ((your_dislikes + 1) as f64)).sqrt();

        let problem = get_problem(problem_id)?;
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

        // println!(
        //     "{}: {}, {}, {}, {}, {}, {}",
        //     problem_id,
        //     your_dislikes,
        //     minimal_dislikes,
        //     max_score,
        //     your_score,
        //     point_ratio,
        //     remaining_score,
        // );
    }

    problems.sort_by_key(|r| Reverse(r.remaining_score));

    let mut table = prettytable::Table::new();

    table.add_row(row![
        "pid",
        "your",
        "best",
        "max score",
        "point ratio",
        "your score",
        "remaining",
    ]);

    for p in problems.iter() {
        // println!(
        //     "{:3}: your = {:5}, best = {:5}, score = {:5}, ratio: {:05.2}, your score = {:5}, remaining = {:5}",
        //     p.problem_id,
        //     p.your_dislikes,
        //     p.minimal_dislikes,
        //     p.max_score,
        //     p.point_ratio,
        //     p.your_score,
        //     p.remaining_score,
        // );

        table.add_row(row![
            p.problem_id,
            p.your_dislikes,
            p.minimal_dislikes,
            p.max_score,
            format!("{:.2}%", p.point_ratio * 100.0),
            p.your_score,
            p.remaining_score
        ]);
    }

    table.printstd();

    Ok(())
}

#[argopt::cmd_group(commands = [solve, max_scores, submit, login, list])]
fn main() -> Result<()> {}
