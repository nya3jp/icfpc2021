

use geom::{
    is_crossing,
    Polygon,
    point::Point,
    // polygon::ContainsResult,
    schema::{BonusType, Edge, Pose, Hole, Problem as P, UsedBonus},
};
use scorer::{is_inside_hole,dislike};
//use chun_triangle_solver::{get_problem};

use std::iter::FromIterator;
use std::collections::HashSet;

fn get_placeable_positions(prob: &P) -> Vec<Point>
{
    let minx = prob.hole.iter().fold(1 << 20, |acc, p| std::cmp::min(acc, p.x as i64));
    let maxx = prob.hole.iter().fold(-1 << 20, |acc, p| std::cmp::max(acc, p.x as i64));
    let miny = prob.hole.iter().fold(1 << 20, |acc, p| std::cmp::min(acc, p.y as i64));
    let maxy = prob.hole.iter().fold(-1 << 20, |acc, p| std::cmp::max(acc, p.y as i64));
    let mut ret = Vec::new();
    for x in minx..=maxx {
        for y in miny..=maxy {
            if prob.hole.contains(&Point {x: x as f64, y: y as f64}) {
                ret.push(Point {x: x as f64, y: y as f64})
            }
        }
    };
    ret
}

fn has_collision(hole: &Hole, p1: &Point, p2: &Point) -> bool
{
    for i in 0..hole.polygon.vertices.len() {
        let h1 = &hole.polygon.vertices[i];
        let h2 = &hole.polygon.vertices[(i + 1) % hole.polygon.vertices.len()];
        if is_crossing(p1, p2, h1, h2) {
            return true;
        }
    }
    return false;
}

struct DistanceCheckInfo {
    lhs: f64, d1 : f64  
}

fn check_distance_impl (prob: &Problem, v1idx: usize, v2idx: usize, q1: &Point, q2: &Point) -> DistanceCheckInfo {
    let p1 = prob.problem.figure.vertices[v1idx];
    let p2 = prob.problem.figure.vertices[v2idx];
    let delta = p1 - p2;
    let d1 = delta.x * delta.x + delta.y * delta.y;
    let qd = q1 - q2;
    let d2 = qd.x * qd.x + qd.y * qd.y;
    // if d1 < d2
    //   | d2/d1 - 1 | = d2/d1 - 1
    //   <=> check d2 * 1000000 - d1 * 1000000 <= eps * d1
    // else
    //   | d2/d1 - 1 | = 1 - d2/d1
    //   <=>check d1 * 1000000 - d2 * 1000000 <= eps * d1
    let lhs = if d1 < d2 {
        d2 * 1000000. - d1 * 1000000.
    } else {
        d1 * 1000000. - d2 * 1000000.
    };
    DistanceCheckInfo { lhs, d1 }
}

fn check_distance (prob: &Problem, v1idx: usize, v2idx: usize, q1: &Point, q2: &Point) -> bool {
    let dinfo = check_distance_impl(prob, v1idx, v2idx, q1, q2);
    let rhs = prob.problem.epsilon as f64 * dinfo.d1;
    if dinfo.lhs > rhs {
        return false;
    }
    return true;
}

fn calc_distance_ratio_penalty (prob: &Problem, v1idx: usize, v2idx: usize, q1: &Point, q2: &Point) -> f64 {
    let dinfo = check_distance_impl(prob, v1idx, v2idx, q1, q2);
    return dinfo.lhs / dinfo.d1; // budget = epsilon * nedge
}

#[derive(Clone,Debug)]
pub struct Problem {
    pub problem: P,
    pub use_bonus: Option<UsedBonus>,
    pub get_bonuses: Vec<BonusType>,
    pub init_state: Option<Pose>,
    pub fixed_vertices: Vec<usize>
}

#[derive(Clone,Debug)]
enum Plan {
    FixSolutionVertex { vertex: usize, point: Point },
    FixVertex (usize),
    FixEdge { edgeid: usize, basevertex: usize, newvertex: usize },
    TestEdge (usize),
    FixTriangle { edgeids: [usize; 3], vertices: (usize, usize, usize) },
}

#[derive(Clone,Debug)]
enum PrepInfo {
    PFixVertex (Vec<Point>),
    PFixEdge (Vec<Point>),
    PFixTriangle (Vec<[Point; 3]>),
    PNone
}

