export type Point = [number, number];
export type Edge = [number, number];
export type Hole = Point[];

export interface Figure {
    edges: Edge[];
    vertices: Point[];
}

export interface ProblemData {
    hole: Hole;
    figure: Figure;
    epsilon: number;
}

export interface Problem {
    problem_id: string;
    created_at: number;
    data: ProblemData;
}

export interface SolutionData {
    vertices: Point[];
}

export interface Solution {
    problem_id: string;
    solution_id: string;
    created_at: number;
    tags: string[];
    solution_sets: string[];
    data: SolutionData;
}
