use anyhow::Result;
use geom::schema::{Pose, Problem};
use once_cell::sync::Lazy;
use reqwest::blocking::{Client, multipart};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::Deref;

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