#[derive(Clone,Debug)]
enum BonusPlanMod {
    UseWallhack { vertex: usize },
    UseSuperFlex { edgeid: usize },
    UseGlobalist, // Use global setup
    UseBereakALeg, // Unsupported yet
    UseNothing
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

fn plan_search(prob: &Problem, bonus_use_plan: &BonusPlanMod) -> Vec<Plan> {
    let n = prob.problem.figure.vertices.len();

    let mut fixed_vertices = prob.fixed_vertices.clone();
    println!("Fixed {} vertices: {:?}", fixed_vertices.len(), fixed_vertices);

    // get degrees and its accompanying info
    let mut degrees = vec![0i64; n];
    for Edge {v1, v2} in prob.problem.figure.edges.iter() {
        degrees[*v1] += 1;
        degrees[*v2] += 1;
    }
    let mut deg_and_ix: Vec<(i64, usize)> = degrees.iter().enumerate().map(|(idx, v)| (-v, idx)).collect();
    deg_and_ix.sort_unstable();
    //let firstnode = deg_and_ix[0].1;
    println!("Degree calculation completed: {:?}", degrees);

    // get triangles in the poses. Values are edge ids.
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    let nedge = prob.problem.figure.edges.len();
    for i in 0..nedge {
        let Edge {v1: vi1, v2:vi2} = prob.problem.figure.edges[i];
        let (vi1, vi2) = if vi1 > vi2 { (vi2, vi1) } else { (vi1, vi2) };
        for j in (i + 1)..nedge {
            let Edge {v1:vj1, v2:vj2} = prob.problem.figure.edges[j];
            let (vj1, vj2) = if vj1 > vj2 { (vj2, vj1) } else { (vj1, vj2) };
            if vi1 != vj1 && vi1 != vj2 && vi2 != vj1 && vi2 != vj2 {
                continue;
            }
            let jicommon = if vi1 == vj1 || vi1 == vj2 { vi1 } else { vi2 };
            let iremain = if vi1 == jicommon { vi2 } else { vi1 };
            let jremain = if vj1 == jicommon { vj2 } else { vj1 };
            for k in (j + 1)..nedge {
                let Edge {v1: vk1, v2: vk2} = prob.problem.figure.edges[k];
                if (vk1, vk2) == (iremain, jremain) || (vk2, vk1) == (iremain, jremain) {
                    // found
                    triangles.push([i, j, k]);
                }
            }
        }
    }
    println!("Found triangles: (denoted by edge id) {:?}", triangles);
    if fixed_vertices.len() > 0 { triangles = Vec::new() } // delete triangles if using fixed vertices

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
            let Edge {v1: vi, v2: vj} = prob.problem.figure.edges[i];
            if !vertices_used[vi] && !vertices_used[vj] { continue; } // not connected
            let newvertex = if vertices_used[vi] { vj } else { vi };
            vertex_score[newvertex] += 1
        }
        let plan = {
            if fixed_vertices.len() > 0 {
                let v = fixed_vertices.pop().unwrap();
                Plan::FixSolutionVertex {vertex: v, point: prob.init_state.as_ref().unwrap().vertices[v] }
            } else if available_triangles.len() > 0 {
                // FIXME find most well-connected triangle here
                let mut best_connection_score = -1; 
                let mut best_triangle = 0xfffffffff as usize;
                for (itriangle, [i, j, k]) in available_triangles.iter().enumerate() {
                    let Edge {v1: vi1, v2: vi2} = prob.problem.figure.edges[*i];
                    let Edge {v1: vj1, v2: vj2} = prob.problem.figure.edges[*j];
                    let Edge {v1: vk1, v2: vk2} = prob.problem.figure.edges[*k];
                    let score = [vi1, vi2, vj1, vj2, vk1, vk2].iter().map(|v| vertex_score[*v]).fold(0, |acc, x| acc + x) / 2;
                    if score > best_connection_score {
                        best_connection_score = score;
                        best_triangle = itriangle;
                    }
                }
                assert!(best_connection_score != -1, "There are triangles but scored -1??");
                let edgeids = available_triangles[best_triangle];
                let vertices = {
                    let [i, j, _k] = edgeids;
                    let Edge {v1: vi1, v2: vi2} = prob.problem.figure.edges[i];
                    let Edge {v1: vj1, v2: vj2} = prob.problem.figure.edges[j];
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
                    let Edge {v1: vi, v2: vj} = prob.problem.figure.edges[i];
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
        let (newvertices, newedgescoveredbyfix): (Vec<usize>, Vec<usize>) = {
            match plan {
                Plan::FixSolutionVertex {vertex, ..} => (vec![vertex], vec![]),
                Plan::FixTriangle {edgeids, vertices} =>
                    (vec![vertices.0, vertices.1, vertices.2], edgeids.to_vec()),
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
            available_triangles.retain(|[i, j, k]| {
                let edges = &prob.problem.figure.edges; 
                edges[*i].v1 != *v && edges[*i].v2 != *v &&
                edges[*j].v1 != *v && edges[*j].v2 != *v &&
                edges[*k].v1 != *v && edges[*k].v2 != *v 
            });
        }
        for e in newedgescoveredbyfix.iter() {
            edges_used[*e] = true;
            remain_edges -= 1;
            available_triangles.retain(|[i, j, k]| i != e && j != e && k != e);
        }
        // New edges to check
        for (edgeid, Edge {v1, v2}) in prob.problem.figure.edges.iter().enumerate() {
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


fn get_all_delta_combination(prob: &Problem, v1idx: usize, v2idx: usize, numedge: Option<usize>) -> Vec<Point> {
    let epsilon = match numedge { Some(multiple) => multiple as i64, None => 1 } * prob.problem.epsilon;
    let mut ret = Vec::new();
    let p1 = prob.problem.figure.vertices[v1idx];
    let p2 = prob.problem.figure.vertices[v2idx];
    let d1 = ((p1.x - p2.x) * (p1.x - p2.x) + (p1.y - p2.y) * (p1.y - p2.y)) as i64;
    let d2maxe6 = (epsilon + 1_000_000) * d1;
    let d2mine6 = (-epsilon + 1_000_000) * d1;
    let dmax = ((d2maxe6 as f64 / 1e6).sqrt().ceil()) as i64;
    for absx in 0..=dmax {
        let absymax = ((std::cmp::max(d2maxe6 - absx * absx * 1_000_000, 0) as f64 / 1e6).sqrt().ceil()) as i64;
        let absymin = ((std::cmp::max(d2mine6 - absx * absx * 1_000_000, 0) as f64 / 1e6).sqrt().floor()) as i64;
        for absy in absymin..=absymax {
            let d = absx * absx + absy * absy;
            if d * 1_000_000 <= d2maxe6 && d * 1_000_000 >= d2mine6 {
                let ax = absx as f64;
                let ay = absy as f64;
                ret.push(Point {x: ax, y: ay});
                ret.push(Point {x: ax, y:-ay});
                ret.push(Point {x:-ax, y: ay});
                ret.push(Point {x:-ax, y:-ay});
            }
        }
    };
    ret.sort_unstable_by(|a, b| (a.x, a.y).partial_cmp(&(b.x, b.y)).unwrap() );
    ret.dedup();
    ret
}

fn precompute_plan(prob: &Problem, resorder: &Vec<Plan>) -> Vec<PrepInfo> {
    let edgenum_or_one =
        if prob.use_bonus.iter().any(|b| b.bonus == BonusType::GLOBALIST) {
            Some(prob.problem.figure.edges.len()) 
        } else {
            None
        };
    println!("Precomputing plans");
    let placeable = get_placeable_positions(&prob.problem);
    println!("placeable: {} pts", placeable.len());
    println!("eps: {}", prob.problem.epsilon);
    resorder.iter().enumerate().map(|(iplan, plan)| 
        match plan {
            Plan::FixSolutionVertex {..} => PrepInfo::PNone,
            Plan::FixTriangle {vertices: (xi, yi, zi), edgeids} => {
                let delta_xy = get_all_delta_combination(prob, *xi, *yi, edgenum_or_one);
                let delta_xz = get_all_delta_combination(prob, *xi, *zi, edgenum_or_one);
                let delta_yz = get_all_delta_combination(prob, *yi, *zi, edgenum_or_one);
                let hash_delta_yz: HashSet<(i64, i64)> = delta_yz.into_iter().map(|d| (d.x as i64, d.y as i64)).collect();
                println!("Triangle: xy edge visiting pattern {}", delta_xy.len());
                println!("Triangle: xz edge visiting pattern {}", delta_xz.len());
                println!("Triangle: yz edge visiting pattern {}", delta_xz.len());
                let mut deltaxy_deltaxz = Vec::new();
                for dxy in delta_xy {
                    for dxz in &delta_xz {
                        let dyz = ((dxz.x - dxy.x) as i64, (dxz.y - dxy.y) as i64);
                        if hash_delta_yz.contains(&dyz) {
                            deltaxy_deltaxz.push((dxy, dxz));
                        }
                    }
                }
                println!("After merging: (dxy, dxz) pair has {} pairs", deltaxy_deltaxz.len());
                let mut retvec = Vec::new();
                //let mut retvec2 = Vec::new();
                //let placeable_set : HashSet<(i64, i64)> = HashSet::from_iter(placeable.iter().cloned());
                for xplace in placeable.iter() {
                    for (dxy, dxz) in deltaxy_deltaxz.iter() {
                        let yplace = Point {x: xplace.x + dxy.x, y: xplace.y + dxy.y};
                        let zplace = Point {x: xplace.x + dxz.x, y: xplace.y + dxz.y};
                        /*
                        assert!(check_distance(prob, *xi, *yi, *xplace, yplace), "x-y distance inconsistent: xp {:?}, yp {:?}, xi{}, yi{} figx {:?} figy {:?}",
                             *xplace, yplace, xi, yi, prob.figure.vertices[*xi], prob.figure.vertices[*yi]);
                        assert!(check_distance(prob, *xi, *zi, *xplace, zplace), "x-z distance inconsistent");
                        */

                        if ! prob.problem.hole.contains(&yplace) {
                            continue;
                        }
                        if ! prob.problem.hole.contains(&zplace) {
                            continue;
                        }

                        // TODO flexible
                        if has_collision(&prob.problem.hole, xplace, &yplace) || has_collision(&prob.problem.hole, xplace, &zplace) || has_collision(&prob.problem.hole, &yplace, &zplace) {
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
                let mut deltas = get_all_delta_combination(prob, *basevertex, *newvertex, edgenum_or_one);
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




fn do_brute(ptr: usize, solve_plan: &Vec<Plan>, precompute_info: &Vec<PrepInfo>, prob: &Problem, globalist_budget: f64, bestscore: &mut usize, resfigure: &mut Pose, bestfigure: &mut Pose)
{
    // println!("depth {}/{}", ptr, resfigure.len());
    if ptr == solve_plan.len() {
        // Success!
        if ! is_inside_hole(&prob.problem, resfigure) { return; }
        let score = dislike(&prob.problem.hole, resfigure);
        //println!("Score {}", score);
        if score < *bestscore {
            println!("Updated best {} -> {}: {{ \"vertices\": {:?} }}", bestscore, score, *resfigure);
            *bestscore = score;
            *bestfigure = resfigure.clone();
        }
        return;
    }
    // otherwise, fill as planned
    let use_globalist = prob.use_bonus.iter().any(|b| b.bonus == BonusType::GLOBALIST);
    let current_plan = &solve_plan[ptr];
    let current_precompute = &precompute_info[ptr];
    match (current_plan, current_precompute) {
        (Plan::FixSolutionVertex {vertex, point}, PrepInfo::PNone) => {
            resfigure.vertices[*vertex] = *point;
            do_brute(ptr + 1, solve_plan, precompute_info, prob, globalist_budget, bestscore, resfigure, bestfigure);
        },
        (Plan::FixTriangle {edgeids, vertices}, PrepInfo::PFixTriangle(threepts)) => {
            for pts in threepts.iter() {
                resfigure.vertices[vertices.0] = pts[0];
                resfigure.vertices[vertices.1] = pts[1];
                resfigure.vertices[vertices.2] = pts[2];
                // collision and distance checks are precomputed except globalist case
                let mut remain_globalist_budget = globalist_budget;
                if use_globalist {
                    let totglobalisteps = 
                        edgeids.iter().fold(0., |acc, e| {
                            let Edge {v1, v2} = prob.problem.figure.edges[*e];
                            acc + calc_distance_ratio_penalty(prob, v1, v2, &resfigure.vertices[v1], &resfigure.vertices[v2])
                        });
                    remain_globalist_budget -= totglobalisteps;
                    if remain_globalist_budget < 0. {
                        continue;
                    }
                }
                do_brute(ptr + 1, solve_plan, precompute_info, prob, remain_globalist_budget, bestscore, resfigure, bestfigure);
            }
        },
        (Plan::FixEdge {edgeid, basevertex, newvertex }, PrepInfo::PFixEdge(edgedeltas)) => {
            for edgedelta in edgedeltas.iter() {
                resfigure.vertices[*newvertex] = resfigure.vertices[*basevertex] + *edgedelta;
                if has_collision(&prob.problem.hole, &resfigure.vertices[*basevertex], &resfigure.vertices[*newvertex]) {
                    continue;
                }
                let mut remain_globalist_budget = globalist_budget;
                if use_globalist {
                    let v1 = basevertex;
                    let v2 = newvertex;
                    remain_globalist_budget -= calc_distance_ratio_penalty(prob, *v1, *v2, &resfigure.vertices[*v1], &resfigure.vertices[*v2]);
                    if remain_globalist_budget < 0. {
                        continue;
                    }
                }
                // this delta is already distance-checked
                do_brute(ptr + 1, solve_plan, precompute_info, prob, remain_globalist_budget, bestscore, resfigure, bestfigure);
            }
        },
        (Plan::FixVertex (vertid), PrepInfo::PFixVertex(placeable)) => {
            for p in placeable.iter() {
                resfigure.vertices[*vertid] = *p;
                do_brute(ptr + 1, solve_plan, precompute_info, prob, globalist_budget, bestscore, resfigure, bestfigure);
            }
        },
        (Plan::TestEdge (edgeid), PrepInfo::PNone) => {
            let Edge {v1, v2} = prob.problem.figure.edges[*edgeid];
            let mut remain_globalist_budget = globalist_budget;
            if use_globalist {
                let Edge {v1, v2} = prob.problem.figure.edges[*edgeid];
                remain_globalist_budget -= calc_distance_ratio_penalty(prob, v1, v2, &resfigure.vertices[v1], &resfigure.vertices[v2]);
                if remain_globalist_budget < 0. {
                    return;
                }
            }else{
                if !check_distance(prob, v1, v2, &resfigure.vertices[v1], &resfigure.vertices[v2]) {
                    return;
                }
            }
            if has_collision(&prob.problem.hole, &resfigure.vertices[v1], &resfigure.vertices[v2]) {
                return;
            }
            do_brute(ptr + 1, solve_plan, precompute_info, prob, remain_globalist_budget, bestscore, resfigure, bestfigure);
        },
        _ => {
            panic!("Plan/Precompute unmatched")
        }
    };
}

pub fn brute(prob: &Problem) -> (usize, Pose) {
    let bonus_use_plans = 
        match prob.use_bonus {
            Some(UsedBonus{bonus: BonusType::GLOBALIST, ..}) => vec![BonusPlanMod::UseGlobalist],
            Some(UsedBonus{bonus: BonusType::BREAK_A_LEG, ..}) => panic!("Break a leg: impl difficulty"),
            Some(UsedBonus{bonus: BonusType::SUPERFLEX, ..}) => panic!("superflex todo"),
            Some(UsedBonus{bonus: BonusType::WALLHACK, ..}) => panic!("todo"),
            None => vec![BonusPlanMod::UseNothing]
        };
    let mut bestscore = 99999999usize;
    let n = prob.problem.figure.vertices.len();
    let bonuses: Option<Vec<UsedBonus>> = match &prob.use_bonus {
        Some(b) => Some(vec![b.clone()]),
        None => None
    };
    let mut resfigure = Pose { vertices: vec![Point {x: 0., y: 0.}; n], bonuses: bonuses.clone() }; // FIXME TODO break_a_leg
    let mut bestfigure = Pose { vertices: vec![Point {x: 0., y: 0.}; n], bonuses: bonuses.clone() };
    for bonus_use_plan in bonus_use_plans {
        println!("retarting solver with plan {:?}", bonus_use_plan);
        let visit_order = plan_search(prob, &bonus_use_plan);
        println!("visit_order = {:?}", visit_order);
        let precomputed = precompute_plan(prob, &visit_order);
        let globalist_budget = prob.problem.epsilon as f64 * prob.problem.figure.edges.len() as f64;
        do_brute(0usize, &visit_order, &precomputed, &prob, globalist_budget, &mut bestscore, &mut resfigure, &mut bestfigure);
    }
    (bestscore, bestfigure)
}


