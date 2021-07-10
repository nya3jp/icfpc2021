

use chun_triangle_solver::geom::*;
use chun_triangle_solver::{get_problem, Problem, Solution};



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

#[derive(Hash,Clone,Debug)]
enum Plan {
    FixVertex (usize),
    FixEdge { edgeid: usize, basevertex: usize, newvertex: usize },
    TestEdge (usize),
    FixTriangle { edgeids: (usize, usize, usize), vertices: (usize, usize, usize) },
}

#[derive(Debug)]
enum PrepInfo {
    PFixVertex (Vec<(i64, i64)>),
    PFixEdge (Vec<(i64, i64)>),
    PFixTriangle (Vec<[(i64, i64); 3]>),
    PNone
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

fn plan_search(prob: &Problem) -> Vec<Plan> {
    let n = prob.figure.vertices.len();

    // get degrees and its accompanying info
    let mut degrees = vec![0i64; n];
    for (v1, v2) in prob.figure.edges.iter() {
        degrees[*v1] += 1;
        degrees[*v2] += 1;
    }
    let mut deg_and_ix: Vec<(i64, usize)> = degrees.iter().enumerate().map(|(idx, v)| (-v, idx)).collect();
    deg_and_ix.sort_unstable();
    //let firstnode = deg_and_ix[0].1;
    println!("Degree calculation completed: {:?}", degrees);

    // get triangles in the poses. Values are edge ids.
    let mut triangles: Vec<(usize, usize, usize)> = Vec::new();
    let nedge = prob.figure.edges.len();
    for i in 0..nedge {
        let (vi1, vi2) = prob.figure.edges[i];
        let (vi1, vi2) = if vi1 > vi2 { (vi2, vi1) } else { (vi1, vi2) };
        for j in (i + 1)..nedge {
            let (vj1, vj2) = prob.figure.edges[j];
            let (vj1, vj2) = if vj1 > vj2 { (vj2, vj1) } else { (vj1, vj2) };
            if vi1 != vj1 && vi1 != vj2 && vi2 != vj1 && vi2 != vj2 {
                continue;
            }
            let jicommon = if vi1 == vj1 || vi1 == vj2 { vi1 } else { vi2 };
            let iremain = if vi1 == jicommon { vi2 } else { vi1 };
            let jremain = if vj1 == jicommon { vj2 } else { vj1 };
            for k in (j + 1)..nedge {
                let (vk1, vk2) = prob.figure.edges[k];
                if (vk1, vk2) == (iremain, jremain) || (vk2, vk1) == (iremain, jremain) {
                    // found
                    triangles.push((i, j, k));
                }
            }
        }
    }
    println!("Found triangles: (denoted by edge id) {:?}", triangles);

    // Generate plans
    let mut resorder : Vec<Plan> = Vec::new();
    let mut remain_vertices = n;
    let mut remain_edges = nedge;
    let mut vertices_used = vec![false; n];
    let mut edges_used = vec![false; nedge];
    let mut available_triangles = triangles.clone();


    while remain_vertices > 0 && remain_edges > 0 {
        // vertex connectivity score
        let mut vertex_score = vec![0; n];
        for i in 0..nedge {
            if edges_used[i] { continue; }
            let (vi, vj) = prob.figure.edges[i];
            if !vertices_used[vi] && !vertices_used[vj] { continue; } // not connected
            let newvertex = if vertices_used[vi] { vj } else { vi };
            vertex_score[newvertex] += 1
        }
        let plan = {
            if available_triangles.len() > 0 {
                // FIXME find most well-connected triangle here
                let mut best_connection_score = -1; 
                let mut best_triangle = 0xfffffffff as usize;
                for (itriangle, (i, j, k)) in available_triangles.iter().enumerate() {
                    let mut total_connected = 0;
                    let (vi1, vi2) = prob.figure.edges[*i];
                    let (vj1, vj2) = prob.figure.edges[*j];
                    let (vk1, vk2) = prob.figure.edges[*k];
                    let score = [vi1, vi2, vj1, vj2, vk1, vk2].iter().map(|v| vertex_score[*v]).fold(0, |acc, x| acc + x) / 2;
                    if score > best_connection_score {
                        best_connection_score = score;
                        best_triangle = itriangle;
                    }
                }
                assert!(best_connection_score != -1, "There are triangles but scored -1??");
                let edgeids = available_triangles[best_triangle];
                let vertices = {
                    let (i, j, k) = edgeids;
                    let (vi1, vi2) = prob.figure.edges[i];
                    let (vj1, vj2) = prob.figure.edges[j];
                    let (vk1, vk2) = prob.figure.edges[k];
                    let jicommon = if vi1 == vj1 || vi1 == vj2 { vi1 } else { vi2 };
                    let iremain = if vi1 == jicommon { vi2 } else { vi1 };
                    let jremain = if vj1 == jicommon { vj2 } else { vj1 };
                    (jicommon, iremain, jremain)
                };
                available_triangles.remove(best_triangle);
                Plan::FixTriangle {edgeids, vertices}
            }else if remain_vertices < n {
                assert!(remain_edges > 0, "There should be remaining edges");
                // find best connected new vertex
                let mut best_vertex_score = -1;
                let mut best_vertex_edge = 0xfffffffff as usize;
                let mut best_vertex_newvertex = 0xfffffffff as usize;
                let mut best_vertex_basevertex = 0xfffffffff as usize;
                for i in 0..nedge {
                    if edges_used[i] { continue; }
                    let (vi, vj) = prob.figure.edges[i];
                    if !vertices_used[vi] && !vertices_used[vj] { continue; } // not connected
                    if vertices_used[vi] && vertices_used[vj] { panic!("vertices are both used by edges are not considered used") }
                    let newvertex = if vertices_used[vi] { vj } else { vi };
                    if vertex_score[newvertex] > best_vertex_score {
                        best_vertex_score = vertex_score[newvertex];
                        best_vertex_edge = i;
                        best_vertex_newvertex = newvertex;
                        best_vertex_basevertex = if vertices_used[vi] { vi } else { vj }
                    }
                }
                println!("best_vertex_score = {}", best_vertex_score);
                assert!(best_vertex_score > 0, "There are remaining edges but no new vertices");
                // 
                Plan::FixEdge {edgeid: best_vertex_edge, basevertex: best_vertex_basevertex, newvertex: best_vertex_newvertex }
            }else{
                // Brand new state, without triangle
                // choose the highest degree point to start from
                Plan::FixVertex (deg_and_ix[0].1)
            }
        };
        println!("Picking Plan {:?}", plan);
        resorder.push(plan.clone());
        // Fix and check connected edges
        let (newvertices, newedgescoveredbyfix) = {
            match plan {
                Plan::FixTriangle {edgeids, vertices} =>
                    (vec![vertices.0, vertices.1, vertices.2], vec![edgeids.0, edgeids.1, edgeids.2]),
                Plan::FixEdge {edgeid, basevertex, newvertex} =>
                (vec![newvertex], vec![edgeid]), // edge is checked by FixEdge itself,
                Plan::FixVertex (newvertex) =>
                (vec![newvertex], Vec::new()),
                Plan::TestEdge (_) => panic!("TestEdge should not come here")
            }
        };
        for v in newvertices.iter() {
            vertices_used[*v] = true;
            remain_vertices -= 1;
        }
        for e in newedgescoveredbyfix.iter() {
            edges_used[*e] = true;
            remain_edges -= 1;
        }
        // New edges to check
        for (edgeid, (v1, v2)) in prob.figure.edges.iter().enumerate() {
            if edges_used[edgeid] {
                continue;
            }
            let v1c = newvertices.iter().any(|x| x == v1);
            let v2c = newvertices.iter().any(|x| x == v2);
            if !(v1c || v2c) { continue };
            let (v1, v2) = if v1c { (v2, v1) } else { (v1, v2) };
            // now v2 is newvert, v1 may be visited vert
            if vertices_used[*v1] {
                // visited. Need to check 
                let newplan = Plan::TestEdge(edgeid);
                edges_used[edgeid] = true;
                println!("Picking Plan {:?}", newplan);
                resorder.push(newplan);
            }

        }
    }
    resorder
}


fn check_distance (prob: &Problem, v1idx: usize, v2idx: usize, q1: (i64, i64), q2: (i64, i64)) -> bool {
    let p1 = prob.figure.vertices[v1idx];
    let p2 = prob.figure.vertices[v2idx];
    let d1 = (p1.0 - p2.0) * (p1.0 - p2.0) + (p1.1 - p2.1) * (p1.1 - p2.1);
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
        return false;
    }
    return true;
}

