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
use chun_oikomi_solver::{get_problem, ENDPOINT};
use std::{thread, time::SystemTime};

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
    init_state: Pose,
}

#[derive(Clone)]
struct OikomiOptions {
    pub time_limit: f64,
    pub neighbor: i64,
    pub header: String,
    pub silent: bool
}

fn visit_order_dfs(ptr: usize, node: usize, edges: &Vec<Edge>, visited: &mut Vec<bool>, resorder: &mut Vec<usize>, n: usize) 
{
    visited[node] = true;
    resorder[ptr] = node;
    if ptr == n - 1 {
        return
    }
    let mut conn = vec![0usize; n];
    for Edge {v1, v2} in edges {
        for p in 0..=ptr {
            let (v1, v2) = if *v2 == resorder[p] { (v2, v1) } else { (v1, v2) };
            if *v1 != resorder[p] {
                continue;
            }
            if visited[*v2] {
                continue;
            }
            conn[*v2] += 1
        }
    }
    let mut conn_ix:Vec<(usize, usize)> = conn.into_iter().enumerate().map(|(idx, v)| (v, idx)).collect();
    conn_ix.sort_unstable_by(|a, b| b.cmp(a));
    let bestnode = conn_ix[0].1;
    visit_order_dfs(ptr + 1, bestnode, edges, visited, resorder, n);
}

fn get_visit_order(prob: &Problem) -> Vec<usize> {
    let n = prob.problem.figure.vertices.len();
    let mut degrees = vec![0i64; n];
    for Edge {v1, v2} in prob.problem.figure.edges.iter() {
        degrees[*v1] += 1;
        degrees[*v2] += 1;
    }
    let mut deg_and_ix: Vec<(i64, usize)> = degrees.into_iter().enumerate().map(|(idx, v)| (v, idx)).collect();
    deg_and_ix.sort_unstable_by(|a, b| b.cmp(a));
    let firstnode = deg_and_ix[0].1;

    let mut visited = vec![false; n];
    let mut resorder = vec![0usize; n];
    visit_order_dfs(0usize, firstnode, &prob.problem.figure.edges, &mut visited, &mut resorder, n);
    resorder
}


fn eval_score(prob: &Problem, figure: &Vec<Point>) -> f64 
{
    let mut score = 0.;
    for h in prob.problem.hole.iter() {
        score += figure.iter().map(|v| (*v - *h).norm_sqr()).fold(0./ 0., f64::min)
    }
    score
}

fn do_brute(ptr: usize, visit_order: &Vec<usize>, budget: i64, prob: &Problem, oikomi_options: &OikomiOptions, timer: &SystemTime,bestscore: &mut f64, resfigure: &mut Vec<Point>, bestfigure: &mut Vec<Point>)
{
    if *bestscore == 0. {
        return;
    }
    if timer.elapsed().unwrap().as_secs_f64() > oikomi_options.time_limit {
        return;
    }
    // println!("depth {}/{}", ptr, resfigure.len());
    let trypos = visit_order[ptr];
    let origpos = prob.init_state.vertices[trypos];
    for dx in -budget..=budget {
        let budgetremain = budget - dx.abs();
        for dy in -budgetremain..=budgetremain {
            let budgetremain = budgetremain - dy.abs();
            let dxdy = Point {x: dx as f64, y: dy as f64};
            let trialpt = origpos + dxdy;
            if !prob.problem.hole.contains(&trialpt) { continue; }
            resfigure[trypos] = trialpt;

            let mut isok = true;
            for Edge {v1, v2} in prob.problem.figure.edges.iter() {
                let (v1, v2) = if *v2 == trypos { (*v2, *v1) } else { (*v1, *v2) };
                if v1 != trypos {
                    continue;
                }
                for j in 0..ptr {
                    if v2 != visit_order[j] {
                        continue; 
                    }
                    let p1 = prob.problem.figure.vertices[v1];
                    let p2 = prob.problem.figure.vertices[v2];
                    let d1 = (p1.x - p2.x) * (p1.x - p2.x) + (p1.y - p2.y) * (p1.y - p2.y);
                    let q1 = resfigure[v1];
                    let q2 = resfigure[v2];
                    let d2 = ((q1.x - q2.x) * (q1.x - q2.x) + (q1.y - q2.y) * (q1.y - q2.y)) as f64;    
                    // if d1 < d2
                    //   | d2/d1 - 1 | = d2/d1 - 1
                    //   <=> check d2 * 1000000 - d1 * 1000000 <= eps * d1
                    // else
                    //   | d2/d1 - 1 | = 1 - d2/d1
                    //   <=>check d1 * 1000000 - d2 * 1000000 <= eps * d1
                    let lhs = if d1 < d2 {
                        d2 * 1000000.0 - d1 * 1000000.0
                    } else {
                        d1 * 1000000.0 - d2 * 1000000.0
                    };
                    let rhs = prob.problem.epsilon as f64 * d1;
                    if lhs > rhs {
                        isok = false;
                        //println!("invalid edge {} {}", v1, v2);
                        break;
                    }
                    // collision check (not strict version)
                    for i in 0..prob.problem.hole.len() {
                        let h1 = &prob.problem.hole[i];
                        let h2 = &prob.problem.hole[(i + 1) % prob.problem.hole.len()];
                        if geom::is_crossing(&q1, &q2, h1, h2) {
                            isok = false;
                            break;
                        }
                    }
                    if !isok {
                        break;
                    }
                }
                if !isok {
                    break;
                }
            }
            if isok {
                if ptr == resfigure.len() - 1 {
                    /*
                    if score < 2442. {
                        println!("score {}, valid {}, figure {:?}", score, is_valid_solution(prob, resfigure), resfigure);
                    }
                    */
                    if ! is_valid_solution(&prob.problem, &Pose {vertices: resfigure.to_vec(), bonuses: None}) { continue; }
                    let score = eval_score(prob, resfigure);
                    //println!("Score {}", score);
                    if score < *bestscore {
                        println!("Updated best {} -> {}", bestscore, score);
                        *bestscore = score;
                        *bestfigure = resfigure.clone();
                    }
                }
                else
                {
                    do_brute(ptr + 1, visit_order, budgetremain, prob, oikomi_options, timer, bestscore, resfigure, bestfigure);
                }
            }
        }
    }
}

