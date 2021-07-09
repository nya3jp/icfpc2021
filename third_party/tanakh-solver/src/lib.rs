use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::Deref;

const ENDPOINT: &str = "https://poses.live";

static API_TOKEN: Lazy<String> =
    Lazy::new(|| std::env::var("API_TOKEN").expect("environment variable API_TOKEN must be set"));

#[derive(Serialize, Deserialize, Debug)]
pub struct Problem {
    pub hole: Vec<(i64, i64)>,
    pub figure: Figure,
    pub epsilon: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Figure {
    pub edges: Vec<(i64, i64)>,
    pub vertices: Vec<(i64, i64)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Solution {
    pub vertices: Vec<(i64, i64)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResult {
    pub id: String,
}

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

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

pub fn hello() -> Result<Value> {
    Ok(serde_json::from_str(&http_get("/api/hello")?)?)
}

pub fn get_problem(problem_id: i64) -> Result<Problem> {
    Ok(serde_json::from_str(&http_get(format!(
        "/api/problems/{}",
        problem_id
    ))?)?)
}

pub fn submit(problem_id: i64, solution: &Solution) -> Result<SubmitResult> {
    Ok(serde_json::from_str(&post_json(
        format!("{}/api/problems/{}/solutions", ENDPOINT, problem_id),
        solution,
    )?)?)
}
