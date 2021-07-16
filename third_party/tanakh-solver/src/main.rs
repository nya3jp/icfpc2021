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

#[macro_use]
extern crate prettytable;

mod sa;
mod solve;

use anyhow::{anyhow, Result};
use geom::schema::BonusType;
use itertools::Itertools;
use reqwest::blocking::ClientBuilder;
use reqwest::cookie::{CookieStore, Jar};
use tanakh_solver::{get_problem, get_problem_states, get_problems, ENDPOINT};

use std::{
    cmp::{max, Reverse},
    fs::File,
    io::Write,
    path::PathBuf,
    sync::Arc,
};

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

#[argopt::cmd_group(commands = [solve::solve, max_scores, submit, login, list, info, commands])]
fn main() -> Result<()> {}
