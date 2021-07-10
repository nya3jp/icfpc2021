

use chun_triangle_solver::geom::*;
use chun_triangle_solver::{get_problem, Problem, Solution};

use std::iter::FromIterator;
use std::collections::HashSet;

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
            available_triangles.retain(|(i, j, k)| {
                let edges = &prob.figure.edges; 
                edges[*i].0 != *v && edges[*i].1 != *v &&
                edges[*j].0 != *v && edges[*j].1 != *v &&
                edges[*k].0 != *v && edges[*k].1 != *v 
            });
        }
        for e in newedgescoveredbyfix.iter() {
            edges_used[*e] = true;
            remain_edges -= 1;
            available_triangles.retain(|(i, j, k)| i != e && j != e && k != e);
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
            Plan::FixTriangle {vertices: (xi, yi, zi), edgeids} => {
                let delta_xy = get_all_delta_combination(prob, *xi, *yi);
                let delta_xz = get_all_delta_combination(prob, *xi, *zi);
                let delta_yz = get_all_delta_combination(prob, *yi, *zi);
                let hash_delta_yz: HashSet<(i64, i64)> = delta_yz.into_iter().collect();
                println!("Triangle: xy edge visiting pattern {}", delta_xy.len());
                println!("Triangle: xz edge visiting pattern {}", delta_xz.len());
                println!("Triangle: yz edge visiting pattern {}", delta_xz.len());
                let mut deltaxy_deltaxz = Vec::new();
                for dxy in delta_xy {
                    for dxz in &delta_xz {
                        let dyz = (dxz.0 - dxy.0, dxz.1 - dxy.1);
                        if hash_delta_yz.contains(&dyz) {
                            deltaxy_deltaxz.push((dxy, dxz));
                        }
                    }
                }
                println!("After merging: (dxy, dxz) pair has {} pairs", deltaxy_deltaxz.len());
                let mut retvec = Vec::new();
                //let mut retvec2 = Vec::new();
                let placeable_set : HashSet<(i64, i64)> = HashSet::from_iter(placeable.iter().cloned());
                for xplace in placeable.iter() {
                    for (dxy, dxz) in deltaxy_deltaxz.iter() {
                        let yplace = (xplace.0 + dxy.0, xplace.1 + dxy.1);
                        let zplace = (xplace.0 + dxz.0, xplace.1 + dxz.1);
                        assert!(check_distance(prob, *xi, *yi, *xplace, yplace), "x-y distance inconsistent: xp {:?}, yp {:?}, xi{}, yi{} figx {:?} figy {:?}",
                             *xplace, yplace, xi, yi, prob.figure.vertices[*xi], prob.figure.vertices[*yi]);
                        assert!(check_distance(prob, *xi, *zi, *xplace, zplace), "x-z distance inconsistent");

                        if ! placeable_set.contains(&yplace) {
                            continue;
                        }
                        if ! placeable_set.contains(&zplace) {
                            continue;
                        }

                        if has_collision(&prob, xplace, &yplace) || has_collision(&prob, xplace, &zplace) || has_collision(&prob, &yplace, &zplace) {
                            continue;
                        }

                        retvec.push([*xplace, yplace, zplace]);
                        if retvec.len() > 400_000_000 {
                            panic!("OVer 400 M entries in triangle placement, exhausting memory for sure");
                        }
                    }
                }
                /*
                let merged_delta = 
                for x in placeable.iter() {
                    for y in placeable.iter() {
                        if !check_distance(prob, *xi, *yi, *x, *y) {
                            continue;
                        }
                        for z in placeable.iter() {
                            if (!check_distance(prob, *xi, *zi, *x, *z) ||
                                !check_distance(prob, *yi, *zi, *y, *z)) {
                                continue;
                            }
                            retvec2.push([*x, *y, *z]);
                        }
                    }
                };*/
                println!("Plan {}: Triangle with {} possible combinations", iplan, retvec.len());
                //println!("Plan {}: Triangle with {} possible combinations (dup check)", iplan, retvec2.len());
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




fn do_brute(ptr: usize, solve_plan: &Vec<Plan>, precompute_info: &Vec<PrepInfo>, prob: &Problem, bestscore: &mut f64, resfigure: &mut Vec<(i64, i64)>, bestfigure: &mut Vec<(i64, i64)>)
{
    // println!("depth {}/{}", ptr, resfigure.len());
    if ptr == solve_plan.len() {
        // Success!
        if ! is_valid_solution(prob, resfigure) { return; }
        let score = eval_score(prob, resfigure);
        //println!("Score {}", score);
        if score < *bestscore {
            println!("Updated best {} -> {}: {{ \"vertices\": {:?} }}", bestscore, score, *resfigure);
            *bestscore = score;
            *bestfigure = resfigure.clone();
        }
        return;
    }
    // otherwise, fill as planned
    let current_plan = &solve_plan[ptr];
    let current_precompute = &precompute_info[ptr];
    match (current_plan, current_precompute) {
        (Plan::FixTriangle {edgeids, vertices}, PrepInfo::PFixTriangle(threepts)) => {
            for pts in threepts.iter() {
                resfigure[vertices.0] = pts[0];
                resfigure[vertices.1] = pts[1];
                resfigure[vertices.2] = pts[2];
                // collision and distance checks are precomputed
                do_brute(ptr + 1, solve_plan, precompute_info, prob, bestscore, resfigure, bestfigure);
            }
        },
        (Plan::FixEdge {edgeid, basevertex, newvertex }, PrepInfo::PFixEdge(edgedeltas)) => {
            for edgedelta in edgedeltas.iter() {
                resfigure[*newvertex] = (resfigure[*basevertex].0 + edgedelta.0, resfigure[*basevertex].1 + edgedelta.1);
                if has_collision(prob, &resfigure[*basevertex], &resfigure[*newvertex]) {
                    continue;
                }
                // this delta is already distance-checked
                do_brute(ptr + 1, solve_plan, precompute_info, prob, bestscore, resfigure, bestfigure);
            }
        },
        (Plan::FixVertex (vertid), PrepInfo::PFixVertex(placeable)) => {
            for p in placeable.iter() {
                resfigure[*vertid] = *p;
                do_brute(ptr + 1, solve_plan, precompute_info, prob, bestscore, resfigure, bestfigure);
            }
        },
        (Plan::TestEdge (edgeid), PrepInfo::PNone) => {
            let (v1, v2) = prob.figure.edges[*edgeid];
            if !check_distance(prob, v1, v2, resfigure[v1], resfigure[v2]) {
                return;
            }
            if has_collision(prob, &resfigure[v1], &resfigure[v2]) {
                return;
            }
            do_brute(ptr + 1, solve_plan, precompute_info, prob, bestscore, resfigure, bestfigure);
        },
        _ => {
            panic!("Plan/Precompute unmatched")
        }
    };
}

pub fn brute(prob: &Problem) -> (f64, Solution) {
    let placeable = get_placeable_positions(prob);
    let visit_order = plan_search(prob);
    println!("visit_order = {:?}", visit_order);
    let precomputed = precompute_plan(prob, &visit_order);
    let n = prob.figure.vertices.len();
    let mut bestscore = 1e8;
    let mut resfigure = vec![(0i64, 0i64); n];
    let mut bestfigure = vec![(0i64, 0i64); n];
    do_brute(0usize, &visit_order, &precomputed, &prob, &mut bestscore, &mut resfigure, &mut bestfigure);
    (bestscore, Solution {vertices: bestfigure})
}


