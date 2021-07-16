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

use geom::schema;
use scorer;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let problem = schema::parse_problem(&args[1])?;
    let pose = schema::parse_pose(&args[2])?;
    let score = scorer::dislike(&problem.hole, &pose);
    let is_valid = scorer::is_valid_solution(&problem, &pose);
    let bonus = scorer::bonus(&problem, &pose);
    if args.len() == 4 && args[3] == "json" {
        println!("{{\"dislike\": {:?}, \"is_valid\": {:?}, \"bonus\": {}}}",
                 score, is_valid, serde_json::to_string(&bonus)?);
    } else {
        eprintln!("Problem {:?}", problem);
        eprintln!("Solution {:?}", pose);
        eprintln!("dislike = {:?}", score);
        eprintln!("is_valid = {:?}", is_valid);
        eprintln!("bonus = {}", serde_json::to_string(&bonus)?);
    }
    Ok(())
}
