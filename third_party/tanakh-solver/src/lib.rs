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

use anyhow::{anyhow, bail, Result};
use easy_scraper::Pattern;
use geom::schema::{Pose, Problem};
use itertools::Itertools;
use once_cell::sync::Lazy;
use reqwest::{
    blocking::{multipart, Client, ClientBuilder},
    cookie::{CookieStore, Jar},
    header::HeaderValue,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
    {fs, ops::Deref},
};

// pub mod geom;

pub const ENDPOINT: &str = "https://poses.live";
pub const DASHBOARD_ENDPOINT: &str = "http://spweek.badalloc.com";

static API_TOKEN: Lazy<String> =
    Lazy::new(|| std::env::var("API_TOKEN").expect("environment variable API_TOKEN must be set"));

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResult {
    pub id: String,
}

pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

pub fn http_get(api: impl AsRef<str>) -> Result<String> {
    Ok(CLIENT
        .get(format!("{}{}", ENDPOINT, api.as_ref()))
        .bearer_auth(API_TOKEN.deref())
        .send()?
        .error_for_status()?
        .text()?)
}

pub fn post_json(api: impl AsRef<str>, json: impl Serialize) -> Result<String> {
    Ok(CLIENT
        .post(api.as_ref())
        .bearer_auth(API_TOKEN.deref())
        .json(&json)
        .send()?
        .error_for_status()?
        .text()?)
}

pub fn post_solution_dashboard(problem_id: i64, file: &str) -> Result<String> {
    let form = multipart::Form::new()
        .text("problem_id", problem_id.to_string())
        .file("solution", file)?;

    Ok(CLIENT
        .post(format!("{}/api/solutions", DASHBOARD_ENDPOINT))
        .multipart(form)
        .send()?
        .error_for_status()?
        .text()?)
}

pub fn hello() -> Result<Value> {
    Ok(serde_json::from_str(&http_get("/api/hello")?)?)
}

pub fn get_problem(problem_id: i64) -> Result<Problem> {
    Ok(serde_json::from_str(&http_get(format!(
        "/api/problems/{}",
        problem_id
    ))?)?)
}

pub fn submit(problem_id: i64, solution: &Pose) -> Result<SubmitResult> {
    Ok(serde_json::from_str(&post_json(
        format!("{}/api/problems/{}/solutions", ENDPOINT, problem_id),
        solution,
    )?)?)
}

pub fn submit_dashboard(problem_id: i64, solution_file_name: &str) -> Result<()> {
    post_solution_dashboard(problem_id, solution_file_name)?;
    Ok(())
}

pub fn get_problems() -> Result<Vec<(i64, Problem)>> {
    let mut ret = vec![];
    for rd in fs::read_dir("./problems")? {
        let rd = rd?;

        let path = rd.path();
        if !matches!(path.extension(), Some(ext) if ext == "problem") {
            continue;
        }

        let problem = serde_json::from_reader(fs::File::open(&path)?)?;
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

pub struct ProblemState {
    pub problem_id: i64,
    pub your_dislikes: i64,
    pub minimal_dislikes: i64,
    pub point_ratio: f64,
    pub max_score: i64,
    pub your_score: i64,
    pub remaining_score: i64,
}

pub fn get_problem_states() -> Result<Vec<ProblemState>> {
    let cookie_store = Arc::new(load_cookie_store("session.txt", ENDPOINT)?);

    let client = ClientBuilder::new()
        .cookie_provider(Arc::clone(&cookie_store))
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

fn load_cookie_store(session_file: impl AsRef<Path>, endpoint: &str) -> Result<Jar> {
    let url = endpoint.parse().unwrap();
    let jar = reqwest::cookie::Jar::default();
    let f = fs::File::open(session_file);

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