fn oikomi(prob: &Problem, oikomi_options: &OikomiOptions) -> (f64, Pose) {
    let mut timer = SystemTime::now();
    let visit_order = get_visit_order(prob);
    println!("visit_order = {:?}", visit_order);
    let n = prob.problem.figure.vertices.len();
    let budget = oikomi_options.neighbor;
    let mut bestscore = eval_score(prob, &prob.init_state.vertices);
    let mut resfigure = prob.init_state.vertices.clone();
    let mut bestfigure = resfigure.clone();
    do_brute(0usize, &visit_order, budget, &prob, &oikomi_options, &timer, &mut bestscore, &mut resfigure, &mut bestfigure);
    (bestscore, Pose {vertices: bestfigure, bonuses: None})
}



#[argopt::subcmd]
fn solve(
    /// time limit in seconds
    //
    #[opt(long, default_value = "5.0")]
    time_limit: f64,

    /// Use specified initial state
    #[opt(long)]
    init_state: Option<PathBuf>,

    /// Neighbor distance to search
    #[opt(long)]
    neighbor: i64,

    #[opt(long)] no_submit: bool,

    problem_id: i64,
) -> Result<()> {
    let problem: P = get_problem(problem_id)?;

    /*
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
    */

    eprintln!("Oikomi: searching up to {} pixel difference solutions from the current one", neighbor);

    let init_state: Pose = match init_state {
        Some (frompath) =>
            serde_json::from_reader(
                File::open(&frompath).expect(&format!("{} is not found", frompath.display())),
            )
            .expect("invalid json file"),
        None =>
            get_problem_latest_solution(problem_id).unwrap()
    };
    let mut min_score = None;
    let mut min_solution = None;
    let problem = Problem {
        problem: problem.clone(),
        init_state: init_state.clone(),
    };

    let (score, solution) = oikomi(
        &problem,
        &OikomiOptions {
            time_limit,
            silent: false,
            neighbor,
            header: format!("Problem {}: ", problem_id),
        }
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
    }

    if min_score.is_none() || min_score.unwrap() > score {
        min_score = Some(score);
        min_solution = Some(solution);
    }

    let score = min_score.unwrap();
    let solution = min_solution.unwrap();

    eprintln!("Score for problem {}: {}", problem_id, score);

    println!("{}", serde_json::to_string(&solution)?);

    if !Path::new("results").exists() {
        fs::create_dir_all("results")?;
    }

    let now = chrono::Local::now();
    let result_file_name = format!(
        "results/{}-{}-{:02}{:02}{:02}{:02}.json",
        problem_id,
        score.round() as i64,
        now.date().day(),
        now.time().hour(),
        now.time().minute(),
        now.time().second(),
    );
    fs::write(&result_file_name, serde_json::to_string(&solution)?)?;

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
        if (score as i64) < problem.your_dislikes {
            if dialoguer::Confirm::new()
                .with_prompt("Submit?")
                .interact()?
            {
                eprintln!("Submitting");
    
                let resp = chun_oikomi_solver::submit(problem_id, &solution)?;
                eprintln!("Response: {:?}", resp);
    
                // Submit to the internal dashboard.
                chun_oikomi_solver::submit_dashboard(problem_id, &result_file_name)?;
            }
        }else{
            println!("Shinchoku damedesu (no point improvement)");
        }
    } else {
        eprintln!("No submission for problem {} found.", problem_id);
    }

    Ok(())
}

#[argopt::subcmd]
fn submit(problem_id: i64, json_file: PathBuf) -> Result<()> {
    let solution = serde_json::from_reader(File::open(json_file)?)?;
    let resp = chun_oikomi_solver::submit(problem_id, &solution)?;
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

fn get_problem_latest_solution(problemid: i64) -> Result<Pose> {
    let solutions = chun_oikomi_solver::get_solutions(problemid).unwrap();
    let (ret, retsc) = 
        solutions.iter().fold((None, 1e8), |(sol, sc), x| {
            if x.dislike < sc {
                (Some(x.data.clone()), x.dislike)
            }
            else
            {
                (sol, sc)
            }
        });
    Ok(ret.unwrap())
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

        let your_dislikes = your_dislikes.unwrap_or(9999999);

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
