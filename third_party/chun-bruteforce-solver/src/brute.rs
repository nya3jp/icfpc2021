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



use chun_bruteforce_solver::geom::*;
use chun_bruteforce_solver::{get_problem, Problem, Solution};



fn get_placeable_positions(prob: &Problem) -> Vec<(i64, i64)>
{
    let minx = prob.hole.iter().fold(1 << 20, |acc, x| std::cmp::min(acc, x.0));
    let maxx = prob.hole.iter().fold(-1 << 20, |acc, x| std::cmp::max(acc, x.0));
    let miny = prob.hole.iter().fold(1 << 20, |acc, x| std::cmp::min(acc, x.1));
    let maxy = prob.hole.iter().fold(-1 << 20, |acc, x| std::cmp::max(acc, x.1));
    let mut ret = Vec::new();
    for x in minx..=maxx {
        for y in miny..=maxy {
            match contains(&prob.hole, &(x, y)) {
                ContainsResult::ON | ContainsResult::IN => ret.push((x, y)),
                ContainsResult::OUT => ()
            }
        }
    };
    ret
}

fn visit_order_dfs(ptr: usize, node: usize, edges: &Vec<(usize, usize)>, visited: &mut Vec<bool>, resorder: &mut Vec<usize>, n: usize) 
{
    visited[node] = true;
    resorder[ptr] = node;
    if ptr == n - 1 {
        return
    }
    let mut conn = vec![0usize; n];
    for (v1, v2) in edges {
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
    let n = prob.figure.vertices.len();
    let mut degrees = vec![0i64; n];
    for (v1, v2) in prob.figure.edges.iter() {
        degrees[*v1] += 1;
        degrees[*v2] += 1;
    }
    let mut deg_and_ix: Vec<(i64, usize)> = degrees.into_iter().enumerate().map(|(idx, v)| (v, idx)).collect();
    deg_and_ix.sort_unstable_by(|a, b| b.cmp(a));
    let firstnode = deg_and_ix[0].1;

    let mut visited = vec![false; n];
    let mut resorder = vec![0usize; n];
    visit_order_dfs(0usize, firstnode, &prob.figure.edges, &mut visited, &mut resorder, n);
    resorder
}



fn pt_sub(p1: &(i64, i64), p2: &(i64, i64)) -> (i64, i64) {
    (p1.0 - p2.0, p1.1 - p2.1)
}

fn norm_sqr(p: &(i64, i64)) -> i64 {
    p.0 * p.0 + p.1 * p.1
}
fn eval_score(prob: &Problem, figure: &Vec<(i64, i64)>) -> f64 
{
    let mut score = 0.;
    for h in prob.hole.iter() {
        score += figure.iter().map(|v| norm_sqr(&pt_sub(v, h))).min().unwrap() as f64
    }
    score
}

fn do_brute(ptr: usize, placeable: &Vec<(i64, i64)>, visit_order: &Vec<usize>, prob: &Problem, bestscore: &mut f64, resfigure: &mut Vec<(i64, i64)>, bestfigure: &mut Vec<(i64, i64)>)
{
    // println!("depth {}/{}", ptr, resfigure.len());
    let trypos = visit_order[ptr];
    for trialpt in placeable {
        //println!("trialpt = {}, {}", trialpt.0, trialpt.1);
        resfigure[trypos] = *trialpt;

        let mut isok = true;
        for (v1, v2) in prob.figure.edges.iter() {
            let (v1, v2) = if *v2 == trypos { (*v2, *v1) } else { (*v1, *v2) };
            if v1 != trypos {
                continue;
            }
            for j in 0..ptr {
                if v2 != visit_order[j] {
                    continue; 
                }
                let p1 = prob.figure.vertices[v1];
                let p2 = prob.figure.vertices[v2];
                let d1 = (p1.0 - p2.0) * (p1.0 - p2.0) + (p1.1 - p2.1) * (p1.1 - p2.1);
                let q1 = resfigure[v1];
                let q2 = resfigure[v2];
                let d2 = (q1.0 - q2.0) * (q1.0 - q2.0) + (q1.1 - q2.1) * (q1.1 - q2.1);    
                // if d1 < d2
                //   | d2/d1 - 1 | = d2/d1 - 1
                //   <=> check d2 * 1000000 - d1 * 1000000 <= eps * d1
                // else
                //   | d2/d1 - 1 | = 1 - d2/d1
                //   <=>check d1 * 1000000 - d2 * 1000000 <= eps * d1
                let lhs = if d1 < d2 {
                    d2 * 1000000 - d1 * 1000000
                } else {
                    d1 * 1000000 - d2 * 1000000
                };
                let rhs = prob.epsilon * d1;
                if lhs > rhs {
                    isok = false;
                    //println!("invalid edge {} {}", v1, v2);
                    break;
                }
                // collision check (not strict version)
                for i in 0..prob.hole.len() {
                    let h1 = &prob.hole[i];
                    let h2 = &prob.hole[(i + 1) % prob.hole.len()];
                    if is_crossing(&q1, &q2, h1, h2) {
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
                if ! is_valid_solution(prob, resfigure) { continue; }
                let score = eval_score(prob, resfigure);
                println!("Score {}", score);
                if score < *bestscore {
                    println!("Updated best {} -> {}", bestscore, score);
                    *bestscore = score;
                    *bestfigure = resfigure.clone();
                }
            }
            else
            {
                do_brute(ptr + 1, placeable, visit_order, prob, bestscore, resfigure, bestfigure);
            }
        }
    }
}

pub fn brute(prob: &Problem) -> (f64, Solution) {
    let placeable = get_placeable_positions(prob);
    let visit_order = get_visit_order(prob);
    println!("visit_order = {:?}", visit_order);
    let n = prob.figure.vertices.len();
    let mut bestscore = 1e8;
    let mut resfigure = vec![(0i64, 0i64); n];
    let mut bestfigure = vec![(0i64, 0i64); n];
    do_brute(0usize, &placeable, &visit_order, &prob, &mut bestscore, &mut resfigure, &mut bestfigure);
    (bestscore, Solution {vertices: bestfigure})
}


