use anyhow::Result;
use log::debug;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::{clap, StructOpt};

const BASE_SUBMISSION_URL: &str = "https://poses.live/api/problems";

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
    dryrun: bool,
}

fn get_submission_url(problem_id: u32) -> String {
    format!("{}/{}/solutions", BASE_SUBMISSION_URL, problem_id)
}

#[tokio::main]
async fn main() -> Result<()> {
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

    let client = reqwest::Client::new()
            .post(submission_url)
            .bearer_auth(api_key)
            .body(solution);
    
    if args.dryrun {
        return Ok(());
    }
    let res = client.send().await?;
    debug!("Status: {}", res.status());
    let body = res.text().await?;
    debug!("Body: {}", body);

    Ok(())
}
