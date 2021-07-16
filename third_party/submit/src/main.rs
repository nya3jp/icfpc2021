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

use anyhow::Result;
use log::debug;
use reqwest::blocking::{multipart, Client};
use std::fs::File;
use once_cell::sync::Lazy;
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::{clap, StructOpt};
use tempfile::NamedTempFile;

const BASE_SUBMISSION_URL: &str = "https://poses.live/api/problems";
const DASHBOARD_ENDPOINT: &str = "http://spweek.badalloc.com/api/solutions";

pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[derive(StructOpt, Debug)]
#[structopt(name = "submit")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(short = "p", long = "problem_id")]
    pub problem_id: u32,

    #[structopt(short = "s", long = "solution")]
    pub solution_file: Option<PathBuf>,

    #[structopt(short, long)]
    pub dryrun: bool,
}

fn get_submission_url(problem_id: u32) -> String {
    format!("{}/{}/solutions", BASE_SUBMISSION_URL, problem_id)
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Opt::from_args();

    let api_key = std::env::var("API_KEY")?;

    let problem_id = args.problem_id;
    let submission_url = get_submission_url(problem_id);

    let mut solution = String::new();
    match args.solution_file {
        Some(path) => {
            let mut file = File::open(path)?;
            file.read_to_string(&mut solution)?;
        }
        None => {
            let stdin = std::io::stdin();
            let mut handle = stdin.lock();
            handle.read_to_string(&mut solution)?;
        }
    }

    debug!("api key = {}", api_key);
    debug!("problem id = {}", problem_id);
    debug!("solution = {}", solution);

    let mut f = NamedTempFile::new()?;
    f.write_all(solution.as_bytes())?;
    let form = multipart::Form::new()
        .text("problem_id", problem_id.to_string())
        .file("solution", f.path())?;
    let dashboard_submission = CLIENT
        .post(DASHBOARD_ENDPOINT)
        .multipart(form)
        .send()?;

    if args.dryrun {
        return Ok(());
    }

    let submission = CLIENT
        .post(submission_url)
        .bearer_auth(api_key)
        .body(solution)
        .send()?;
    debug!("Status: {}", submission.status());
    let body = submission.text()?;
    debug!("Body: {}", body);

    Ok(())
}
