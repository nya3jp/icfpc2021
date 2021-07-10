export type Point = [number, number];
export type Edge = [number, number];
export type Hole = Point[];

export interface Figure {
    edges: Edge[];
    vertices: Point[];
}

export interface Problem {
    hole: Hole;
    figure: Figure;
    epsilon: number;
}

export interface Pose {
    vertices: Point[];
}

export interface Solution {
    problem_id: string;
    solution_id: string;
    created_at: number;
    tags: string[];
    solution_sets: string[];
}

export type SolutionMap = { [key: string]: Solution };
export type PoseMap = { [key: string]: Pose };

export function solutionKey(problem_id: string, solution_id: string): string {
    return problem_id + "-" + solution_id;
}
