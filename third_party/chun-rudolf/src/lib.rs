use anyhow::Result;
use geom::schema::{Pose, Hole, Figure, Bonus};
use once_cell::sync::Lazy;
use reqwest::blocking::{Client, multipart};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::Deref;

// pub mod geom;

pub const DASHBOARD_ENDPOINT: &str = "http://spweek.badalloc.com";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProblemDashboardData {
    pub hole: Hole,
    pub figure: Figure,
    pub epsilon: i64,
    pub bonuses: Vec<Bonus>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProblemDashboard {
    pub data: ProblemDashboardData,
    pub minimal_dislike: usize
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SolutionDashboard {
    pub solution_id: usize,
    pub problem_id: usize,
    pub created_at: usize,
    pub dislike: usize,
    pub reject_reason: String,
    pub data: Pose
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResult {
    pub id: String,
}

pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

pub fn http_get_dashboard(api: impl AsRef<str>) -> Result<String> {
    Ok(CLIENT
        .get(format!("{}{}", DASHBOARD_ENDPOINT, api.as_ref()))
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
    Ok(serde_json::from_str(&http_get_dashboard("/api/hello")?)?)
}

pub fn get_problem(problem_id: i64) -> Result<ProblemDashboard> {
    Ok(serde_json::from_str(&http_get_dashboard(format!(
        "/api/problems/{}",
        problem_id
    ))?)?)
}

pub fn get_solutions(problem_id: i64) -> Result<Vec<SolutionDashboard>> {
    Ok(serde_json::from_str(&http_get_dashboard(format!(
        "/api/problems/{}/solutions",
        problem_id
    ))?)?)
}

pub fn submit_dashboard(problem_id: i64, solution_file_name: &str) -> Result<()> {
    post_solution_dashboard(problem_id, solution_file_name)?;
    Ok(())
}