fn get_all_delta_combination(prob: &Problem, v1idx: usize, v2idx: usize) -> Vec<(i64, i64)> {
    let mut ret = Vec::new();
    let p1 = prob.figure.vertices[v1idx];
    let p2 = prob.figure.vertices[v2idx];
    let d1 = (p1.0 - p2.0) * (p1.0 - p2.0) + (p1.1 - p2.1) * (p1.1 - p2.1);
    let d2max = (prob.epsilon + 1_000_000) * d1 / 1_000_000;
    let d2min = (-prob.epsilon + 1_000_000) * d1 / 1_000_000;
    let dmax = ((d2max as f64).sqrt().ceil()) as i64;
    let dmin = ((d2min as f64).sqrt().floor()) as i64;
    for absx in dmin..=dmax {
        let absymax = ((std::cmp::max(d2max - absx * absx, 0) as f64).sqrt().ceil()) as i64;
        let absymin = ((std::cmp::max(d2min - absx * absx, 0) as f64).sqrt().ceil()) as i64;
        for absy in absymin..=absymax {
            let d = absx * absx + absy * absy;
            if d <= d2max && d >= d2min {
                ret.push((absx, absy));
                ret.push((absx, -absy));
                ret.push((-absx, absy));
                ret.push((-absx, -absy));
            }
        }
    };
    ret
}

