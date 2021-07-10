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
    tags: string[];
    solution_sets: string[];
}