/*
enum PrepInfo {
    PFixVertex (Vec<(i64, i64)>),
    PFixEdge (Vec<(i64, i64)>),
    PFixTriangle (Vec<[(i64, i64); 3]>),
    PNone
*/
fn precompute_plan(prob: &Problem, resorder: &Vec<Plan>) -> Vec<PrepInfo> {
    println!("Precomputing plans");
    let placeable = get_placeable_positions(prob);
    println!("placeable: {} pts", placeable.len());
    println!("eps: {}", prob.epsilon);
    resorder.iter().enumerate().map(|(iplan, plan)| 
        match plan {
            Plan::FixTriangle {vertices, edgeids} => {
                let mut retvec = Vec::new();
                for x in placeable.iter() {
                    for y in placeable.iter() {
                        if !check_distance(prob, vertices.0, vertices.1, *x, *y) {
                            continue;
                        }
                        for z in placeable.iter() {
                            if (!check_distance(prob, vertices.0, vertices.2, *x, *z) ||
                                !check_distance(prob, vertices.1, vertices.2, *y, *z)) {
                                continue;
                            }
                            retvec.push([*x, *y, *z]);
                        }
                    }
                }
                println!("Plan {}: Triangle with {} possible combinations", iplan, retvec.len());
                PrepInfo::PFixTriangle(retvec)
            },
            Plan::FixEdge {edgeid, basevertex, newvertex } => {
                let mut deltas = get_all_delta_combination(prob, *basevertex, *newvertex);
                println!("Plan {}: Fixed edge with {} possible combinations", iplan, deltas.len());
                PrepInfo::PFixEdge(deltas)
            },
            Plan::FixVertex (vertexid) =>
                PrepInfo::PFixVertex(placeable.clone()),
            Plan::TestEdge (_) =>
                PrepInfo::PNone
        }
    ).collect()
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
    let visit_order = plan_search(prob);
    println!("visit_order = {:?}", visit_order);
    let precomputed = precompute_plan(prob, &visit_order);
    panic!("debug stop");
    let visit_order = panic!("");
    let n = prob.figure.vertices.len();
    let mut bestscore = 1e8;
    let mut resfigure = vec![(0i64, 0i64); n];
    let mut bestfigure = vec![(0i64, 0i64); n];
    do_brute(0usize, &placeable, &visit_order, &prob, &mut bestscore, &mut resfigure, &mut bestfigure);
    (bestscore, Solution {vertices: bestfigure})
}


